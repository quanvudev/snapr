use crate::{State, capture, gallery};
use crate::gif::{self, Media};
use crate::queue::{self, CANCELLED, Progress, Task};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Manager};
use windows::Media::MediaProperties::{MediaEncodingProfile, MediaEncodingSubtypes};
use windows::Media::Transcoding::MediaTranscoder;
use windows::Storage::StorageFile;
use windows::core::HSTRING;
use windows_future::{AsyncActionProgressHandler, AsyncStatus};

#[derive(Clone, Copy, Default, PartialEq)]
pub enum Quality {
    Low,
    #[default]
    Medium,
    High,
}

const QUALITIES: [(&str, Quality); 3] = [("low", Quality::Low), ("medium", Quality::Medium), ("high", Quality::High)];
const BITS_PER_PIXEL_LOW: f64 = 0.04;
const BITS_PER_PIXEL_MEDIUM: f64 = 0.07;
const BITS_PER_PIXEL_HIGH: f64 = 0.12;
const MIN_BITRATE: f64 = 500_000.0;
const MAX_BITRATE: f64 = 40_000_000.0;
const KIND: &str = "compress";
const VIDEO_EXT: &str = "mp4";
const COMPRESSED_SUFFIX: &str = "_compressed";
const HEIGHTS: [u32; 2] = [1080, 720];
const MIN_SIDE: u32 = 2;
const PERCENT: u64 = 100;
const POLL_INTERVAL: Duration = Duration::from_millis(100);
const FALLBACK_FPS: f64 = 30.0;
const BITS_PER_BYTE: u64 = 8;
const MS_PER_SECOND: u64 = 1000;

impl Quality {
    pub fn parse(name: &str) -> Option<Quality> {
        QUALITIES.iter().find(|(known, _)| *known == name).map(|&(_, quality)| quality)
    }

    fn bits_per_pixel(self) -> f64 {
        match self {
            Quality::Low => BITS_PER_PIXEL_LOW,
            Quality::Medium => BITS_PER_PIXEL_MEDIUM,
            Quality::High => BITS_PER_PIXEL_HIGH,
        }
    }
}

pub fn bitrate(width: u32, height: u32, fps: f64, quality: Quality) -> u32 {
    let bits = f64::from(width) * f64::from(height) * fps * quality.bits_per_pixel();
    bits.clamp(MIN_BITRATE, MAX_BITRATE) as u32
}

pub fn guarded_mp4(app: &AppHandle, path: &str) -> Result<PathBuf, String> {
    let path = gallery::guard(app, path).ok_or("path outside the output folder")?;
    if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(VIDEO_EXT)) {
        return Err("not an MP4 video".into());
    }
    Ok(path)
}

fn even(value: u32) -> u32 {
    (value & !1).max(MIN_SIDE)
}

fn output_size(width: u32, height: u32, target: Option<u32>) -> (u32, u32) {
    let out_height = even(target.map_or(height, |h| h.min(height)));
    let out_width = even(((u64::from(width) * u64::from(out_height)) / u64::from(height.max(1))) as u32);
    (out_width, out_height)
}

struct Source {
    width: u32,
    height: u32,
    duration_ms: u64,
    fps: f64,
    audio_bitrate: u32,
    file: StorageFile,
    profile: MediaEncodingProfile,
}

fn storage_file(path: &Path) -> windows::core::Result<StorageFile> {
    StorageFile::GetFileFromPathAsync(&HSTRING::from(path.as_os_str()))?.join()
}

fn source(path: &Path) -> Result<Source, String> {
    let (width, height, duration_ms) = gif::probe(path)?;
    let read = || -> windows::core::Result<Source> {
        let file = storage_file(path)?;
        let profile = MediaEncodingProfile::CreateFromFileAsync(&file)?.join()?;
        let rate = profile.Video()?.FrameRate()?;
        let (num, den) = (rate.Numerator()?, rate.Denominator()?);
        let fps = if num > 0 && den > 0 { f64::from(num) / f64::from(den) } else { FALLBACK_FPS };
        let audio_bitrate = profile.Audio().and_then(|audio| audio.Bitrate()).unwrap_or(0);
        Ok(Source { width, height, duration_ms, fps, audio_bitrate, file, profile })
    };
    read().map_err(|e| e.to_string())
}

pub struct CompressTask {
    path: PathBuf,
    out_dir: PathBuf,
    quality: Quality,
    height: Option<u32>,
    replace: bool,
}

impl CompressTask {
    pub fn validated(app: &AppHandle, path: &str, quality: &str, height: Option<u32>) -> Result<CompressTask, String> {
        let quality = Quality::parse(quality).ok_or("unsupported quality")?;
        if height.is_some_and(|h| !HEIGHTS.contains(&h)) {
            return Err("unsupported resolution".into());
        }
        let path = guarded_mp4(app, path)?;
        let out_dir = path.parent().ok_or("no output folder")?.to_path_buf();
        Ok(CompressTask { path, out_dir, quality, height, replace: false })
    }

