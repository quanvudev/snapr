use super::segment::{Resampler, Segment, Segmenter};
use super::stream::{Line, Model, RATE};
use super::{Language, acquire, model_dir, ready};
use crate::translate::engine::Engine;
use crate::translate::prompt;
use crate::{State, audio, events, onnx, panel, translate};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};

pub const MODEL_MISSING: &str = "MODEL_MISSING";
pub const NO_AUDIO: &str = "NO_AUDIO";
const POLL: Duration = Duration::from_millis(100);
const PARTIAL_SAMPLES: usize = RATE * 160 / 1000;
const MAX_BATCH: usize = 8;
const FLUSH_SAMPLES: usize = RATE / 2;
const MAX_SENTENCE_LINES: usize = 4;
const MAX_SENTENCE_CHARS: usize = 240;
const SENTENCE_ENDS: &[char] = &['.', '?', '!', '…', '。', '？', '！'];
const SWITCH_TICKS: usize = 40;
const MIN_TURN_TICKS: usize = 50;
const SWITCH_RATIO: f32 = 4.0;
const MIN_ENERGY: f32 = 1e-4;

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum Source {
    Mic,
    System,
}

fn louder(mic: f32, system: f32) -> Source {
    if system > mic { Source::System } else { Source::Mic }
}

struct Turn {
    first: Source,
    mic: f32,
    system: f32,
    ticks: usize,
    recent: VecDeque<(f32, f32)>,
}

impl Turn {
    fn new(first: Source) -> Self {
        Turn { first, mic: 0.0, system: 0.0, ticks: 0, recent: VecDeque::with_capacity(SWITCH_TICKS + 1) }
    }

    fn owner(&self) -> Source {
        if self.mic + self.system > 0.0 { louder(self.mic, self.system) } else { self.first }
    }

    fn switched(&mut self, mic: f32, system: f32) -> bool {
        self.mic += mic;
        self.system += system;
        self.ticks += 1;
        self.recent.push_back((mic, system));
        if self.recent.len() > SWITCH_TICKS {
            self.recent.pop_front();
        }
        if self.ticks < MIN_TURN_TICKS || self.recent.len() < SWITCH_TICKS {
            return false;
        }
        let (m, s) = self.recent.iter().fold((0.0, 0.0), |(a, b), (m, s)| (a + m, b + s));
        let (other, own) = match self.owner() {
            Source::Mic => (s, m),
            Source::System => (m, s),
        };
        other / SWITCH_TICKS as f32 > MIN_ENERGY && other > own * SWITCH_RATIO
    }
}

struct Sentence {
    id: u64,
    text: String,
    lines: usize,
    source: Source,
}

fn join(prefix: Option<&Sentence>, text: &str, spaced: bool) -> String {
    match prefix {
        Some(open) if !open.text.is_empty() && !text.is_empty() => {
            if spaced { format!("{} {text}", open.text) } else { format!("{}{text}", open.text) }
        }
        Some(open) if text.is_empty() => open.text.clone(),
        _ => text.to_string(),
    }
}

fn complete(text: &str, lines: usize) -> bool {
    text.trim_end_matches(['"', '\'', '”', '’', ')']).ends_with(SENTENCE_ENDS)
        || lines >= MAX_SENTENCE_LINES
        || text.chars().count() >= MAX_SENTENCE_CHARS
}

pub struct Session {
    capture: Option<audio::Capture>,
    stop: Arc<AtomicBool>,
    threads: Vec<JoinHandle<()>>,
}

#[derive(Clone, Serialize)]
struct Caption {
    id: u64,
    text: String,
    #[serde(rename = "final")]
    done: bool,
    source: Source,
}

#[derive(Clone, Serialize)]
struct Translation {
    id: u64,
    translation: String,
}

fn log(text: &str) {
    if cfg!(debug_assertions) && std::env::var_os("SNAPR_CAPTIONS_LOG").is_some() {
        eprintln!("captions: {text}");
    }
}

fn emit(app: &AppHandle, payload: impl Serialize + Clone) {
    let _ = app.emit_to(panel::CAPTIONS, events::LIVE_CAPTION, payload);
}

pub fn running(app: &AppHandle) -> bool {
    app.state::<State>().captions.lock().unwrap().is_some()
}

