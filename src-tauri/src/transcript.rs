mod decode;
pub mod languages;
mod mel;
mod tokens;

use crate::config::WhisperModel;
use crate::gif::Media;
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::{events, onnx, translate, video, whisper};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Media::MediaFoundation::{
    IMFAttributes, IMFSourceReader, MF_MT_AUDIO_AVG_BYTES_PER_SECOND, MF_MT_AUDIO_BITS_PER_SAMPLE, MF_MT_AUDIO_BLOCK_ALIGNMENT,
    MF_MT_AUDIO_NUM_CHANNELS, MF_MT_AUDIO_SAMPLES_PER_SECOND, MF_MT_MAJOR_TYPE, MF_MT_SUBTYPE, MF_PD_DURATION,
    MF_SOURCE_READER_ALL_STREAMS, MF_SOURCE_READER_FIRST_AUDIO_STREAM, MF_SOURCE_READER_MEDIASOURCE,
    MF_SOURCE_READERF_ENDOFSTREAM, MFAudioFormat_Float, MFCreateMediaType, MFCreateSourceReaderFromURL, MFMediaType_Audio,
};
use windows::core::{GUID, HSTRING};

pub const NO_AUDIO: &str = "NO_AUDIO";
pub const MODEL_MISSING: &str = "MODEL_MISSING";
pub const TRANSCRIBING: &str = "TRANSCRIBING";
pub const ALREADY_ENGLISH: &str = "ALREADY_ENGLISH";
const ENGLISH: &str = "en";
const TRANSCRIBE_KIND: &str = "transcribe";
const TRANSLATE_KIND: &str = "translate";
const ORIGINAL: &str = "original";
pub(crate) const SRT_EXT: &str = "srt";
const SAMPLE_RATE: u32 = 16_000;
const CHANNELS: u32 = 1;
const SAMPLE_BYTES: u32 = 4;
const BITS_PER_SAMPLE: u32 = 32;
const BITS_PER_BYTE: u32 = 8;
const PROGRESS_TOTAL: u64 = 100;
const DECODE_PERCENT: u64 = 10;
const MS_PER_SECOND: u64 = 1000;
const SECONDS_PER_MINUTE: u64 = 60;
const MINUTES_PER_HOUR: u64 = 60;
const ARROW: &str = "-->";
const TIME_PARTS: usize = 3;
const WHISPER_AUDIO: AudioFormat =
    AudioFormat { subtype: MFAudioFormat_Float, rate: SAMPLE_RATE, channels: CHANNELS, bits: BITS_PER_SAMPLE };

pub(crate) struct AudioFormat {
    pub(crate) subtype: GUID,
    pub(crate) rate: u32,
    pub(crate) channels: u32,
    pub(crate) bits: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Track {
    #[default]
    Original,
    Translation(&'static str),
}

pub const ENGLISH_TRACK: Track = Track::Translation(ENGLISH);

impl Track {
    pub fn parse(text: &str) -> Option<Track> {
        if text == ORIGINAL {
            return Some(Track::Original);
        }
        translate::languages::find(text).map(|language| Track::Translation(language.code))
    }

    pub fn code(self) -> Option<&'static str> {
        match self {
            Track::Original => None,
            Track::Translation(code) => Some(code),
        }
    }

    pub fn kind(self) -> &'static str {
        match self {
            Track::Original => TRANSCRIBE_KIND,
            Track::Translation(_) => TRANSLATE_KIND,
        }
    }

    pub fn stem(self, video: &Path) -> String {
        let stem = video.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        match self {
            Track::Original => stem,
            Track::Translation(code) => format!("{stem}.{code}"),
        }
    }

    pub fn path(self, video: &Path) -> PathBuf {
        video.with_file_name(format!("{}.{SRT_EXT}", self.stem(video)))
    }

    pub fn all() -> impl Iterator<Item = Track> {
        std::iter::once(Track::Original)
            .chain(translate::languages::LANGUAGES.iter().map(|language| Track::Translation(language.code)))
    }
}

impl Serialize for Track {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.code().unwrap_or(ORIGINAL))
    }
}

