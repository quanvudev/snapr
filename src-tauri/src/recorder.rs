use crate::Action;
use image::RgbaImage;
use std::ffi::c_void;
use std::path::PathBuf;
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
}

type Flags = (Target, PathBuf, EventLoopProxy<Action>, u32);

struct Handler {
    encoder: Option<VideoEncoder>,
    size: (u32, u32),
    interval: i64,
    next: Option<i64>,
    kept: Option<i64>,
    last: RgbaImage,
    path: PathBuf,
    proxy: EventLoopProxy<Action>,
}

impl Handler {
    fn finalize(&mut self) {
        let Some(encoder) = self.encoder.take() else { return };
        match encoder.finish() {
            Ok(()) => {
                let _ = self.proxy.send_event(Action::Recorded(std::mem::take(&mut self.last), self.path.clone()));
            }
            Err(e) => self.fail(e),
        }
    }

    fn fail(&self, e: impl std::fmt::Display) {
        let _ = self.proxy.send_event(Action::RecordFailed(e.to_string()));
    }

    fn keep(&mut self, frame: &mut Frame) {
        let (w, h) = (frame.width().min(self.size.0), frame.height().min(self.size.1));
        let Ok(buffer) = frame.buffer_crop(0, 0, w, h) else { return };
        let mut raw = Vec::new();
        let mut px = buffer.as_nopadding_buffer(&mut raw).to_vec();
        px.as_chunks_mut::<4>().0.iter_mut().for_each(|p| p.swap(0, 2));
        if let Some(img) = RgbaImage::from_raw(w, h, px) {
            self.last = img;
        }
    }
}

impl GraphicsCaptureApiHandler for Handler {
    type Flags = Flags;
    type Error = String;

    fn new(ctx: Context<Flags>) -> Result<Self, String> {
        let (target, path, proxy, fps) = ctx.flags;
        let item: GraphicsCaptureItemType = match target {
            Target::Window(hwnd) => Window::from_raw_hwnd(hwnd as *mut c_void).try_into().map_err(|e| format!("{e}")),
            Target::Monitor(hmonitor) => {
                Monitor::from_raw_hmonitor(hmonitor as *mut c_void).try_into().map_err(|e| format!("{e}"))
            }
        }?;
        let (GraphicsCaptureItemType::Window((item, _)) | GraphicsCaptureItemType::Monitor((item, _))) = item else {
            return Err("unsupported capture target".into());
        };
        let size = item.Size().map_err(|e| e.to_string())?;
        let size = (size.Width.max(0) as u32 & !1, size.Height.max(0) as u32 & !1);
        if size.0 == 0 || size.1 == 0 {
            return Err("window is too small".into());
        }
        let encoder = VideoEncoder::new(
            VideoSettingsBuilder::new(size.0, size.1).sub_type(VideoSettingsSubType::H264).frame_rate(fps),
            AudioSettingsBuilder::new().disabled(true),
            ContainerSettingsBuilder::new(),
            &path,
        )
        .map_err(|e| e.to_string())?;
        Ok(Self { encoder: Some(encoder), size, interval: 10_000_000 / fps as i64, next: None, kept: None, last: RgbaImage::new(0, 0), path, proxy })
    }

    fn on_frame_arrived(&mut self, frame: &mut Frame, control: InternalCaptureControl) -> Result<(), String> {
        let Some(encoder) = self.encoder.as_mut() else { return Ok(()) };
        let Ok(time) = frame.timestamp().map(|t| t.Duration) else { return Ok(()) };
        if self.next.is_some_and(|next| time < next - self.interval / 2) {
            return Ok(());
        }
        if let Err(e) = encoder.send_frame(frame) {
            self.encoder = None;
            self.fail(e);
            control.stop();
            return Ok(());
        }
        self.next = Some(self.next.map_or(time, |next| next.max(time)) + self.interval);
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
    pub(crate) fn start(proxy: EventLoopProxy<Action>, target: Target, path: PathBuf, fps: u32) -> Result<Self, String> {
        let flags = (target, path.clone(), proxy, fps);
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
            Target::Monitor(hmonitor) => Handler::start_free_threaded(Settings::new(
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

    pub(crate) fn stop(self) {
        let handler = self.0.callback();
        let _ = self.0.stop();
        handler.lock().finalize();
    }
}
