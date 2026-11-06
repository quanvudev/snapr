use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use ort::environment::Environment;
use crate::onnx::{Feed, Runner, Workload};
use ort::value::{DynValue, Tensor};
use serde_json::Value;
use tokenizers::Tokenizer;

use crate::queue::CANCELLED;

pub const MODEL_FILE: &str = "model_q4.onnx";
pub const TOKENIZER_FILE: &str = "tokenizer.json";
pub const CONFIG_FILE: &str = "config.json";
const BOS: i64 = 120_000;
const USER: i64 = 120_006;
const ASSISTANT: i64 = 120_007;
const EOS: i64 = 120_020;
const PAD: i64 = 120_002;
const REPETITION_PENALTY: f32 = 1.05;
const MIN_OUTPUT_TOKENS: usize = 64;
const OUTPUT_PER_INPUT: usize = 4;
const INPUT_IDS: &str = "input_ids";
const ATTENTION_MASK: &str = "attention_mask";
const POSITION_IDS: &str = "position_ids";
const PAST_PREFIX: &str = "past_key_values.";
const PRESENT_PREFIX: &str = "present.";
const LOGITS: &str = "logits";

type Cache = HashMap<String, DynValue>;

fn err(e: impl ToString) -> String {
    e.to_string()
}

fn int(json: &Value, key: &str) -> Result<usize, String> {
    json[key].as_u64().map(|v| v as usize).ok_or_else(|| format!("{key} missing"))
}

struct Batch {
    ids: Vec<i64>,
    mask: Vec<i64>,
    positions: Vec<i64>,
}

impl Batch {
    fn prefill(prompts: &[Vec<i64>]) -> Self {
        let width = prompts.iter().map(Vec::len).max().unwrap_or(0);
        let mut batch = Batch { ids: Vec::new(), mask: Vec::new(), positions: Vec::new() };
        for prompt in prompts {
            for at in 0..width {
                let real = at < prompt.len();
                batch.ids.push(if real { prompt[at] } else { PAD });
                batch.mask.push(real as i64);
                batch.positions.push(at.min(prompt.len() - 1) as i64);
            }
        }
        batch
    }

    fn decode(tokens: &[i64], lengths: &[usize], total: usize) -> Self {
        let mut batch = Batch { ids: tokens.to_vec(), mask: Vec::new(), positions: Vec::new() };
        for &length in lengths {
            batch.mask.extend((0..total).map(|at| (at <= length) as i64));
            batch.positions.push(length as i64);
        }
        batch
    }
}

pub fn batches(tokens: &[usize], size: usize) -> Vec<Vec<usize>> {
    let mut order: Vec<usize> = (0..tokens.len()).collect();
    order.sort_by_key(|&i| tokens[i]);
    order.chunks(size.max(1)).map(<[usize]>::to_vec).collect()
}

fn pick(scores: &mut [f32], seen: &HashSet<i64>) -> i64 {
    for &id in seen {
        if let Some(score) = scores.get_mut(id as usize) {
            *score = if *score > 0.0 { *score / REPETITION_PENALTY } else { *score * REPETITION_PENALTY };
        }
    }
    scores.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(EOS, |(i, _)| i as i64)
}

pub struct Engine {
    session: Runner,
    tokenizer: Tokenizer,
    kv_heads: usize,
    head_dim: usize,
    inputs: Vec<String>,
}

impl Engine {
    pub fn load(env: &Environment, dir: &Path) -> Result<Self, String> {
        let config: Value =
            serde_json::from_str(&std::fs::read_to_string(dir.join(CONFIG_FILE)).map_err(err)?).map_err(err)?;
        let tokenizer = Tokenizer::from_file(dir.join(TOKENIZER_FILE)).map_err(err)?;
        let session = crate::onnx::runner(env, &dir.join(MODEL_FILE), Workload::Translate)?;
        let inputs = session.input_names();
        Ok(Self {
            session,
            tokenizer,
            kv_heads: int(&config, "num_key_value_heads")?,
            head_dim: int(&config, "head_dim")?,
            inputs,
        })
    }

