use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use tauri::{AppHandle, Manager};

use super::engine::{Engine, batches};
use super::languages::{default_target, find};
use super::{acquire, model_dir, prompt, ready};
use crate::queue::{self, Progress, Task};
use crate::subtitle::NO_TRANSCRIPT;
use crate::transcript::{self, ENGLISH_TRACK, MODEL_MISSING, SRT_EXT, Segment, TranscribeTask, Track};
use crate::{onnx, video};

pub const SAME_LANGUAGE: &str = "SAME_LANGUAGE";
pub const UNSUPPORTED_LANGUAGE: &str = "UNSUPPORTED_LANGUAGE";
const KIND: &str = "translate";
const ENGLISH: &str = "en";
const BATCH: usize = 8;

pub struct HymtTask {
    path: PathBuf,
    out_dir: PathBuf,
    source: String,
    target: &'static str,
    segments: Vec<Segment>,
    model_dir: PathBuf,
    runtime: onnx::Target,
}

pub fn task(app: &AppHandle, path: &str, target: Option<&str>) -> Result<Box<dyn Task>, String> {
    if ready(app) {
        return Ok(Box::new(HymtTask::validated(app, path, target)?));
    }
    match target {
        None | Some(ENGLISH) => Ok(Box::new(TranscribeTask::validated(app, path, ENGLISH_TRACK)?)),
        Some(_) => Err(MODEL_MISSING.into()),
    }
}

impl HymtTask {
    pub fn validated(app: &AppHandle, path: &str, target: Option<&str>) -> Result<HymtTask, String> {
        let path = video::guarded_mp4(app, path)?;
        let out_dir = path.parent().ok_or("no output folder")?.to_path_buf();
        let text = std::fs::read_to_string(Track::Original.path(&path)).map_err(|_| NO_TRANSCRIPT.to_string())?;
        let segments = transcript::parse_srt(&text);
        if !segments.iter().any(|segment| !segment.text.trim().is_empty()) {
            return Err(NO_TRANSCRIPT.into());
        }
        let source = transcript::languages::get(app, &path).unwrap_or_default();
        let ui = app.state::<crate::State>().cfg.read().unwrap().language.clone();
        let target = match target {
            Some(code) => find(code).ok_or(UNSUPPORTED_LANGUAGE)?.code,
            None => default_target(&source, &ui),
        };
        if target.eq_ignore_ascii_case(&source) {
            return Err(SAME_LANGUAGE.into());
        }
        Ok(HymtTask {
            path,
            out_dir,
            source,
            target,
            segments,
            model_dir: model_dir(app),
            runtime: onnx::target(app),
        })
    }
}

impl HymtTask {
    fn translate(
        &self,
        env: &'static ort::environment::Environment,
        cues: &[usize],
        progress: &Progress,
        cancel: &AtomicBool,
    ) -> Result<Vec<Segment>, String> {
        let total = cues.len() as u64;
        progress.set(0, total);
        let mut engine = Engine::load(env, &self.model_dir)?;
        let mut out: Vec<Segment> = self
            .segments
            .iter()
            .map(|segment| Segment { start_ms: segment.start_ms, end_ms: segment.end_ms, text: String::new() })
            .collect();
        let requests: Vec<(String, usize)> = cues
            .iter()
            .map(|&i| {
                let text = self.segments[i].text.trim();
                (prompt::build(&self.source, self.target, text), engine.tokens(text))
            })
            .collect();
        let tokens: Vec<usize> = requests.iter().map(|(_, n)| *n).collect();
        let mut finished = 0u64;
        for group in batches(&tokens, BATCH) {
            let batch: Vec<(&str, usize)> = group.iter().map(|&k| (requests[k].0.as_str(), requests[k].1)).collect();
            let replies = engine.complete_batch(&batch, cancel)?;
            for (&k, reply) in group.iter().zip(replies) {
                out[cues[k]].text = prompt::clean(&reply, self.target);
            }
            finished += group.len() as u64;
            progress.set(finished, total);
        }
        Ok(out)
    }
}

impl Task for HymtTask {
    fn kind(&self) -> &'static str {
        KIND
    }

    fn name(&self) -> String {
        self.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn source(&self) -> Option<&Path> {
        Some(&self.path)
    }

    fn detail(&self) -> Option<String> {
        Some(self.target.to_owned())
    }

    fn run(&self, progress: &Progress, cancel: &AtomicBool) -> Result<Option<PathBuf>, String> {
        if !self.model_dir.is_dir() {
            return Err(MODEL_MISSING.into());
        }
        let _in_use = acquire();
        let cues: Vec<usize> = (0..self.segments.len()).filter(|&i| !self.segments[i].text.trim().is_empty()).collect();
        let env = onnx::init(&self.runtime)?;
        let out = onnx::retry_on_cpu(onnx::Workload::Translate, || self.translate(env, &cues, progress, cancel))?;
        let track = Track::Translation(self.target);
        queue::write_part_to(&self.out_dir, &track.stem(&self.path), SRT_EXT, |part| transcript::write_srt(part, &out), || {
            track.path(&self.path)
        })
    }
}
