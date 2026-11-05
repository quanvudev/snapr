use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use ort::environment::Environment;
use ort::session::{Session, SessionInputValue};
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

pub struct Engine {
    session: Session,
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
        let session = Session::builder(env).map_err(err)?.commit_from_file(dir.join(MODEL_FILE)).map_err(err)?;
        let inputs = session.inputs().iter().map(|i| i.name().to_owned()).collect();
        Ok(Self {
            session,
            tokenizer,
            kv_heads: int(&config, "num_key_value_heads")?,
            head_dim: int(&config, "head_dim")?,
            inputs,
        })
    }

    fn step(&mut self, ids: &[i64], start: usize, cache: Option<&Cache>) -> Result<(Vec<f32>, Cache), String> {
        let total = start + ids.len();
        let empty = Tensor::from_array(([1, self.kv_heads, 0, self.head_dim], Vec::<f32>::new())).map_err(err)?;
        let mut feed: Vec<(String, SessionInputValue)> = Vec::with_capacity(self.inputs.len());
        for name in &self.inputs {
            let value: SessionInputValue = match name.as_str() {
                INPUT_IDS => Tensor::from_array(([1, ids.len()], ids.to_vec())).map_err(err)?.into(),
                ATTENTION_MASK => Tensor::from_array(([1, total], vec![1i64; total])).map_err(err)?.into(),
                POSITION_IDS => {
                    let positions: Vec<i64> = (start..total).map(|p| p as i64).collect();
                    Tensor::from_array(([1, ids.len()], positions)).map_err(err)?.into()
                }
                past if past.starts_with(PAST_PREFIX) => {
                    let present = format!("{PRESENT_PREFIX}{}", &past[PAST_PREFIX.len()..]);
                    match cache.and_then(|c| c.get(&present)) {
                        Some(value) => value.into(),
                        None => (&empty).into(),
                    }
                }
                other => return Err(format!("unexpected input {other}")),
            };
            feed.push((name.clone(), value));
        }
        let outputs = self.session.run(feed).map_err(err)?;
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

    fn encode(&self, text: &str) -> Result<Vec<i64>, String> {
        let encoding = self.tokenizer.encode(text, false).map_err(err)?;
        Ok(encoding.get_ids().iter().map(|&id| id as i64).collect())
    }

    pub fn complete(&mut self, prompt: &str, source_tokens: usize, cancel: &AtomicBool) -> Result<String, String> {
        let mut ids = vec![BOS, USER];
        ids.extend(self.encode(prompt)?);
        ids.push(ASSISTANT);
        let limit = (source_tokens * OUTPUT_PER_INPUT).max(MIN_OUTPUT_TOKENS);
        let mut seen: HashSet<i64> = ids.iter().copied().collect();
        let mut generated: Vec<u32> = Vec::new();
        let (mut logits, mut cache) = self.step(&ids, 0, None)?;
        let mut position = ids.len();
        while generated.len() < limit {
            if cancel.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            for &id in &seen {
                if let Some(score) = logits.get_mut(id as usize) {
                    *score = if *score > 0.0 { *score / REPETITION_PENALTY } else { *score * REPETITION_PENALTY };
                }
            }
            let token = logits.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(EOS, |(i, _)| i as i64);
            if token == EOS {
                break;
            }
            generated.push(token as u32);
            seen.insert(token);
            (logits, cache) = self.step(&[token], position, Some(&cache))?;
            position += 1;
        }
        self.tokenizer.decode(&generated, true).map_err(err)
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

    #[test]
    #[ignore]
    fn translates_one_sentence() {
        let (Ok(dir), Ok(runtime)) = (std::env::var(DIR_ENV), std::env::var(RUNTIME_ENV)) else { return };
        let env = crate::onnx::init(Path::new(&runtime)).unwrap();
        let mut engine = Engine::load(env, Path::new(&dir)).unwrap();
        let prompt = crate::translate::prompt::build("vi", "en", "Xin chào, hôm nay trời rất đẹp.");
        let source = engine.tokens("Xin chào, hôm nay trời rất đẹp.");
        let started = std::time::Instant::now();
        let text = engine.complete(&prompt, source, &AtomicBool::new(false)).unwrap();
        eprintln!("{text:?} in {:?}", started.elapsed());
        assert!(!text.trim().is_empty());
    }
}
