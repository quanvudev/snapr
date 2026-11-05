use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use ort::environment::Environment;
use ort::session::{Session, SessionInputValue};
use ort::value::{DynValue, Tensor};
use serde_json::Value;

use super::{ALREADY_ENGLISH, Segment};
use super::mel::{Mel, N_FRAMES, N_SAMPLES, SAMPLE_RATE};
use super::tokens::Tokens;
use crate::queue::CANCELLED;

const MAX_WINDOW_TOKENS: usize = 224;
pub const WINDOW_SAMPLES: usize = N_SAMPLES;
const TIME_PRECISION_MS: u64 = 20;
const SAMPLES_PER_TIMESTAMP: usize = SAMPLE_RATE * TIME_PRECISION_MS as usize / 1000;
const AUTO: &str = "auto";
const ENGLISH: &str = "en";
const INPUT_IDS: &str = "input_ids";
const ENCODER_HIDDEN_STATES: &str = "encoder_hidden_states";
const USE_CACHE_BRANCH: &str = "use_cache_branch";
const PAST_PREFIX: &str = "past_key_values.";
const PRESENT_PREFIX: &str = "present.";
const ENCODER_CACHE: &str = ".encoder.";
const LOGITS: &str = "logits";

type Cache = HashMap<String, DynValue>;

struct Ids {
    start: i64,
    eos: i64,
    transcribe: i64,
    translate: Option<i64>,
    timestamp_begin: i64,
    languages: HashMap<String, i64>,
    suppress: Vec<i64>,
    begin_suppress: Vec<i64>,
}

struct Model {
    encoder: Session,
    decoder: Session,
    heads: usize,
    head_dim: usize,
    n_mels: usize,
}

fn err(e: impl ToString) -> String {
    e.to_string()
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(&std::fs::read_to_string(path).map_err(err)?).map_err(err)
}

fn int(json: &Value, key: &str) -> Result<i64, String> {
    json[key].as_i64().ok_or_else(|| format!("{key} missing"))
}

fn ints(json: &Value, key: &str) -> Vec<i64> {
    json[key].as_array().map(|a| a.iter().filter_map(Value::as_i64).collect()).unwrap_or_default()
}

impl Ids {
    fn load(json: &Value) -> Result<Self, String> {
        let no_timestamps = int(json, "no_timestamps_token_id")?;
        let languages = json["lang_to_id"]
            .as_object()
            .ok_or("lang_to_id missing")?
            .iter()
            .filter_map(|(k, v)| Some((k.clone(), v.as_i64()?)))
            .collect();
        Ok(Self {
            start: int(json, "decoder_start_token_id")?,
            eos: int(json, "eos_token_id")?,
            transcribe: json["task_to_id"]["transcribe"].as_i64().ok_or("task_to_id missing")?,
            translate: json["task_to_id"]["translate"].as_i64(),
            timestamp_begin: no_timestamps + 1,
            languages,
            suppress: ints(json, "suppress_tokens"),
            begin_suppress: ints(json, "begin_suppress_tokens"),
        })
    }

    fn language(&self, code: &str) -> Result<i64, String> {
        self.languages.get(&format!("<|{code}|>")).copied().ok_or_else(|| format!("unknown language {code}"))
    }

    fn code(&self, id: i64) -> Option<String> {
        self.languages
            .iter()
            .find(|&(_, &v)| v == id)
            .map(|(k, _)| k.trim_start_matches("<|").trim_end_matches("|>").to_owned())
    }

    fn task(&self, translate: bool) -> Result<i64, String> {
        if translate { self.translate.ok_or_else(|| "model cannot translate".into()) } else { Ok(self.transcribe) }
    }

    fn is_timestamp(&self, id: i64) -> bool {
        id >= self.timestamp_begin
    }
}

fn session(env: &Environment, path: &Path) -> Result<Session, String> {
    Session::builder(env).map_err(err)?.commit_from_file(path).map_err(err)
}

fn argmax(scores: &[f32]) -> usize {
    scores.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(i, _)| i)
}

fn mask(scores: &mut [f32], from: i64, to: i64) {
    let clamp = |i: i64| (i.max(0) as usize).min(scores.len());
    let (from, to) = (clamp(from), clamp(to));
    if from < to {
        scores[from..to].fill(f32::NEG_INFINITY);
    }
}

fn log_sum_exp(scores: &[f32]) -> f64 {
    let max = scores.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    if max == f32::NEG_INFINITY {
        return f64::NEG_INFINITY;
    }
    max as f64 + scores.iter().map(|&s| ((s - max) as f64).exp()).sum::<f64>().ln()
}

