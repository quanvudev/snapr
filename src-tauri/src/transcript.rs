mod decode;
mod mel;
mod tokens;

use crate::config::WhisperModel;
use crate::gif::Media;
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::{onnx, video, whisper};
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};
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
const KIND: &str = "transcribe";
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

#[derive(Clone, Serialize, Deserialize)]
pub struct Segment {
    pub start_ms: u64,
    pub end_ms: u64,
    pub text: String,
}

pub struct TranscribeTask {
    path: PathBuf,
    out_dir: PathBuf,
    model: WhisperModel,
    model_dir: PathBuf,
    runtime: PathBuf,
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

pub fn pcm(path: &Path, cancel: &AtomicBool, progress: &dyn Fn(f32)) -> Result<Vec<f32>, String> {
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

fn write_srt(part: &Path, segments: &[Segment]) -> Result<(), String> {
    fs::write(part, render_srt(segments)).map_err(|e| e.to_string())
}

impl TranscribeTask {
    pub fn validated(app: &AppHandle, path: &str) -> Result<TranscribeTask, String> {
        let path = video::guarded_mp4(app, path)?;
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
            path,
            out_dir,
            model,
            model_dir: whisper::model_dir(app, model),
            runtime: onnx::runtime_path(app),
            language,
        })
    }

    fn stem(&self) -> String {
        self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }
}

impl Task for TranscribeTask {
    fn kind(&self) -> &'static str {
        KIND
    }

    fn name(&self) -> String {
        self.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn source(&self) -> Option<&Path> {
        Some(&self.path)
    }

    fn run(&self, progress: &Progress, cancel: &AtomicBool) -> Result<Option<PathBuf>, String> {
        if !self.path.is_file() {
            return Err("video not found".into());
        }
        let _in_use = whisper::acquire(self.model);
        progress.set(0, PROGRESS_TOTAL);
        let samples = pcm(&self.path, cancel, &|fraction| {
            progress.set((fraction * DECODE_PERCENT as f32) as u64, PROGRESS_TOTAL);
        })?;
        onnx::init(&self.runtime)?;
        let remaining = (PROGRESS_TOTAL - DECODE_PERCENT) as f32;
        let segments = decode::transcribe(
            &self.model_dir,
            &self.language,
            &samples,
            &|fraction| progress.set(DECODE_PERCENT + (fraction * remaining) as u64, PROGRESS_TOTAL),
            cancel,
        )?;
        drop(samples);
        let stem = self.stem();
        queue::write_part_to(&self.out_dir, &stem, SRT_EXT, |part| write_srt(part, &segments), || {
            self.out_dir.join(format!("{stem}.{SRT_EXT}"))
        })
    }
}

#[tauri::command]
pub fn transcript_read(app: AppHandle, path: String) -> Result<Option<Vec<Segment>>, String> {
    let srt = video::guarded_mp4(&app, &path)?.with_extension(SRT_EXT);
    match fs::read_to_string(&srt) {
        Ok(text) => Ok(Some(parse_srt(&text))),
        Err(e) if e.kind() == ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn transcript_save(app: AppHandle, path: String, segments: Vec<Segment>) -> Result<(), String> {
    let path = video::guarded_mp4(&app, &path)?;
    if app.state::<crate::State>().queue.has(KIND, &path) {
        return Err(TRANSCRIBING.into());
    }
    let dir = path.parent().ok_or("no output folder")?;
    let stem = path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
    queue::write_part_to(dir, &stem, SRT_EXT, |part| write_srt(part, &segments), || path.with_extension(SRT_EXT)).map(drop)
}