    fn step(&mut self, batch: &Batch, cache: Option<&Cache>, pick: &[usize]) -> Result<(Vec<Vec<f32>>, Cache), String> {
        let rows = pick.len();
        let width = batch.ids.len() / rows;
        let total = batch.mask.len() / rows;
        let empty = Tensor::from_array(([rows, self.kv_heads, 0, self.head_dim], Vec::<f32>::new())).map_err(err)?.into_dyn();
        let mut feed: Vec<(String, Feed)> = Vec::with_capacity(self.inputs.len());
        for name in &self.inputs {
            let value = match name.as_str() {
                INPUT_IDS => Feed::Owned(Tensor::from_array(([rows, width], batch.ids.clone())).map_err(err)?.into_dyn()),
                ATTENTION_MASK => Feed::Owned(Tensor::from_array(([rows, total], batch.mask.clone())).map_err(err)?.into_dyn()),
                POSITION_IDS => Feed::Owned(Tensor::from_array(([rows, width], batch.positions.clone())).map_err(err)?.into_dyn()),
                past if past.starts_with(PAST_PREFIX) => {
                    let present = format!("{PRESENT_PREFIX}{}", &past[PAST_PREFIX.len()..]);
                    match cache.and_then(|c| c.get(&present)) {
                        Some(value) => Feed::Ref(value),
                        None => Feed::Ref(&empty),
                    }
                }
                other => return Err(format!("unexpected input {other}")),
            };
            feed.push((name.clone(), value));
        }
        let outputs = self.session.run(feed, &[LOGITS])?;
        let mut logits = Vec::new();
        let mut present = Cache::new();
        for (name, value) in outputs {
            if name == LOGITS {
                let (shape, data) = value.try_extract_tensor::<f32>().map_err(err)?;
                let vocab = *shape.last().ok_or("logits without shape")? as usize;
                let seq = data.len() / (rows * vocab);
                logits = pick
                    .iter()
                    .enumerate()
                    .map(|(row, &at)| {
                        let from = (row * seq + at.min(seq - 1)) * vocab;
                        data[from..from + vocab].to_vec()
                    })
                    .collect();
            } else if name.starts_with(PRESENT_PREFIX) {
                present.insert(name.to_owned(), value);
            }
        }
        Ok((logits, present))
    }

    fn encode(&self, text: &str) -> Result<Vec<i64>, String> {
        let encoding = self.tokenizer.encode(text, false).map_err(err)?;
        Ok(encoding.get_ids().iter().map(|&id| id as i64).collect())
    }