fn apply_rules(ids: &Ids, scores: &mut [f32], sampled: &[i64]) {
    let vocab = scores.len() as i64;
    for &id in &ids.suppress {
        mask(scores, id, id + 1);
    }
    mask(scores, ids.eos + 1, ids.timestamp_begin);
    if sampled.is_empty() {
        for &id in &ids.begin_suppress {
            mask(scores, id, id + 1);
        }
        mask(scores, 0, ids.timestamp_begin);
    }
    let last = sampled.last().is_some_and(|&t| ids.is_timestamp(t));
    let penultimate = sampled.len() < 2 || ids.is_timestamp(sampled[sampled.len() - 2]);
    if last {
        if penultimate {
            mask(scores, ids.timestamp_begin, vocab);
        } else {
            mask(scores, 0, ids.eos);
        }
    }
    if let Some(&latest) = sampled.iter().rev().find(|&&t| ids.is_timestamp(t)) {
        let bound = if last && !penultimate { latest } else { latest + 1 };
        mask(scores, ids.timestamp_begin, bound);
    }
    let split = (ids.timestamp_begin.max(0) as usize).min(scores.len());
    let timestamp = log_sum_exp(&scores[split..]);
    let best_text = scores[..split].iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;
    if timestamp > best_text {
        mask(scores, 0, ids.timestamp_begin);
    }
}

impl Model {
    fn load(env: &Environment, dir: &Path) -> Result<Self, String> {
        let config = read_json(&dir.join("config.json"))?;
        let preprocessor = read_json(&dir.join("preprocessor_config.json"))?;
        let heads = int(&config, "decoder_attention_heads")? as usize;
        Ok(Self {
            encoder: session(env, &dir.join("encoder.onnx"))?,
            decoder: session(env, &dir.join("decoder.onnx"))?,
            heads,
            head_dim: int(&config, "d_model")? as usize / heads.max(1),
            n_mels: int(&preprocessor, "feature_size")? as usize,
        })
    }

    fn encode(&mut self, features: Vec<f32>) -> Result<DynValue, String> {
        let input = Tensor::from_array(([1, self.n_mels, N_FRAMES], features)).map_err(err)?;
        let name = self.encoder.inputs().first().ok_or("encoder has no input")?.name().to_owned();
        let outputs = self.encoder.run(vec![(name, SessionInputValue::from(input))]).map_err(err)?;
        outputs.into_iter().next().map(|(_, v)| v).ok_or_else(|| "encoder has no output".into())
    }

    fn step(&mut self, ids: &[i64], hidden: &DynValue, cache: Option<&Cache>) -> Result<(Vec<f32>, Cache), String> {
        let empty = Tensor::from_array(([1, self.heads, 0, self.head_dim], Vec::<f32>::new())).map_err(err)?;
        let names: Vec<String> = self.decoder.inputs().iter().map(|i| i.name().to_owned()).collect();
        let mut inputs: Vec<(String, SessionInputValue)> = Vec::with_capacity(names.len());
        for name in names {
            let value: SessionInputValue = match name.as_str() {
                INPUT_IDS => Tensor::from_array(([1, ids.len()], ids.to_vec())).map_err(err)?.into(),
                ENCODER_HIDDEN_STATES => hidden.into(),
                USE_CACHE_BRANCH => Tensor::from_array(([1usize], vec![cache.is_some()])).map_err(err)?.into(),
                past if past.starts_with(PAST_PREFIX) => {
                    let present = format!("{PRESENT_PREFIX}{}", &past[PAST_PREFIX.len()..]);
                    match cache.and_then(|c| c.get(&present)) {
                        Some(value) => value.into(),
                        None => (&empty).into(),
                    }
                }
                other => return Err(format!("unexpected decoder input {other}")),
            };
            inputs.push((name, value));
        }
        let outputs = self.decoder.run(inputs).map_err(err)?;
        let mut logits = Vec::new();
        let mut present = Cache::new();
        for (name, value) in outputs {
            if name == LOGITS {
                let (shape, data) = value.try_extract_tensor::<f32>().map_err(err)?;
                let vocab = *shape.last().ok_or("logits without shape")? as usize;
                logits = data[data.len() - vocab..].to_vec();
            } else if name.starts_with(PRESENT_PREFIX) {
                present.insert(name.to_owned(), value);
            }
        }
        Ok((logits, present))
    }

    fn detect(&mut self, ids: &Ids, hidden: &DynValue) -> Result<i64, String> {
        let (logits, _) = self.step(&[ids.start], hidden, None)?;
        ids.languages
            .values()
            .copied()
            .filter(|&id| (id as usize) < logits.len())
            .max_by(|&a, &b| logits[a as usize].total_cmp(&logits[b as usize]))
            .ok_or_else(|| "no language token".into())
    }

