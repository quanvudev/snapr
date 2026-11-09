use crate::Action;
use crate::audio::Chunk;
use crate::video::{self, Quality};
use image::RgbaImage;
use std::ffi::c_void;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use windows_capture::capture::{CaptureControl, Context, GraphicsCaptureApiHandler};
use windows_capture::encoder::{
    AudioSettingsBuilder, ContainerSettingsBuilder, VideoEncoder, VideoSettingsBuilder, VideoSettingsSubType,
};
use windows_capture::frame::Frame;
use windows_capture::graphics_capture_api::InternalCaptureControl;
use windows_capture::settings::{
    ColorFormat, CursorCaptureSettings, DirtyRegionSettings, DrawBorderSettings, GraphicsCaptureItemType,
    MinimumUpdateIntervalSettings, SecondaryWindowSettings, Settings,
};
use windows_capture::monitor::Monitor;
use windows_capture::window::Window;
use winit::event_loop::EventLoopProxy;

#[derive(Clone, Copy)]
pub(crate) enum Target {
    Window(isize),
    Monitor(isize),
    Area { monitor: isize, rect: Area },
}

pub(crate) type Area = (u32, u32, u32, u32);

type Flags = (Target, PathBuf, EventLoopProxy<Action>, u32, Quality, Option<Receiver<Chunk>>);

struct Handler {
    encoder: Option<VideoEncoder>,
    size: (u32, u32),
    crop: Option<Area>,
    interval: i64,
    next: Option<i64>,
    kept: Option<i64>,
    last: (Vec<u8>, u32, u32),
    path: PathBuf,
    proxy: EventLoopProxy<Action>,
    audio: Option<Receiver<Chunk>>,
    paused: Option<Instant>,
    offset: i64,
}

impl Handler {
    fn finalize(&mut self) {
        if self.paused.is_none() && self.flush().is_err() {
            return;
        }
        let Some(encoder) = self.encoder.take() else { return };
        match encoder.finish() {
            Ok(()) => {
                let thumbnail = self.thumbnail();
                let _ = self.proxy.send_event(Action::Recorded(thumbnail, self.path.clone(), self.audio.is_some()));
            }
            Err(e) => self.fail(e),
        }
    }

    fn fail(&self, e: impl std::fmt::Display) {
        let _ = self.proxy.send_event(Action::RecordFailed(e.to_string()));
    }

    fn discard(&self) {
        if let Some(audio) = &self.audio {
            audio.try_iter().for_each(drop);
        }
    }

    fn flush(&mut self) -> Result<(), ()> {
        let (Some(audio), Some(encoder)) = (&self.audio, self.encoder.as_mut()) else { return Ok(()) };
        if self.next.is_none() {
            audio.try_iter().for_each(drop);
            return Ok(());
        }
        let sent = audio.try_iter().try_for_each(|chunk| {
            let bytes: Vec<u8> = chunk.pcm.iter().flat_map(|s| s.to_le_bytes()).collect();
            encoder.send_audio_buffer(&bytes, 0)
        });
        sent.map_err(|e| {
            self.encoder = None;
            self.fail(e);
        })
    }

    fn pause(&mut self) {
        if self.paused.is_none() && self.flush().is_ok() {
            self.paused = Some(Instant::now());
        }
    }

    fn resume(&mut self) {
        let Some(at) = self.paused.take() else { return };
        self.discard();
        self.offset += (at.elapsed().as_nanos() / 100) as i64;
    }

    fn keep(&mut self, frame: &mut Frame) {
        let (x, y) = self.crop.map_or((0, 0), |(x, y, _, _)| (x, y));
        let w = frame.width().saturating_sub(x).min(self.size.0);
        let h = frame.height().saturating_sub(y).min(self.size.1);
        let Ok(buffer) = frame.buffer_crop(x, y, x + w, y + h) else { return };
        let mut raw = Vec::new();
        let px = buffer.as_nopadding_buffer(&mut raw).to_vec();
        self.last = (px, w, h);
    }

    fn thumbnail(&mut self) -> RgbaImage {
        let (mut px, w, h) = std::mem::take(&mut self.last);
        px.as_chunks_mut::<4>().0.iter_mut().for_each(|p| p.swap(0, 2));
        RgbaImage::from_raw(w, h, px).unwrap_or_default()
    }
}

impl GraphicsCaptureApiHandler for Handler {
    type Flags = Flags;
    type Error = String;

