use super::tokens::Tokens;
use crate::onnx::{self, Feed, Runner, Workload};
use ort::environment::Environment;
use ort::value::{DynValue, PrimitiveTensorElementType, Tensor};
use serde::Deserialize;
use std::path::Path;

pub const RATE: usize = 16_000;
pub const CHUNK: usize = 1280;
const SAMPLE_BUFFER: usize = 79;
const CONV_TAPS: usize = 4;
const LEFT_CONTEXT_PER_LAYER: usize = 16;
const MAX_TOKENS: usize = 256;
const CONFIG_FILE: &str = "streaming_config.json";
const TOKENIZER_FILE: &str = "tokenizer.bin";
const FRONTEND_FILE: &str = "frontend.model.ort";
const WEIGHTS_FILE: &str = "frontend.weights.ort";
const ENCODER_FILE: &str = "encoder.ort";
const ADAPTER_FILE: &str = "adapter.ort";
const CROSS_KV_FILE: &str = "cross_kv.ort";
const DECODER_FILE: &str = "decoder_kv.ort";
pub const FILES: [&str; 8] =
    [ADAPTER_FILE, CROSS_KV_FILE, DECODER_FILE, ENCODER_FILE, FRONTEND_FILE, WEIGHTS_FILE, CONFIG_FILE, TOKENIZER_FILE];

#[derive(Deserialize)]
struct Config {
    encoder_dim: usize,
    decoder_dim: usize,
    depth: usize,
    nheads: usize,
    head_dim: usize,
    vocab_size: usize,
    bos_id: i64,
    eos_id: i64,
    total_lookahead: usize,
    d_model_frontend: usize,
    c1: usize,
}

pub struct Model {
    config: Config,
    frontend: Runner,
    weights: Vec<(String, DynValue)>,
    encoder: Runner,
    encoder_cache: bool,
    adapter: Runner,
    cross_kv: Runner,
    decoder: Runner,
    tokens: Tokens,
    tokens_per_second: f32,
}

pub struct Line {
    pending: Vec<f32>,
    samples: usize,
    sample_buffer: Vec<f32>,
    sample_len: i64,
    conv1: Vec<f32>,
    conv2: Vec<f32>,
    frame_count: i64,
    features: Vec<f32>,
    feature_count: usize,
    emitted: usize,
    pos_offset: i64,
    memory: Vec<f32>,
    memory_len: usize,
    cross: Option<(DynValue, DynValue)>,
    last: Vec<i64>,
}

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn take(outputs: &mut Vec<(String, DynValue)>, name: &str) -> Result<DynValue, String> {
    let at = outputs.iter().position(|(n, _)| n == name).ok_or_else(|| format!("missing output {name}"))?;
    Ok(outputs.swap_remove(at).1)
}

fn floats(value: &DynValue) -> Result<(Vec<i64>, &[f32]), String> {
    let (shape, data) = value.try_extract_tensor::<f32>().map_err(err)?;
    Ok((shape.to_vec(), data))
}

fn scalar(value: &DynValue) -> Result<i64, String> {
    let (_, data) = value.try_extract_tensor::<i64>().map_err(err)?;
    data.first().copied().ok_or_else(|| "empty scalar".into())
}

fn owned<T: PrimitiveTensorElementType + std::fmt::Debug + Clone + 'static, const N: usize>(
    shape: [usize; N],
    data: Vec<T>,
) -> Result<DynValue, String> {
    Ok(Tensor::from_array((shape, data)).map_err(err)?.into_dyn())
}

fn argmax(row: &[f32]) -> i64 {
    row.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(i, _)| i as i64)
}

impl Model {
    pub fn load(env: &Environment, dir: &Path, tokens_per_second: f32) -> Result<Model, String> {
        let text = std::fs::read_to_string(dir.join(CONFIG_FILE)).map_err(err)?;
        let config: Config = serde_json::from_str(&text).map_err(err)?;
        let runner = |file: &str| onnx::runner(env, &dir.join(file), Workload::Captions);
        let mut weights_session = runner(WEIGHTS_FILE)?;
        let weights = weights_session.run(Vec::new(), &[])?;
        let encoder = runner(ENCODER_FILE)?;
        let encoder_cache = encoder.input_names().iter().any(|n| n == "cache");
        Ok(Model {
            frontend: runner(FRONTEND_FILE)?,
            weights,
            encoder,
            encoder_cache,
            adapter: runner(ADAPTER_FILE)?,
            cross_kv: runner(CROSS_KV_FILE)?,
            decoder: runner(DECODER_FILE)?,
            tokens: Tokens::load(&dir.join(TOKENIZER_FILE))?,
            config,
            tokens_per_second,
        })
    }