pub fn start(app: &AppHandle, language: Language, target: Option<&'static str>) -> Result<(), String> {
    if running(app) {
        return Ok(());
    }
    if !ready(app, language) {
        return Err(MODEL_MISSING.into());
    }
    let mut capture = panel::audio(app).filter(audio::Capture::has_audio).ok_or(NO_AUDIO)?;
    let chunks = capture.chunks().ok_or(NO_AUDIO)?;
    let runtime = onnx::target(app);
    let stop = Arc::new(AtomicBool::new(false));
    let translate = target.filter(|t| *t != language.code() && translate::ready(app));
    let (lines, pending) = mpsc::channel();
    let mut threads = Vec::new();
    let (dir, handle, flag) = (model_dir(app, language), app.clone(), stop.clone());
    let to_translator = translate.map(|_| lines);
    let runtime_copy = runtime.clone();
    let spaced = !matches!(language, Language::Zh | Language::Ja);
    threads.push(std::thread::spawn(move || {
        let _in_use = acquire(language);
        let result = onnx::init(&runtime_copy)
            .and_then(|env| onnx::on_cpu(|| Model::load(env, &dir, language.tokens_per_second())))
            .and_then(|model| recognize(&handle, model, chunks, to_translator, spaced, &flag));
        if let Err(e) = result {
            log(&e);
            crate::tray::tooltip(&handle, &format!("Snapr: live captions failed: {e}"));
            let app = handle.clone();
            let _ = handle.run_on_main_thread(move || stop_session(&app));
        }
    }));
    if let Some(target) = translate {
        let (dir, handle, flag) = (translate::model_dir(app), app.clone(), stop.clone());
        let source = language.code();
        threads.push(std::thread::spawn(move || {
            let _in_use = translate::acquire();
            let result = onnx::init(&runtime)
                .and_then(|env| Engine::load(env, &dir))
                .and_then(|engine| translate_lines(&handle, engine, pending, source, target, &flag));
            if let Err(e) = result {
                log(&e);
            }
        }));
    }
    *app.state::<State>().captions.lock().unwrap() = Some(Session { capture: Some(capture), stop, threads });
    Ok(())
}

fn recognize(
    app: &AppHandle,
    model: Model,
    chunks: Receiver<audio::Chunk>,
    lines: Option<Sender<(u64, String)>>,
    spaced: bool,
    stop: &AtomicBool,
) -> Result<(), String> {
    let mut recognizer = Recognizer {
        app,
        model,
        lines,
        spaced,
        resampler: Resampler::default(),
        segmenter: Segmenter::default(),
        samples: Vec::new(),
        line: None,
        open: None,
        next: 0,
        since: 0,
        quiet: 0,
        shown: String::new(),
    };
    while !stop.load(Ordering::Relaxed) {
        match chunks.recv_timeout(POLL) {
            Ok(chunk) => recognizer.chunk(&chunk)?,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        }
        while let Ok(chunk) = chunks.try_recv() {
            recognizer.chunk(&chunk)?;
        }
        recognizer.partial()?;
    }
    Ok(())
}

struct Recognizer<'a> {
    app: &'a AppHandle,
    model: Model,
    lines: Option<Sender<(u64, String)>>,
    spaced: bool,
    resampler: Resampler,
    segmenter: Segmenter,
    samples: Vec<f32>,
    line: Option<(u64, Line, Turn)>,
    open: Option<Sentence>,
    next: u64,
    since: usize,
    quiet: usize,
    shown: String,
}

impl Recognizer<'_> {
    fn chunk(&mut self, chunk: &audio::Chunk) -> Result<(), String> {
        self.samples.clear();
        self.resampler.push(&chunk.pcm, &mut self.samples);
        let received = self.samples.len();
        for segment in self.segmenter.push(&self.samples) {
            match segment {
                Segment::Speech(audio) => self.speech(&audio, louder(chunk.mic, chunk.system))?,
                Segment::End => self.end()?,
            }
        }
        if let Some((_, _, turn)) = self.line.as_mut()
            && turn.switched(chunk.mic, chunk.system)
            && self.segmenter.cut().is_some()
        {
            self.end()?;
        }
        if self.line.is_none() && self.open.is_some() {
            self.quiet += received;
            if self.quiet >= FLUSH_SAMPLES
                && let Some(sentence) = self.open.take()
            {
                self.close(sentence);
            }
        }
        Ok(())
    }

    fn speech(&mut self, audio: &[f32], start: Source) -> Result<(), String> {
        if self.line.is_none() {
            if let Some(open) = self.open.take_if(|o| o.source != start) {
                self.close(open);
            }
            self.since = 0;
            self.quiet = 0;
            self.shown.clear();
            let id = match &self.open {
                Some(open) => open.id,
                None => {
                    self.next += 1;
                    self.next
                }
            };
            self.line = Some((id, self.model.line(), Turn::new(start)));
        }
        let Some((_, current, _)) = self.line.as_mut() else { return Ok(()) };
        self.since += audio.len();
        self.model.feed(current, audio)
    }

    fn end(&mut self) -> Result<(), String> {
        let Some((id, mut current, turn)) = self.line.take() else { return Ok(()) };
        self.model.finish(&mut current)?;
        let text = self.model.text(&mut current)?;
        let source = self.open.as_ref().map_or(turn.owner(), |o| o.source);
        let lines = self.open.as_ref().map_or(0, |o| o.lines) + usize::from(!text.is_empty());
        let sentence = Sentence { id, text: join(self.open.as_ref(), &text, self.spaced), lines, source };
        self.quiet = 0;
        if complete(&sentence.text, sentence.lines) {
            self.open = None;
            self.close(sentence);
        } else {
            log(&format!("{id} open: {}", sentence.text));
            emit(self.app, Caption { id, text: sentence.text.clone(), done: false, source });
            self.open = Some(sentence);
        }
        Ok(())
    }

    fn partial(&mut self) -> Result<(), String> {
        let Some((id, current, turn)) = self.line.as_mut() else { return Ok(()) };
        if self.since < PARTIAL_SAMPLES {
            return Ok(());
        }
        self.since = 0;
        let text = join(self.open.as_ref(), &self.model.text(current)?, self.spaced);
        if text != self.shown {
            let source = self.open.as_ref().map_or(turn.owner(), |o| o.source);
            log(&format!("{id} {source:?}: {text}"));
            emit(self.app, Caption { id: *id, text: text.clone(), done: false, source });
            self.shown = text;
        }
        Ok(())
    }

    fn close(&self, sentence: Sentence) {
        log(&format!("{} final {:?}: {}", sentence.id, sentence.source, sentence.text));
        emit(self.app, Caption { id: sentence.id, text: sentence.text.clone(), done: true, source: sentence.source });
        if let Some(lines) = self.lines.as_ref().filter(|_| !sentence.text.is_empty()) {
            let _ = lines.send((sentence.id, sentence.text));
        }
    }
}