    fn generate(&mut self, ids: &Ids, hidden: &DynValue, language: i64, task: i64, cancel: &AtomicBool) -> Result<Vec<i64>, String> {
        let mut sampled = Vec::new();
        let (mut logits, mut cache) = self.step(&[ids.start, language, task], hidden, None)?;
        while sampled.len() < MAX_WINDOW_TOKENS {
            if cancel.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            apply_rules(ids, &mut logits, &sampled);
            let token = argmax(&logits) as i64;
            if token == ids.eos {
                break;
            }
            sampled.push(token);
            let (next_logits, mut next) = self.step(&[token], hidden, Some(&cache))?;
            next.extend(cache.into_iter().filter(|(name, _)| name.contains(ENCODER_CACHE)));
            (logits, cache) = (next_logits, next);
        }
        Ok(sampled)
    }
}

fn segments(ids: &Ids, tokens: &Tokens, sampled: &[i64], offset: usize, window: usize, out: &mut Vec<Segment>) -> usize {
    let ms = |samples: usize| (samples * 1000 / SAMPLE_RATE) as u64;
    let time = |t: i64| ms(offset) + (t - ids.timestamp_begin) as u64 * TIME_PRECISION_MS;
    let mut push = |start: u64, end: u64, text: &[i64]| {
        let text: Vec<i64> = text.iter().copied().filter(|&t| t < ids.eos).collect();
        let text = tokens.decode(&text).trim().to_owned();
        if !text.is_empty() {
            out.push(Segment { start_ms: start, end_ms: end.max(start), text });
        }
    };
    let ts = |i: usize| ids.is_timestamp(sampled[i]);
    let single_ending = sampled.len() >= 2 && !ts(sampled.len() - 2) && ts(sampled.len() - 1);
    let mut cuts: Vec<usize> = (1..sampled.len()).filter(|&i| ts(i) && ts(i - 1)).collect();
    if cuts.is_empty() {
        let end = sampled
            .iter()
            .rev()
            .find(|&&t| ids.is_timestamp(t) && t != ids.timestamp_begin)
            .map_or(ms(offset + window), |&t| time(t));
        push(ms(offset), end, sampled);
        return window;
    }
    if single_ending {
        cuts.push(sampled.len());
    }
    let mut last = 0;
    for &cut in &cuts {
        let slice = &sampled[last..cut];
        push(time(slice[0]), time(slice[slice.len() - 1]), slice);
        last = cut;
    }
    if single_ending {
        return window;
    }
    let advance = (sampled[last - 1] - ids.timestamp_begin) as usize * SAMPLES_PER_TIMESTAMP;
    if advance == 0 { window } else { advance.min(window) }
}

pub fn transcribe(
    model_dir: &Path,
    language: &str,
    translate: bool,
    pcm: &[f32],
    progress: &dyn Fn(f32),
    cancel: &AtomicBool,
) -> Result<(Vec<Segment>, Option<String>), String> {
    let ids = Ids::load(&read_json(&model_dir.join("generation_config.json"))?)?;
    let task = ids.task(translate)?;
    let tokens = Tokens::load(&model_dir.join("vocab.json"))?;
    let mut model = Model::load(crate::onnx::environment()?, model_dir)?;
    let mel = Mel::new(model.n_mels);
    let mut language = (language != AUTO).then(|| ids.language(language)).transpose()?;
    let mut out = Vec::new();
    let mut offset = 0;
    while offset < pcm.len() {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let window = &pcm[offset..(offset + N_SAMPLES).min(pcm.len())];
        let hidden = model.encode(mel.features(window))?;
        let lang = match language {
            Some(lang) => lang,
            None => *language.insert(model.detect(&ids, &hidden)?),
        };
        if translate && ids.code(lang).as_deref() == Some(ENGLISH) {
            return Err(ALREADY_ENGLISH.into());
        }
        let sampled = model.generate(&ids, &hidden, lang, task, cancel)?;
        offset += segments(&ids, &tokens, &sampled, offset, window.len(), &mut out);
        progress((offset as f32 / pcm.len() as f32).min(1.0));
    }
    Ok((out, language.and_then(|id| ids.code(id))))
}

pub fn detect(model_dir: &Path, pcm: &[f32]) -> Result<String, String> {
    let ids = Ids::load(&read_json(&model_dir.join("generation_config.json"))?)?;
    let mut model = Model::load(crate::onnx::environment()?, model_dir)?;
    let mel = Mel::new(model.n_mels);
    let window = &pcm[..N_SAMPLES.min(pcm.len())];
    let hidden = model.encode(mel.features(window))?;
    let lang = model.detect(&ids, &hidden)?;
    ids.code(lang).ok_or_else(|| "no language token".into())
}