    pub fn line(&self) -> Line {
        Line {
            pending: Vec::new(),
            samples: 0,
            sample_buffer: vec![0.0; SAMPLE_BUFFER],
            sample_len: 0,
            conv1: vec![0.0; self.config.d_model_frontend * CONV_TAPS],
            conv2: vec![0.0; self.config.c1 * CONV_TAPS],
            frame_count: 0,
            features: Vec::new(),
            feature_count: 0,
            emitted: 0,
            pos_offset: 0,
            memory: Vec::new(),
            memory_len: 0,
            cross: None,
            last: Vec::new(),
        }
    }

    fn frontend(&mut self, line: &mut Line, chunk: Vec<f32>) -> Result<(), String> {
        let c = &self.config;
        let mut feed = vec![
            ("audio_chunk".to_string(), Feed::Owned(owned([1, chunk.len()], chunk)?)),
            ("sample_buffer".to_string(), Feed::Owned(owned([1, SAMPLE_BUFFER], line.sample_buffer.clone())?)),
            ("sample_len".to_string(), Feed::Owned(owned([1], vec![line.sample_len])?)),
            ("conv1_buffer".to_string(), Feed::Owned(owned([1, c.d_model_frontend, CONV_TAPS], line.conv1.clone())?)),
            ("conv2_buffer".to_string(), Feed::Owned(owned([1, c.c1, CONV_TAPS], line.conv2.clone())?)),
            ("frame_count".to_string(), Feed::Owned(owned([1], vec![line.frame_count])?)),
        ];
        feed.extend(self.weights.iter().map(|(name, value)| (name.clone(), Feed::Ref(value))));
        let mut out = self.frontend.run(feed, &[])?;
        let features = take(&mut out, "features")?;
        let (shape, data) = floats(&features)?;
        let count = shape.get(1).copied().unwrap_or(0) as usize;
        line.features.extend_from_slice(&data[..count * self.config.encoder_dim]);
        line.feature_count += count;
        line.sample_buffer = floats(&take(&mut out, "sample_buffer_out")?)?.1.to_vec();
        line.sample_len = scalar(&take(&mut out, "sample_len_out")?)?;
        line.conv1 = floats(&take(&mut out, "conv1_buffer_out")?)?.1.to_vec();
        line.conv2 = floats(&take(&mut out, "conv2_buffer_out")?)?.1.to_vec();
        line.frame_count = scalar(&take(&mut out, "frame_count_out")?)?;
        Ok(())
    }

    fn encode(&mut self, line: &mut Line, last: bool) -> Result<(), String> {
        let c = &self.config;
        let total = line.feature_count;
        let stable = if last { total } else { total.saturating_sub(c.total_lookahead) };
        if stable <= line.emitted {
            return Ok(());
        }
        let new = stable - line.emitted;
        let start = line.emitted.saturating_sub(LEFT_CONTEXT_PER_LAYER * c.depth);
        let window = line.features[start * c.encoder_dim..total * c.encoder_dim].to_vec();
        let features = owned([1, total - start, c.encoder_dim], window)?;
        let mut feed = vec![("features".to_string(), Feed::Owned(features))];
        if self.encoder_cache {
            let cache = vec![0.0f32; c.depth * LEFT_CONTEXT_PER_LAYER * c.encoder_dim];
            feed.push(("cache".to_string(), Feed::Owned(owned([c.depth, 1, LEFT_CONTEXT_PER_LAYER, c.encoder_dim], cache)?)));
            feed.push(("cache_valid".to_string(), Feed::Owned(owned([LEFT_CONTEXT_PER_LAYER], vec![0.0f32; LEFT_CONTEXT_PER_LAYER])?)));
        }
        let mut out = self.encoder.run(feed, &[])?;
        let encoded = take(&mut out, "encoded")?;
        let (shape, data) = floats(&encoded)?;
        let offset = line.emitted - start;
        if shape.get(1).copied().unwrap_or(0) < (offset + new) as i64 {
            return Err("encoder window misaligned".into());
        }
        let slice = data[offset * c.encoder_dim..(offset + new) * c.encoder_dim].to_vec();
        let mut out = self.adapter.run(
            vec![
                ("encoded".to_string(), Feed::Owned(owned([1, new, c.encoder_dim], slice)?)),
                ("pos_offset".to_string(), Feed::Owned(owned([1], vec![line.pos_offset])?)),
            ],
            &[],
        )?;
        let memory = take(&mut out, "memory")?;
        line.memory.extend_from_slice(&floats(&memory)?.1[..new * c.decoder_dim]);
        line.memory_len += new;
        line.cross = None;
        line.emitted = stable;
        line.pos_offset += new as i64;
        Ok(())
    }