fn translate_lines(app: &AppHandle, mut engine: Engine, pending: Receiver<(u64, String)>, source: &str, target: &str, stop: &AtomicBool) -> Result<(), String> {
    while !stop.load(Ordering::Relaxed) {
        let first = match pending.recv_timeout(POLL) {
            Ok(line) => line,
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => break,
        };
        let mut batch = vec![first];
        while batch.len() < MAX_BATCH
            && let Ok(line) = pending.try_recv()
        {
            batch.push(line);
        }
        let requests: Vec<(String, usize)> =
            batch.iter().map(|(_, text)| (prompt::build(source, target, text), engine.tokens(text))).collect();
        let refs: Vec<(&str, usize)> = requests.iter().map(|(p, n)| (p.as_str(), *n)).collect();
        let replies = engine.complete_batch(&refs, stop)?;
        for ((id, _), reply) in batch.iter().zip(replies) {
            let translation = prompt::clean(&reply, target);
            log(&format!("{id} -> {translation}"));
            emit(app, Translation { id: *id, translation });
        }
    }
    Ok(())
}

pub fn stop_session(app: &AppHandle) {
    let session = app.state::<State>().captions.lock().unwrap().take();
    if let Some(mut session) = session {
        session.stop.store(true, Ordering::Relaxed);
        drop(session.capture.take());
        for thread in session.threads {
            let _ = thread.join();
        }
    }
    panel::close(app);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn open(text: &str) -> Sentence {
        Sentence { id: 1, text: text.into(), lines: 1, source: Source::Mic }
    }

    #[test]
    fn joins_lines_into_sentences() {
        assert_eq!(join(None, "Hello", true), "Hello");
        assert_eq!(join(Some(&open("Today we will")), "talk about it.", true), "Today we will talk about it.");
        assert_eq!(join(Some(&open("今天我们")), "讨论。", false), "今天我们讨论。");
        assert_eq!(join(Some(&open("Today")), "", true), "Today");
    }

    #[test]
    fn detects_a_turn_change() {
        let mut turn = Turn::new(Source::Mic);
        for _ in 0..MIN_TURN_TICKS {
            assert!(!turn.switched(0.01, 0.0));
        }
        assert!(!turn.switched(0.01, 0.002));
        let mut switched = false;
        for _ in 0..SWITCH_TICKS {
            switched |= turn.switched(0.0, 0.01);
        }
        assert!(switched);
        assert_eq!(Turn::new(Source::System).owner(), Source::System);
    }

    #[test]
    fn keeps_a_turn_through_echo() {
        let mut turn = Turn::new(Source::System);
        for _ in 0..MIN_TURN_TICKS + SWITCH_TICKS {
            assert!(!turn.switched(0.002, 0.01));
        }
        assert_eq!(turn.owner(), Source::System);
    }

    #[test]
    fn closes_at_sentence_end() {
        assert!(complete("Welcome to the meeting.", 1));
        assert!(complete("Is it ready?\"", 1));
        assert!(complete("会议开始。", 1));
        assert!(!complete("Today we will talk about the new release and", 1));
        assert!(complete("and so on and", MAX_SENTENCE_LINES));
        assert!(complete(&"a".repeat(MAX_SENTENCE_CHARS), 1));
    }
}