impl<'de> Deserialize<'de> for Track {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Track::parse(&text).ok_or_else(|| serde::de::Error::custom(format!("unknown track {text}")))
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct TranscribeTask {
    app: AppHandle,
    path: PathBuf,
    track: Track,
    then_translate: bool,
    out_dir: PathBuf,
    model: WhisperModel,
    model_dir: PathBuf,
    runtime: onnx::Target,
    language: String,
}

fn audio_stream() -> u32 {
    MF_SOURCE_READER_FIRST_AUDIO_STREAM.0 as u32
}

pub(crate) fn open_audio(path: &Path, format: &AudioFormat) -> Result<IMFSourceReader, String> {
    let fail = |e: windows::core::Error| e.to_string();
    unsafe {
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), None::<&IMFAttributes>).map_err(fail)?;
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false).map_err(fail)?;
        reader.SetStreamSelection(audio_stream(), true).map_err(|_| NO_AUDIO.to_string())?;
        let ty = MFCreateMediaType().map_err(fail)?;
        ty.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Audio).map_err(fail)?;
        let block = format.channels * format.bits / BITS_PER_BYTE;
        ty.SetGUID(&MF_MT_SUBTYPE, &format.subtype).map_err(fail)?;
        ty.SetUINT32(&MF_MT_AUDIO_NUM_CHANNELS, format.channels).map_err(fail)?;
        ty.SetUINT32(&MF_MT_AUDIO_SAMPLES_PER_SECOND, format.rate).map_err(fail)?;
        ty.SetUINT32(&MF_MT_AUDIO_BITS_PER_SAMPLE, format.bits).map_err(fail)?;
        ty.SetUINT32(&MF_MT_AUDIO_BLOCK_ALIGNMENT, block).map_err(fail)?;
        ty.SetUINT32(&MF_MT_AUDIO_AVG_BYTES_PER_SECOND, format.rate * block).map_err(fail)?;
        reader.SetCurrentMediaType(audio_stream(), None, &ty).map_err(fail)?;
        Ok(reader)
    }
}

fn duration_hns(reader: &IMFSourceReader) -> u64 {
    unsafe { reader.GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION) }
        .ok()
        .and_then(|value| u64::try_from(&value).ok())
        .unwrap_or(0)
}

pub(crate) fn read_audio(reader: &IMFSourceReader) -> Result<Option<(Vec<u8>, u64)>, String> {
    loop {
        let mut flags = 0u32;
        let mut stamp = 0i64;
        let mut sample = None;
        unsafe {
            reader
                .ReadSample(audio_stream(), 0, None, Some(&mut flags), Some(&mut stamp), Some(&mut sample))
                .map_err(|e| e.to_string())?;
        }
        if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
            return Ok(None);
        }
        let Some(sample) = sample else { continue };
        let buffer = unsafe { sample.ConvertToContiguousBuffer() }.map_err(|e| e.to_string())?;
        let mut data = std::ptr::null_mut();
        let mut len = 0u32;
        unsafe { buffer.Lock(&mut data, None, Some(&mut len)) }.map_err(|e| e.to_string())?;
        let bytes = unsafe { std::slice::from_raw_parts(data as *const u8, len as usize) }.to_vec();
        unsafe { buffer.Unlock() }.map_err(|e| e.to_string())?;
        return Ok(Some((bytes, stamp.max(0) as u64)));
    }
}

pub fn pcm(path: &Path, cancel: &AtomicBool, progress: &dyn Fn(f32), limit: Option<usize>) -> Result<Vec<f32>, String> {
    let _media = Media::start()?;
    let reader = open_audio(path, &WHISPER_AUDIO)?;
    let duration = duration_hns(&reader);
    let mut samples = Vec::new();
    loop {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let Some((bytes, stamp)) = read_audio(&reader)? else { break };
        samples.extend(bytes.as_chunks::<{ SAMPLE_BYTES as usize }>().0.iter().map(|b| f32::from_le_bytes(*b)));
        if let Some(limit) = limit.filter(|&limit| samples.len() >= limit) {
            samples.truncate(limit);
            break;
        }
        if duration > 0 {
            progress((stamp as f32 / duration as f32).min(1.0));
        }
    }
    Ok(samples)
}