    pub fn feed(&mut self, line: &mut Line, samples: &[f32]) -> Result<(), String> {
        line.pending.extend_from_slice(samples);
        let whole = line.pending.len() / CHUNK * CHUNK;
        let chunks: Vec<f32> = line.pending.drain(..whole).collect();
        for chunk in chunks.chunks(CHUNK) {
            self.frontend(line, chunk.to_vec())?;
            line.samples += CHUNK;
        }
        self.encode(line, false)
    }

    pub fn finish(&mut self, line: &mut Line) -> Result<(), String> {
        if !line.pending.is_empty() {
            let mut chunk = std::mem::take(&mut line.pending);
            line.samples += chunk.len();
            chunk.resize(CHUNK, 0.0);
            self.frontend(line, chunk)?;
        }
        self.encode(line, true)
    }

    fn cross(&mut self, line: &mut Line) -> Result<(), String> {
        if line.cross.is_some() {
            return Ok(());
        }
        let memory = owned([1, line.memory_len, self.config.decoder_dim], line.memory.clone())?;
        let mut out = self.cross_kv.run(vec![("memory".to_string(), Feed::Owned(memory))], &[])?;
        line.cross = Some((take(&mut out, "k_cross")?, take(&mut out, "v_cross")?));
        Ok(())
    }

    fn empty_cache(&self) -> Result<(DynValue, DynValue), String> {
        let c = &self.config;
        let shape = [c.depth, 1, c.nheads, 0, c.head_dim];
        Ok((owned(shape, Vec::<f32>::new())?, owned(shape, Vec::<f32>::new())?))
    }

    fn step(&mut self, line: &Line, ids: &[i64], cache: &mut (DynValue, DynValue)) -> Result<Vec<i64>, String> {
        let (k_cross, v_cross) = line.cross.as_ref().ok_or("cross attention not ready")?;
        let mut out = self.decoder.run(
            vec![
                ("token".to_string(), Feed::Owned(owned([1, ids.len()], ids.to_vec())?)),
                ("k_self".to_string(), Feed::Ref(&cache.0)),
                ("v_self".to_string(), Feed::Ref(&cache.1)),
                ("out_k_cross".to_string(), Feed::Ref(k_cross)),
                ("out_v_cross".to_string(), Feed::Ref(v_cross)),
            ],
            &[],
        )?;
        let logits = take(&mut out, "logits")?;
        let vocab = self.config.vocab_size;
        let best = floats(&logits)?.1.chunks(vocab).map(argmax).collect();
        *cache = (take(&mut out, "out_k_self")?, take(&mut out, "out_v_self")?);
        Ok(best)
    }

    fn continue_from(&mut self, line: &Line, mut token: i64, result: &mut Vec<i64>, limit: usize, cache: &mut (DynValue, DynValue)) -> Result<(), String> {
        while token != self.config.eos_id && result.len() < limit {
            result.push(token);
            token = *self.step(line, &[token], cache)?.last().ok_or("no logits")?;
        }
        Ok(())
    }

