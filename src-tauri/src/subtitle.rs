use crate::gif::{self, HNS_PER_MS, Media, Video};
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::transcript::{self, AudioFormat, NO_AUDIO, SRT_EXT, Segment};
use crate::video::{self, Quality};
use crate::{draw, toolbar};
use ab_glyph::FontVec;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::AppHandle;
use tiny_skia::{Pixmap, Point};
use windows::Win32::Media::MediaFoundation::{IMFSample, IMFSourceReader, MFAudioFormat_PCM};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder, VideoSettingsSubType,
};

pub const NO_TRANSCRIPT: &str = "NO_TRANSCRIPT";
pub const NO_FONT: &str = "NO_FONT";
const KIND: &str = "subtitle";
const VIDEO_EXT: &str = "mp4";
const SUBTITLED_SUFFIX: &str = "_subtitled";
const FONT_FILES: [&str; 2] = ["segoeuib.ttf", "segoeui.ttf"];
const SIZE_RATIO: [f32; 3] = [0.04, 0.055, 0.075];
const MIN_TEXT_PX: f32 = 16.0;
const EDGE_MARGIN_RATIO: f32 = 0.06;
const MAX_WIDTH_RATIO: f32 = 0.9;
const BOX_ALPHA: f32 = 0.6;
const BOX_PAD_X_RATIO: f32 = 0.5;
const BOX_PAD_Y_RATIO: f32 = 0.25;
const LINE_HEIGHT_RATIO: f32 = 1.25;
const BOX_RADIUS_RATIO: f32 = 0.25;
const BOX_RGB: [u8; 3] = [0x00, 0x00, 0x00];
const TEXT_RGB: [u8; 3] = [0xFF, 0xFF, 0xFF];
const FALLBACK_FPS: f64 = 30.0;
const PERCENT: u64 = 100;
const BYTES_PER_PIXEL: usize = 4;
const ALPHA: usize = 3;
const CHANNEL_MAX: u16 = u8::MAX as u16;
const ROUNDING: u16 = CHANNEL_MAX / 2;
const FRAME_QUEUE: usize = 8;
const QUEUE_RETRY: std::time::Duration = std::time::Duration::from_millis(2);
const AUDIO_RATE: u32 = 48_000;
const AUDIO_CHANNELS: u32 = 2;
const AUDIO_BITS: u32 = 16;
const ENCODER_AUDIO: AudioFormat =
    AudioFormat { subtype: MFAudioFormat_PCM, rate: AUDIO_RATE, channels: AUDIO_CHANNELS, bits: AUDIO_BITS };

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Position {
    Bottom,
    Top,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum Size {
    Small,
    Medium,
    Large,
}

struct Cue {
    bgra: Vec<u8>,
    width: usize,
    left: usize,
    bottom: usize,
}

fn wrap(font: &FontVec, size: f32, text: &str, max: f32) -> Vec<String> {
    let width = |line: &str| draw::measure(font, size, line).0;
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let joined = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if line.is_empty() || width(&joined) <= max {
            line = joined;
        } else {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        }
        while width(&line) > max && line.chars().nth(1).is_some() {
            let mut cut = String::new();
            for c in line.chars() {
                let mut next = cut.clone();
                next.push(c);
                if !cut.is_empty() && width(&next) > max {
                    break;
                }
                cut = next;
            }
            line = line[cut.len()..].to_string();
            lines.push(cut);
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

impl Cue {
    fn render(font: &FontVec, frame: (u32, u32), text: &str, position: Position, size: Size) -> Option<Cue> {
        let (frame_w, frame_h) = (frame.0 as f32, frame.1 as f32);
        let text_px = (frame_h * SIZE_RATIO[size as usize]).max(MIN_TEXT_PX);
        let (pad_x, pad_y) = (text_px * BOX_PAD_X_RATIO, text_px * BOX_PAD_Y_RATIO);
        let line_height = text_px * LINE_HEIGHT_RATIO;
        let max_box = frame_w * MAX_WIDTH_RATIO;
        let lines = wrap(font, text_px, text, max_box - 2.0 * pad_x);
        let widths: Vec<f32> = lines.iter().map(|line| draw::measure(font, text_px, line).0).collect();
        let box_w = (widths.iter().copied().fold(0.0, f32::max) + 2.0 * pad_x).min(frame_w);
        let box_h = (lines.len() as f32 * line_height + 2.0 * pad_y).min(frame_h);
        let mut pixmap = Pixmap::new(box_w.ceil() as u32, box_h.ceil() as u32)?;
        let alpha = (BOX_ALPHA * f32::from(u8::MAX)).round() as u8;
        toolbar::fill(&mut pixmap, (0.0, 0.0, box_w, box_h), text_px * BOX_RADIUS_RATIO, BOX_RGB, alpha);
        let glyph_height = draw::measure(font, text_px, "").1;
        for (i, (line, width)) in lines.iter().zip(&widths).enumerate() {
            let top = pad_y + i as f32 * line_height + (line_height - glyph_height) / 2.0;
            let at = Point::from_xy((box_w - width) / 2.0, top);
            draw::text(&mut pixmap, font, at, std::slice::from_ref(line), text_px, TEXT_RGB, None);
        }
        let margin = frame_h * EDGE_MARGIN_RATIO;
        let y = match position {
            Position::Bottom => frame_h - margin - box_h,
            Position::Top => margin,
        };
        let left = ((frame_w - box_w) / 2.0).round().max(0.0) as usize;
        let top = y.clamp(0.0, (frame_h - box_h).max(0.0)).round() as usize;
        let width = (pixmap.width() as usize).min((frame.0 as usize).saturating_sub(left));
        let height = (pixmap.height() as usize).min((frame.1 as usize).saturating_sub(top));
        let stride = pixmap.width() as usize * BYTES_PER_PIXEL;
        let rows = pixmap.data().chunks_exact(stride).take(height).rev();
        let bgra = rows
            .flat_map(|row| row[..width * BYTES_PER_PIXEL].as_chunks::<BYTES_PER_PIXEL>().0.iter().flat_map(|p| [p[2], p[1], p[0], p[3]]))
            .collect();
        Some(Cue { bgra, width, left, bottom: frame.1 as usize - top - height })
    }

    fn burn(&self, frame: &mut [u8], frame_width: u32) {
        let stride = frame_width as usize * BYTES_PER_PIXEL;
        let row_len = self.width * BYTES_PER_PIXEL;
        if row_len == 0 {
            return;
        }
        for (row, src) in self.bgra.chunks_exact(row_len).enumerate() {
            let start = (self.bottom + row) * stride + self.left * BYTES_PER_PIXEL;
            let dst = &mut frame[start..start + row_len];
            for (d, s) in dst.as_chunks_mut::<BYTES_PER_PIXEL>().0.iter_mut().zip(src.as_chunks::<BYTES_PER_PIXEL>().0) {
                let keep = CHANNEL_MAX - u16::from(s[ALPHA]);
                for (dc, sc) in d.iter_mut().zip(s) {
                    let blended = u16::from(*sc) + (u16::from(*dc) * keep + ROUNDING) / CHANNEL_MAX;
                    *dc = blended.min(CHANNEL_MAX) as u8;
                }
            }
        }
    }
}

pub struct SubtitleTask {
    path: PathBuf,
    out_dir: PathBuf,
    segments: Vec<Segment>,
    position: Position,
    size: Size,
    font: FontVec,
}

fn open_audio(path: &Path) -> Result<Option<IMFSourceReader>, String> {
    match transcript::open_audio(path, &ENCODER_AUDIO) {
        Ok(reader) => Ok(Some(reader)),
        Err(e) if e == NO_AUDIO => Ok(None),
        Err(e) => Err(e),
    }
}

fn read_audio(reader: Option<&IMFSourceReader>) -> Result<Option<(Vec<u8>, u64)>, String> {
    reader.map_or(Ok(None), transcript::read_audio)
}

impl SubtitleTask {
    pub fn validated(app: &AppHandle, path: &str, position: Position, size: Size) -> Result<SubtitleTask, String> {
        let path = video::guarded_mp4(app, path)?;
        let out_dir = path.parent().ok_or("no output folder")?.to_path_buf();
        let srt = std::fs::read_to_string(path.with_extension(SRT_EXT)).map_err(|_| NO_TRANSCRIPT.to_string())?;
        let segments: Vec<Segment> =
            transcript::parse_srt(&srt).into_iter().filter(|segment| !segment.text.trim().is_empty()).collect();
        if segments.is_empty() {
            return Err(NO_TRANSCRIPT.into());
        }
        let font = draw::system_font(&FONT_FILES).ok_or(NO_FONT)?;
        Ok(SubtitleTask { path, out_dir, segments, position, size, font })
    }

    fn stem(&self) -> String {
        let stem = self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        format!("{stem}{SUBTITLED_SUFFIX}")
    }

    fn active(&self, ms: u64) -> Option<usize> {
        let index = self.segments.partition_point(|segment| segment.start_ms <= ms).checked_sub(1)?;
        (ms < self.segments[index].end_ms).then_some(index)
    }

    fn frame(
        &self,
        video: &Video,
        sample: &IMFSample,
        stamp: u64,
        size: (u32, u32),
        buffer: &mut Vec<u8>,
        cache: &mut Option<(usize, Option<Cue>)>,
    ) -> Result<(), String> {
        video.bottom_up_bgra(sample, size.0, size.1, buffer).map_err(|e| e.to_string())?;
        let Some(index) = self.active(stamp / HNS_PER_MS) else { return Ok(()) };
        if cache.as_ref().is_none_or(|(cached, _)| *cached != index) {
            let text = self.segments[index].text.trim();
            *cache = Some((index, Cue::render(&self.font, size, text, self.position, self.size)));
        }
        if let Some((_, Some(cue))) = cache {
            cue.burn(buffer, size.0);
        }
        Ok(())
    }

    fn encode(&self, part: &Path, progress: &dyn Fn(u64), cancel: &AtomicBool) -> Result<(), String> {
        if !self.path.is_file() {
            return Err("video not found".into());
        }
        let _media = Media::start()?;
        let video = gif::open(&self.path).map_err(|e| e.to_string())?;
        let audio = open_audio(&self.path)?;
        let (width, height) = (video.width & !1, video.height & !1);
        if width == 0 || height == 0 {
            return Err("video is too small".into());
        }
        let fps = video.fps.unwrap_or(FALLBACK_FPS);
        let mut encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(width, height)
                .sub_type(VideoSettingsSubType::H264)
                .frame_rate(fps.round() as u32)
                .frame_queue(FRAME_QUEUE)
                .bitrate(video::bitrate(width, height, fps, Quality::High)),
            AudioSettingsBuilder::new()
                .sample_rate(AUDIO_RATE)
                .channel_count(AUDIO_CHANNELS)
                .bit_per_sample(AUDIO_BITS)
                .disabled(audio.is_none()),
            ContainerSettingsBuilder::new(),
            part,
        )
        .map_err(|e| e.to_string())?;
        progress(0);
        let mut cache = None;
        let mut sent = false;
        let mut pending: Option<(Vec<u8>, u64)> = None;
        let mut frame = video.read().map_err(|e| e.to_string())?;
        let mut chunk = read_audio(audio.as_ref())?;
        loop {
            if cancel.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            let video_stamp = pending.as_ref().map(|(_, stamp)| *stamp).or(frame.as_ref().map(|(_, stamp)| *stamp));
            let take_video = match (video_stamp, &chunk) {
                (None, None) => break,
                (Some(video_at), Some((_, audio_at))) => video_at <= *audio_at,
                (video_next, _) => video_next.is_some(),
            };
            if take_video {
                if pending.is_none() {
                    let Some((sample, stamp)) = frame.take() else { break };
                    let mut buffer = Vec::new();
                    self.frame(&video, &sample, stamp, (width, height), &mut buffer, &mut cache)?;
                    pending = Some((buffer, stamp));
                    frame = video.read().map_err(|e| e.to_string())?;
                }
                let Some((buffer, stamp)) = pending.take() else { break };
                match encoder.try_send_frame_buffer(buffer, stamp as i64).map_err(|e| e.to_string())? {
                    None => {
                        sent = true;
                        if let Some(done) = (stamp * PERCENT).checked_div(video.duration) {
                            progress(done.min(PERCENT));
                        }
                    }
                    Some(back) => {
                        pending = Some((back, stamp));
                        if let Some((bytes, _)) = chunk.take() {
                            encoder.send_audio_buffer(&bytes, 0).map_err(|e| e.to_string())?;
                            chunk = read_audio(audio.as_ref())?;
                        } else {
                            std::thread::sleep(QUEUE_RETRY);
                        }
                    }
                }
            } else if let Some((bytes, _)) = chunk.take() {
                encoder.send_audio_buffer(&bytes, 0).map_err(|e| e.to_string())?;
                chunk = read_audio(audio.as_ref())?;
            }
        }
        if !sent {
            return Err("no frames decoded".into());
        }
        encoder.finish().map_err(|e| e.to_string())
    }
}

impl Task for SubtitleTask {
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
        queue::write_part(&self.out_dir, &self.stem(), VIDEO_EXT, |part| {
            self.encode(part, &|done| progress.set(done, PERCENT), cancel)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_ENV: &str = "SNAPR_SUBTITLE_TEST";
    const TEST_OUTPUT: &str = "snapr_subtitle_test.mp4.part";

    #[test]
    #[ignore]
    fn encodes_without_stalling() {
        let Ok(video) = std::env::var(TEST_ENV) else { return };
        let path = PathBuf::from(video);
        let srt = std::fs::read_to_string(path.with_extension(SRT_EXT)).unwrap();
        let segments = transcript::parse_srt(&srt).into_iter().filter(|s| !s.text.trim().is_empty()).collect();
        let task = SubtitleTask {
            out_dir: std::env::temp_dir(),
            path,
            segments,
            position: Position::Bottom,
            size: Size::Medium,
            font: draw::system_font(&FONT_FILES).unwrap(),
        };
        let part = std::env::temp_dir().join(TEST_OUTPUT);
        let result = task.encode(&part, &|_| {}, &AtomicBool::new(false));
        let _ = std::fs::remove_file(&part);
        assert!(result.is_ok(), "{result:?}");
    }
}