    fn new(ctx: Context<Flags>) -> Result<Self, String> {
        let (target, path, proxy, fps, quality, audio) = ctx.flags;
        let item: GraphicsCaptureItemType = match target {
            Target::Window(hwnd) => Window::from_raw_hwnd(hwnd as *mut c_void).try_into().map_err(|e| format!("{e}")),
            Target::Monitor(hmonitor) | Target::Area { monitor: hmonitor, .. } => {
                Monitor::from_raw_hmonitor(hmonitor as *mut c_void).try_into().map_err(|e| format!("{e}"))
            }
        }?;
        let crop = match target {
            Target::Area { rect, .. } => Some(rect),
            _ => None,
        };
        let (GraphicsCaptureItemType::Window((item, _)) | GraphicsCaptureItemType::Monitor((item, _))) = item else {
            return Err("unsupported capture target".into());
        };
        let size = item.Size().map_err(|e| e.to_string())?;
        let size = match crop {
            Some((_, _, w, h)) => (w & !1, h & !1),
            None => (size.Width.max(0) as u32 & !1, size.Height.max(0) as u32 & !1),
        };
        if size.0 == 0 || size.1 == 0 {
            return Err("window is too small".into());
        }
        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(size.0, size.1)
                .sub_type(VideoSettingsSubType::H264)
                .frame_rate(fps)
                .bitrate(video::bitrate(size.0, size.1, f64::from(fps), quality)),
            AudioSettingsBuilder::new().disabled(audio.is_none()),
            ContainerSettingsBuilder::new(),
            &path,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { encoder: Some(encoder), size, crop, interval: 10_000_000 / fps as i64, next: None, kept: None, last: (Vec::new(), 0, 0), path, proxy, audio, paused: None, offset: 0 })
    }

    fn on_frame_arrived(&mut self, frame: &mut Frame, control: InternalCaptureControl) -> Result<(), String> {
        if self.paused.is_some() {
            self.discard();
            return Ok(());
        }
        if self.encoder.is_none() {
            return Ok(());
        }
        let Ok(time) = frame.timestamp().map(|t| t.Duration) else { return Ok(()) };
        if self.next.is_some_and(|next| time < next - self.interval / 2) {
            return Ok(());
        }
        if self.next.is_none() {
            self.discard();
        }
        let Some(encoder) = self.encoder.as_mut() else { return Ok(()) };
        let (x, y) = self.crop.map_or((0, 0), |(x, y, _, _)| (x, y));
        let sent = encoder.send_frame_region(frame, x, y, time - self.offset).map_err(|e| e.to_string());
        if let Err(e) = sent {
            self.encoder = None;
            self.fail(e);
            control.stop();
            return Ok(());
        }
        self.next = Some(self.next.map_or(time, |next| next.max(time)) + self.interval);
        if self.flush().is_err() {
            control.stop();
            return Ok(());
        }
        if self.kept.is_none_or(|kept| time - kept >= 2_500_000) {
            self.kept = Some(time);
            self.keep(frame);
        }
        Ok(())
    }

    fn on_closed(&mut self) -> Result<(), String> {
        self.finalize();
        Ok(())
    }
}

pub(crate) struct Recorder(CaptureControl<Handler, String>);

impl Recorder {
    pub(crate) fn start(
        proxy: EventLoopProxy<Action>,
        target: Target,
        path: PathBuf,
        fps: u32,
        quality: Quality,
        audio: Option<Receiver<Chunk>>,
    ) -> Result<Self, String> {
        let flags = (target, path.clone(), proxy, fps, quality, audio);
        let started = match target {
            Target::Window(hwnd) => Handler::start_free_threaded(Settings::new(
                Window::from_raw_hwnd(hwnd as *mut c_void),
                CursorCaptureSettings::WithCursor,
                DrawBorderSettings::Default,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                flags,
            )),
            Target::Monitor(hmonitor) | Target::Area { monitor: hmonitor, .. } => Handler::start_free_threaded(Settings::new(
                Monitor::from_raw_hmonitor(hmonitor as *mut c_void),
                CursorCaptureSettings::WithCursor,
                DrawBorderSettings::Default,
                SecondaryWindowSettings::Default,
                MinimumUpdateIntervalSettings::Default,
                DirtyRegionSettings::Default,
                ColorFormat::Bgra8,
                flags,
            )),
        };
        started.map(Self).map_err(|e| {
            let _ = std::fs::remove_file(&path);
            e.to_string()
        })
    }

    pub(crate) fn pause(&self) {
        self.0.callback().lock().pause();
    }

    pub(crate) fn resume(&self) {
        self.0.callback().lock().resume();
    }

    pub(crate) fn paused(&self) -> bool {
        self.0.callback().lock().paused.is_some()
    }

    pub(crate) fn stop(self) {
        let handler = self.0.callback();
        let _ = self.0.stop();
        handler.lock().finalize();
    }
}