fn clock(ms: u64) -> String {
    let hours = ms / (MS_PER_SECOND * SECONDS_PER_MINUTE * MINUTES_PER_HOUR);
    let minutes = ms / (MS_PER_SECOND * SECONDS_PER_MINUTE) % MINUTES_PER_HOUR;
    let seconds = ms / MS_PER_SECOND % SECONDS_PER_MINUTE;
    format!("{hours:02}:{minutes:02}:{seconds:02},{:03}", ms % MS_PER_SECOND)
}

fn render_srt(segments: &[Segment]) -> String {
    let mut out = String::new();
    for (index, segment) in segments.iter().enumerate() {
        let _ = write!(out, "{}\n{} {ARROW} {}\n{}\n\n", index + 1, clock(segment.start_ms), clock(segment.end_ms), segment.text);
    }
    out
}

fn parse_clock(text: &str) -> Option<u64> {
    let mut halves = text.trim().split([',', '.']);
    let hms = halves.next()?;
    let millis: u64 = halves.next()?.trim().parse().ok()?;
    let parts: Vec<u64> = hms.split(':').map(|p| p.parse().ok()).collect::<Option<_>>()?;
    let [hours, minutes, seconds] = <[u64; TIME_PARTS]>::try_from(parts).ok()?;
    let total_seconds = (hours * MINUTES_PER_HOUR + minutes) * SECONDS_PER_MINUTE + seconds;
    Some(total_seconds * MS_PER_SECOND + millis)
}

pub(crate) fn parse_srt(text: &str) -> Vec<Segment> {
    let normalized = text.trim_start_matches('\u{feff}').replace("\r\n", "\n");
    normalized
        .split("\n\n")
        .filter_map(|block| {
            let mut lines = block.lines();
            let timing = lines.find(|line| line.contains(ARROW))?;
            let (start, end) = timing.split_once(ARROW)?;
            let text = lines.collect::<Vec<_>>().join(" ").trim().to_string();
            Some(Segment { start_ms: parse_clock(start)?, end_ms: parse_clock(end)?, text })
        })
        .collect()
}

pub(crate) fn write_srt(part: &Path, segments: &[Segment]) -> Result<(), String> {
    fs::write(part, render_srt(segments)).map_err(|e| e.to_string())
}

impl TranscribeTask {
    pub fn validated(app: &AppHandle, path: &str, track: Track) -> Result<TranscribeTask, String> {
        let path = video::guarded_mp4(app, path)?;
        if track == ENGLISH_TRACK && languages::get(app, &path).as_deref() == Some(ENGLISH) {
            return Err(ALREADY_ENGLISH.into());
        }
        let out_dir = path.parent().ok_or("no output folder")?.to_path_buf();
        let (model, language) = {
            let state = app.state::<crate::State>();
            let cfg = state.cfg.read().unwrap();
            (cfg.whisper_model, cfg.transcript_language().to_string())
        };
        if !whisper::ready(app, model) {
            return Err(MODEL_MISSING.into());
        }
        Ok(TranscribeTask {
            app: app.clone(),
            path,
            track,
            then_translate: false,
            out_dir,
            model,
            model_dir: whisper::model_dir(app, model),
            runtime: onnx::target(app),
            language,
        })
    }

    pub fn then_translate(mut self, then: bool) -> Self {
        self.then_translate = then;
        self
    }

    fn queue_translation(&self) {
        let Some(path) = self.path.to_str() else { return };
        if let Ok(task) = translate::task::task(&self.app, path, None) {
            self.app.state::<crate::State>().queue.add(&self.app, task, None);
        }
    }
}