    pub fn complete_batch(&mut self, requests: &[(&str, usize)], cancel: &AtomicBool) -> Result<Vec<String>, String> {
        if requests.is_empty() {
            return Ok(Vec::new());
        }
        let mut prompts = Vec::with_capacity(requests.len());
        for (prompt, _) in requests {
            let mut ids = vec![BOS, USER];
            ids.extend(self.encode(prompt)?);
            ids.push(ASSISTANT);
            prompts.push(ids);
        }
        let limits: Vec<usize> =
            requests.iter().map(|&(_, source)| (source * OUTPUT_PER_INPUT).max(MIN_OUTPUT_TOKENS)).collect();
        let mut seen: Vec<HashSet<i64>> = prompts.iter().map(|ids| ids.iter().copied().collect()).collect();
        let mut generated: Vec<Vec<u32>> = vec![Vec::new(); prompts.len()];
        let mut done = vec![false; prompts.len()];
        let mut lengths: Vec<usize> = prompts.iter().map(Vec::len).collect();
        let prefill = Batch::prefill(&prompts);
        let last: Vec<usize> = lengths.iter().map(|&len| len - 1).collect();
        let (mut logits, mut cache) = self.step(&prefill, None, &last)?;
        let mut total = prefill.mask.len() / prompts.len();
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            let mut next = Vec::with_capacity(prompts.len());
            for (row, scores) in logits.iter_mut().enumerate() {
                if !done[row] && generated[row].len() >= limits[row] {
                    done[row] = true;
                }
                if done[row] {
                    next.push(EOS);
                    continue;
                }
                let token = pick(scores, &seen[row]);
                if token == EOS {
                    done[row] = true;
                } else {
                    generated[row].push(token as u32);
                    seen[row].insert(token);
                }
                next.push(token);
            }
            if done.iter().all(|&d| d) {
                break;
            }
            total += 1;
            let decode = Batch::decode(&next, &lengths, total);
            for length in &mut lengths {
                *length += 1;
            }
            (logits, cache) = self.step(&decode, Some(&cache), &vec![0; prompts.len()])?;
        }
        generated.iter().map(|tokens| self.tokenizer.decode(tokens, true).map_err(err)).collect()
    }

    pub fn tokens(&self, text: &str) -> usize {
        self.encode(text).map_or(0, |ids| ids.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIR_ENV: &str = "SNAPR_HYMT_DIR";
    const RUNTIME_ENV: &str = "SNAPR_ORT_DLL";
    const CUES: [&str; 20] = [
        "Xin chào mọi người.",
        "Hôm nay chúng ta sẽ nói về cách tối ưu hiệu năng của ứng dụng.",
        "Được.",
        "Bạn có thể bật tăng tốc phần cứng trong phần cài đặt mô hình AI.",
        "Cảm ơn bạn đã xem video này, hẹn gặp lại lần sau nhé!",
        "Không.",
        "Đầu tiên, hãy mở cửa sổ cài đặt từ khay hệ thống.",
        "Sau đó chọn mục Ghi màn hình ở thanh bên trái.",
        "Tôi nghĩ cách này nhanh hơn nhiều so với trước đây.",
        "Bạn có câu hỏi nào không?",
        "Chắc chắn rồi, tôi sẽ gửi báo cáo cho bạn vào chiều nay.",
        "Phần mềm này chạy hoàn toàn trên máy tính của bạn, không cần mạng.",
        "Hãy thử lại một lần nữa.",
        "Nếu card đồ hoạ của bạn là NVIDIA, hãy chọn CUDA để có tốc độ cao nhất.",
        "Ồ, thú vị thật!",
        "Video này dài khoảng năm phút.",
        "Chúng tôi đã đo trên một máy có tám nhân và mười sáu luồng.",
        "Kết quả cho thấy dịch theo lô nhanh hơn gấp đôi.",
        "Tạm biệt.",
        "Đừng quên bấm theo dõi kênh để xem các video tiếp theo.",
    ];

    fn engine() -> Option<Engine> {
        let (Ok(dir), Ok(runtime)) = (std::env::var(DIR_ENV), std::env::var(RUNTIME_ENV)) else { return None };
        let kind = match std::env::var("SNAPR_ORT_EP").as_deref() {
            Ok("cuda") => crate::onnx::Runtime::Cuda,
            Ok("directml") => crate::onnx::Runtime::DirectMl,
            _ => crate::onnx::Runtime::Cpu,
        };
        let target = crate::onnx::Target { runtime: kind, dll: runtime.into(), device: 0 };
        let env = crate::onnx::init(&target).unwrap();
        Some(Engine::load(env, Path::new(&dir)).unwrap())
    }

    fn requests(engine: &Engine, cues: &[&str]) -> Vec<(String, usize)> {
        cues.iter().map(|cue| (crate::translate::prompt::build("vi", "en", cue), engine.tokens(cue))).collect()
    }

    fn translate(engine: &mut Engine, requests: &[(String, usize)], batch: usize) -> Vec<String> {
        let cancel = AtomicBool::new(false);
        let mut out = vec![String::new(); requests.len()];
        let tokens: Vec<usize> = requests.iter().map(|(_, n)| *n).collect();
        for group in batches(&tokens, batch) {
            let chunk: Vec<(&str, usize)> = group.iter().map(|&i| (requests[i].0.as_str(), requests[i].1)).collect();
            for (i, text) in group.iter().zip(engine.complete_batch(&chunk, &cancel).unwrap()) {
                out[*i] = text;
            }
        }
        out
    }

    #[test]
    #[ignore]
    fn batches_match_single_cues() {
        let Some(mut engine) = engine() else { return };
        let requests = requests(&engine, &CUES);
        let started = std::time::Instant::now();
        let single = translate(&mut engine, &requests, 1);
        let single_time = started.elapsed();
        let started = std::time::Instant::now();
        let batched = translate(&mut engine, &requests, 8);
        let batched_time = started.elapsed();
        let mut same = 0;
        for ((cue, a), b) in CUES.iter().zip(&single).zip(&batched) {
            eprintln!("{} {cue:?}
    single:  {a:?}
    batched: {b:?}", if a == b { "SAME" } else { "DIFF" });
            same += (a == b) as usize;
        }
        eprintln!("{same}/20 identical, single {single_time:?}, batched {batched_time:?}");
        assert!(same >= 19);
        for (index, text) in batched.iter().enumerate() {
            for (other, cue) in single.iter().enumerate() {
                if other != index && cue.len() > 12 {
                    assert!(!text.contains(cue.as_str()), "cue {index} contains cue {other}");
                }
            }
        }
    }

    #[test]
    #[ignore]
    fn batched_hundred_cues_timing() {
        let Some(mut engine) = engine() else { return };
        let cues: Vec<&str> = CUES.iter().cycle().take(100).copied().collect();
        let requests = requests(&engine, &cues);
        for batch in [1, 8] {
            let started = std::time::Instant::now();
            translate(&mut engine, &requests, batch);
            eprintln!("100 cues at batch {batch}: {:?}", started.elapsed());
        }
    }
}