    pub fn text(&mut self, line: &mut Line) -> Result<String, String> {
        if line.memory_len == 0 {
            return Ok(String::new());
        }
        self.cross(line)?;
        let seconds = line.samples as f32 / RATE as f32;
        let limit = ((seconds * self.tokens_per_second).ceil() as usize).clamp(1, MAX_TOKENS);
        let bos = self.config.bos_id;
        let draft: Vec<i64> = line.last.iter().copied().filter(|&t| t != bos && t != self.config.eos_id).take(limit).collect();
        let mut cache = self.empty_cache()?;
        let mut result = Vec::new();
        if draft.is_empty() {
            let first = self.step(line, &[bos], &mut cache)?[0];
            self.continue_from(line, first, &mut result, limit, &mut cache)?;
        } else {
            let input: Vec<i64> = std::iter::once(bos).chain(draft.iter().copied()).collect();
            let predictions = self.step(line, &input, &mut cache)?;
            let agreed = draft.iter().zip(&predictions).take_while(|(d, p)| d == p).count();
            result.extend_from_slice(&draft[..agreed]);
            if agreed == draft.len() {
                self.continue_from(line, predictions[agreed], &mut result, limit, &mut cache)?;
            } else {
                cache = self.empty_cache()?;
                let next = *self.step(line, &input[..=agreed], &mut cache)?.last().ok_or("no logits")?;
                self.continue_from(line, next, &mut result, limit, &mut cache)?;
            }
        }
        line.last = result;
        Ok(self.tokens.text(&line.last))
    }
}

#[cfg(test)]
impl Line {
    fn seconds(&self) -> f32 {
        (self.samples + self.pending.len()) as f32 / RATE as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Instant;

    const STEP: usize = 2560;

    fn wav(path: &Path) -> Vec<f32> {
        let data = std::fs::read(path).unwrap();
        let mut at = 12;
        let (mut channels, mut rate) = (1usize, 0u32);
        while at + 8 <= data.len() {
            let id = &data[at..at + 4];
            let len = u32::from_le_bytes(data[at + 4..at + 8].try_into().unwrap()) as usize;
            let body = &data[at + 8..(at + 8 + len).min(data.len())];
            if id == b"fmt " {
                channels = u16::from_le_bytes([body[2], body[3]]) as usize;
                rate = u32::from_le_bytes(body[4..8].try_into().unwrap());
            } else if id == b"data" {
                assert_eq!(rate as usize, RATE, "the WAV must be 16 kHz");
                return body
                    .chunks_exact(2 * channels)
                    .map(|frame| i16::from_le_bytes([frame[0], frame[1]]) as f32 / 32768.0)
                    .collect();
            }
            at += 8 + len + len % 2;
        }
        panic!("no data chunk");
    }

    fn environment() -> &'static Environment {
        let dll = std::env::var("SNAPR_ORT_DLL").map(PathBuf::from).unwrap_or_else(|_| {
            PathBuf::from(std::env::var("LOCALAPPDATA").unwrap()).join("com.quanvudev.snapr").join("paddle").join("onnxruntime.dll")
        });
        onnx::init(&onnx::Target { runtime: onnx::Runtime::Cpu, dll, device: 0 }).unwrap()
    }

    #[test]
    #[ignore]
    fn transcribes_wav() {
        let (Ok(dir), Ok(file)) = (std::env::var("SNAPR_CAPTIONS_MODEL"), std::env::var("SNAPR_CAPTIONS_WAV")) else { return };
        let started = Instant::now();
        let mut model = Model::load(environment(), Path::new(&dir), 6.5).unwrap();
        eprintln!("load {:?}", started.elapsed());
        let audio = wav(Path::new(&file));
        let mut line = model.line();
        let mut slowest = std::time::Duration::ZERO;
        for step in audio.chunks(STEP) {
            let started = Instant::now();
            model.feed(&mut line, step).unwrap();
            let text = model.text(&mut line).unwrap();
            slowest = slowest.max(started.elapsed());
            eprintln!("{:5.2}s {text}", line.seconds());
        }
        model.finish(&mut line).unwrap();
        let text = model.text(&mut line).unwrap();
        eprintln!("final: {text}\nslowest step {slowest:?} for {} ms of audio", STEP * 1000 / RATE);
        assert!(!text.is_empty());
    }
}
