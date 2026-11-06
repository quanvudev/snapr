use crate::gif::{self, HNS_PER_MS, Media};
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::transcript::{self, SRT_EXT, Segment, Track, languages};
use crate::video::{self, Quality};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::Duration;
use tauri::AppHandle;
use windows::Foundation::TimeSpan;
use windows::Media::Editing::{MediaClip, MediaComposition, MediaTrimmingPreference};
use windows::Media::Transcoding::TranscodeFailureReason;
use windows_future::{AsyncOperationProgressHandler, AsyncStatus};

const KIND: &str = "trim";
const VIDEO_EXT: &str = "mp4";
const TRIMMED_SUFFIX: &str = "_trimmed";
const END_SLACK_MS: u64 = 100;
const PERCENT: u64 = 100;
const POLL_INTERVAL: Duration = Duration::from_millis(100);

pub struct TrimTask {
    app: AppHandle,
    path: PathBuf,
    out_dir: PathBuf,
    start_ms: u64,
    end_ms: u64,
}

fn cut(segments: Vec<Segment>, start_ms: u64, end_ms: u64) -> Vec<Segment> {
    segments
        .into_iter()
        .filter(|s| s.end_ms > s.start_ms && s.end_ms > start_ms && s.start_ms < end_ms)
        .map(|s| Segment {
            start_ms: s.start_ms.max(start_ms) - start_ms,
            end_ms: s.end_ms.min(end_ms) - start_ms,
            text: s.text,
        })
        .collect()
}

fn span(ms: u64) -> TimeSpan {
    TimeSpan { Duration: (ms * HNS_PER_MS) as i64 }
}

fn render(path: &Path, start_ms: u64, end_ms: u64, part: &Path, report: &dyn Fn(u64), cancel: &AtomicBool) -> Result<(), String> {
    if !path.is_file() {
        return Err("video not found".into());
    }
    let _media = Media::start()?;
    let source = video::source(path)?;
    std::fs::File::create(part).map_err(|e| e.to_string())?;
    let rate = video::bitrate(source.width, source.height, source.fps, Quality::High);
    let percent = Arc::new(AtomicU64::new(0));
    let operation = {
        let percent = percent.clone();
        let start = || -> windows::core::Result<_> {
            let destination = video::storage_file(part)?;
            source.profile.Video()?.SetBitrate(rate)?;
            let clip = MediaClip::CreateFromFileAsync(&source.file)?.join()?;
            let original = clip.OriginalDuration()?.Duration;
            let from_end = (original - span(end_ms).Duration).max(0);
            clip.SetTrimTimeFromStart(span(start_ms))?;
            clip.SetTrimTimeFromEnd(TimeSpan { Duration: from_end })?;
            let composition = MediaComposition::new()?;
            composition.Clips()?.Append(&clip)?;
            let operation =
                composition.RenderToFileWithProfileAsync(&destination, MediaTrimmingPreference::Precise, &source.profile)?;
            operation.SetProgress(&AsyncOperationProgressHandler::new(move |_, value: windows::core::Ref<f64>| {
                percent.store((*value).clamp(0.0, PERCENT as f64) as u64, Ordering::Relaxed);
                Ok(())
            }))?;
            Ok(operation)
        };
        start().map_err(|e| e.to_string())?
    };
    report(0);
    loop {
        let status = operation.Status().map_err(|e| e.to_string())?;
        if status != AsyncStatus::Started {
            let reason = operation.GetResults().map_err(|e| e.to_string())?;
            if reason != TranscodeFailureReason::None {
                return Err(format!("cannot trim: {reason:?}"));
            }
            return Ok(());
        }
        if cancel.load(Ordering::Relaxed) {
            let _ = operation.Cancel();
            while operation.Status().is_ok_and(|status| status == AsyncStatus::Started) {
                std::thread::sleep(POLL_INTERVAL);
            }
            return Err(CANCELLED.into());
        }
        report(percent.load(Ordering::Relaxed));
        std::thread::sleep(POLL_INTERVAL);
    }
}

impl TrimTask {
    pub fn validated(app: &AppHandle, path: &str, start_ms: u64, end_ms: u64) -> Result<TrimTask, String> {
        let path = video::guarded_mp4(app, path)?;
        let (_, _, duration) = gif::probe(&path)?;
        if start_ms >= end_ms || end_ms > duration + END_SLACK_MS {
            return Err("invalid range".into());
        }
        let out_dir = path.parent().ok_or("no output folder")?.to_path_buf();
        Ok(TrimTask { app: app.clone(), path, out_dir, start_ms, end_ms: end_ms.min(duration) })
    }

    fn stem(&self) -> String {
        let stem = self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        format!("{stem}{TRIMMED_SUFFIX}")
    }

    fn copy_tracks(&self, output: &Path) {
        for track in Track::all() {
            let Ok(text) = std::fs::read_to_string(track.path(&self.path)) else { continue };
            let segments = cut(transcript::parse_srt(&text), self.start_ms, self.end_ms);
            let _ = queue::write_part_to(
                &self.out_dir,
                &track.stem(output),
                SRT_EXT,
                |part| transcript::write_srt(part, &segments),
                || track.path(output),
            );
        }
        if let Some(code) = languages::get(&self.app, &self.path) {
            languages::set(&self.app, output, &code);
        }
    }
}

impl Task for TrimTask {
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
        let output = queue::write_part(&self.out_dir, &self.stem(), VIDEO_EXT, |part| {
            render(&self.path, self.start_ms, self.end_ms, part, &|done| progress.set(done, PERCENT), cancel)
        })?;
        if let Some(output) = &output {
            self.copy_tracks(output);
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg(start_ms: u64, end_ms: u64, text: &str) -> Segment {
        Segment { start_ms, end_ms, text: text.into() }
    }

    fn times(segments: &[Segment]) -> Vec<(u64, u64, &str)> {
        segments.iter().map(|s| (s.start_ms, s.end_ms, s.text.as_str())).collect()
    }

    #[test]
    fn keeps_overlapping_cues_shifted() {
        let out = cut(vec![seg(6_000, 8_000, "a"), seg(10_000, 12_000, "b")], 5_000, 40_000);
        assert_eq!(times(&out), vec![(1_000, 3_000, "a"), (5_000, 7_000, "b")]);
    }

    #[test]
    fn clips_cues_to_range() {
        let out = cut(vec![seg(3_000, 7_000, "start"), seg(38_000, 45_000, "end")], 5_000, 40_000);
        assert_eq!(times(&out), vec![(0, 2_000, "start"), (33_000, 35_000, "end")]);
    }

    #[test]
    fn drops_out_of_range_cues() {
        let out = cut(
            vec![seg(0, 4_000, "before"), seg(4_000, 5_000, "touch start"), seg(40_000, 41_000, "touch end"), seg(50_000, 52_000, "after")],
            5_000,
            40_000,
        );
        assert!(out.is_empty());
    }

    #[test]
    #[ignore]
    fn renders_trim() {
        let source = PathBuf::from(std::env::var("SNAPR_TRIM_SOURCE").unwrap());
        let part = PathBuf::from(std::env::var("SNAPR_TRIM_OUT").unwrap());
        let started = std::time::Instant::now();
        render(&source, 10_000, 70_000, &part, &|done| println!("{done}%"), &AtomicBool::new(false)).unwrap();
        println!("{:?} {:?}", started.elapsed(), gif::probe(&part).unwrap());
    }

    #[test]
    fn drops_empty_cues() {
        let out = cut(vec![seg(10_000, 10_000, "zero"), seg(12_000, 11_000, "reversed")], 5_000, 40_000);
        assert!(out.is_empty());
    }
}