impl Task for TranscribeTask {
    fn kind(&self) -> &'static str {
        self.track.kind()
    }

    fn name(&self) -> String {
        self.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn source(&self) -> Option<&Path> {
        Some(&self.path)
    }

    fn detail(&self) -> Option<String> {
        self.track.code().map(str::to_owned)
    }

    fn run(&self, progress: &Progress, cancel: &AtomicBool) -> Result<Option<PathBuf>, String> {
        if !self.path.is_file() {
            return Err("video not found".into());
        }
        let _in_use = whisper::acquire(self.model);
        progress.set(0, PROGRESS_TOTAL);
        let samples = pcm(
            &self.path,
            cancel,
            &|fraction| {
                progress.set((fraction * DECODE_PERCENT as f32) as u64, PROGRESS_TOTAL);
            },
            None,
        )?;
        onnx::init(&self.runtime)?;
        let remaining = (PROGRESS_TOTAL - DECODE_PERCENT) as f32;
        let (segments, language) = onnx::retry_on_cpu(onnx::Workload::Whisper, || {
            decode::transcribe(
                &self.model_dir,
                &self.language,
                self.track == ENGLISH_TRACK,
                &samples,
                &|fraction| progress.set(DECODE_PERCENT + (fraction * remaining) as u64, PROGRESS_TOTAL),
                cancel,
            )
        })?;
        drop(samples);
        let stem = self.track.stem(&self.path);
        let output = queue::write_part_to(&self.out_dir, &stem, SRT_EXT, |part| write_srt(part, &segments), || {
            self.track.path(&self.path)
        })?;
        if self.track == Track::Original
            && let Some(language) = language
        {
            languages::set(&self.app, &self.path, &language);
            if self.then_translate {
                self.queue_translation();
            }
        }
        Ok(output)
    }
}

static DETECTING: Mutex<Option<HashSet<PathBuf>>> = Mutex::new(None);

fn detect_language(app: &AppHandle, path: &Path) -> Option<String> {
    let model = app.state::<crate::State>().cfg.read().unwrap().whisper_model;
    if !whisper::ready(app, model) {
        return None;
    }
    let first = DETECTING.lock().unwrap().get_or_insert_with(HashSet::new).insert(path.to_path_buf());
    if !first {
        return None;
    }
    let result = (|| {
        let _in_use = whisper::acquire(model);
        let never = AtomicBool::new(false);
        let samples = pcm(path, &never, &|_| {}, Some(decode::WINDOW_SAMPLES))?;
        if samples.is_empty() {
            return Err(NO_AUDIO.to_string());
        }
        onnx::init(&onnx::target(app))?;
        onnx::retry_on_cpu(onnx::Workload::Whisper, || decode::detect(&whisper::model_dir(app, model), &samples))
    })();
    if let Some(busy) = DETECTING.lock().unwrap().as_mut() {
        busy.remove(path);
    }
    let code = result.ok()?;
    languages::set(app, path, &code);
    let _ = app.emit_to("main", events::GALLERY_CHANGED, ());
    Some(code)
}

#[tauri::command]
pub async fn transcript_detect(app: AppHandle, path: String) -> Option<String> {
    let path = video::guarded_mp4(&app, &path).ok()?;
    if let Some(code) = languages::get(&app, &path) {
        return Some(code);
    }
    tauri::async_runtime::spawn_blocking(move || detect_language(&app, &path)).await.ok().flatten()
}

#[tauri::command]
pub fn transcript_read(app: AppHandle, path: String, track: Option<Track>) -> Result<Option<Vec<Segment>>, String> {
    let srt = track.unwrap_or_default().path(&video::guarded_mp4(&app, &path)?);
    match fs::read_to_string(&srt) {
        Ok(text) => Ok(Some(parse_srt(&text))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn transcript_save(app: AppHandle, path: String, track: Option<Track>, segments: Vec<Segment>) -> Result<(), String> {
    let track = track.unwrap_or_default();
    let path = video::guarded_mp4(&app, &path)?;
    if app.state::<crate::State>().queue.has(track.kind(), &path, track.code()) {
        return Err(TRANSCRIBING.into());
    }
    let dir = path.parent().ok_or("no output folder")?;
    queue::write_part_to(dir, &track.stem(&path), SRT_EXT, |part| write_srt(part, &segments), || track.path(&path)).map(drop)
}