    fn original_stem(&self) -> String {
        self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn stem(&self) -> String {
        format!("{}{COMPRESSED_SUFFIX}", self.original_stem())
    }

    fn replace_original(&self) -> PathBuf {
        match gallery::delete_file(&self.path) {
            Ok(()) => self.path.clone(),
            Err(_) => capture::stem_path(&self.out_dir, &self.stem(), VIDEO_EXT),
        }
    }

    fn transcode(&self, part: &Path, progress: &Progress, cancel: &AtomicBool) -> Result<(), String> {
        if !self.path.is_file() {
            return Err("video not found".into());
        }
        let _media = Media::start()?;
        let source = source(&self.path)?;
        std::fs::File::create(part).map_err(|e| e.to_string())?;
        let (width, height) = output_size(source.width, source.height, self.height);
        let rate = bitrate(width, height, source.fps, self.quality);
        let percent = Arc::new(AtomicU64::new(0));
        let operation = {
            let percent = percent.clone();
            let start = || -> windows::core::Result<_> {
                let destination = storage_file(part)?;
                let video = source.profile.Video()?;
                video.SetSubtype(&MediaEncodingSubtypes::H264()?)?;
                video.SetWidth(width)?;
                video.SetHeight(height)?;
                video.SetBitrate(rate)?;
                let transcoder = MediaTranscoder::new()?;
                transcoder.SetHardwareAccelerationEnabled(true)?;
                let prepared = transcoder.PrepareFileTranscodeAsync(&source.file, &destination, &source.profile)?.join()?;
                if !prepared.CanTranscode()? {
                    return Ok(Err(format!("cannot transcode: {:?}", prepared.FailureReason()?)));
                }
                let operation = prepared.TranscodeAsync()?;
                operation.SetProgress(&AsyncActionProgressHandler::new(move |_, value: windows::core::Ref<f64>| {
                    percent.store((*value).clamp(0.0, PERCENT as f64) as u64, Ordering::Relaxed);
                    Ok(())
                }))?;
                Ok(Ok(operation))
            };
            start().map_err(|e| e.to_string())??
        };
        progress.set(0, PERCENT);
        loop {
            let status = operation.Status().map_err(|e| e.to_string())?;
            if status != AsyncStatus::Started {
                return operation.GetResults().map_err(|e| e.to_string());
            }
            if cancel.load(Ordering::Relaxed) {
                let _ = operation.Cancel();
                while operation.Status().is_ok_and(|status| status == AsyncStatus::Started) {
                    std::thread::sleep(POLL_INTERVAL);
                }
                return Err(CANCELLED.into());
            }
            progress.set(percent.load(Ordering::Relaxed), PERCENT);
            std::thread::sleep(POLL_INTERVAL);
        }
    }
}

impl Task for CompressTask {
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
        let work = |part: &Path| self.transcode(part, progress, cancel);
        if self.replace {
            return queue::write_part_to(&self.out_dir, &self.original_stem(), VIDEO_EXT, work, || self.replace_original());
        }
        queue::write_part(&self.out_dir, &self.stem(), VIDEO_EXT, work)
    }
}

pub fn compress_recording(app: &AppHandle, path: PathBuf, height: u32, quality: Quality) {
    let app = app.clone();
    std::thread::spawn(move || {
        let Some(out_dir) = path.parent().map(Path::to_path_buf) else { return };
        if !gif::probe(&path).is_ok_and(|(_, source_height, _)| source_height > height) {
            return;
        }
        let task = CompressTask { path, out_dir, quality, height: Some(height), replace: true };
        app.state::<State>().queue.add(&app, Box::new(task), None);
    });
}

#[derive(Serialize)]
pub struct Estimate {
    quality: String,
    height: Option<u32>,
    bytes: u64,
}

#[derive(Serialize)]
pub struct CompressInfo {
    width: u32,
    height: u32,
    fps: f64,
    duration_ms: u64,
    size: u64,
    estimates: Vec<Estimate>,
}

fn info(path: &Path) -> Result<CompressInfo, String> {
    let size = std::fs::metadata(path).map_err(|e| e.to_string())?.len();
    let _media = Media::start()?;
    let source = source(path)?;
    let targets = std::iter::once(None).chain(HEIGHTS.iter().filter(|&&h| h <= source.height).map(|&h| Some(h)));
    let targets: Vec<Option<u32>> = targets.collect();
    let estimates = QUALITIES
        .iter()
        .flat_map(|&(name, quality)| {
            let source = &source;
            targets.iter().map(move |&target| {
                let (width, height) = output_size(source.width, source.height, target);
                let bits = u64::from(bitrate(width, height, source.fps, quality)) + u64::from(source.audio_bitrate);
                Estimate { quality: name.into(), height: target, bytes: bits * source.duration_ms / BITS_PER_BYTE / MS_PER_SECOND }
            })
        })
        .collect();
    Ok(CompressInfo {
        width: source.width,
        height: source.height,
        fps: source.fps,
        duration_ms: source.duration_ms,
        size,
        estimates,
    })
}

#[tauri::command]
pub async fn compress_info(app: AppHandle, path: String) -> Result<CompressInfo, String> {
    let path = guarded_mp4(&app, &path)?;
    tauri::async_runtime::spawn_blocking(move || info(&path)).await.map_err(|e| e.to_string())?
}
