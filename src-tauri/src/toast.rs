use image::{RgbaImage, imageops};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::{Duration, Instant};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, SetWindowDisplayAffinity, SetWindowLongPtrW, WDA_EXCLUDEFROMCAPTURE,
    WS_EX_NOACTIVATE,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const BORDER: u32 = 0x3090f0;
const EDGE: u32 = 2;
const MARGIN: i32 = 16;
const MAX: (u32, u32) = (320, 200);

pub struct Toast {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    pixels: Vec<u32>,
    path: PathBuf,
    secs: u32,
    deadline: Option<Instant>,
}

impl Toast {
    pub fn open(event_loop: &ActiveEventLoop, img: &RgbaImage, path: PathBuf, secs: u32) -> Option<Self> {
        let (w, h) = img.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        let ratio = (MAX.0 as f32 / w as f32).min(MAX.1 as f32 / h as f32).min(1.0);
        let tw = ((w as f32 * ratio) as u32).max(1);
        let th = ((h as f32 * ratio) as u32).max(1);
        let thumb = imageops::thumbnail(img, tw, th);
        let (ow, oh) = (tw + 2 * EDGE, th + 2 * EDGE);
        let mut pixels = vec![BORDER; (ow * oh) as usize];
        for (x, y, p) in thumb.enumerate_pixels() {
            let a = p[3] as u32;
            let [r, g, b] = [p[0], p[1], p[2]].map(|c| (c as u32 * a + 0x20 * (255 - a)) / 255);
            pixels[((y + EDGE) * ow + x + EDGE) as usize] = r << 16 | g << 8 | b;
        }
        let (px, py) = position(ow as i32, oh as i32);
        let attrs = Window::default_attributes()
            .with_title("snapr")
            .with_decorations(false)
            .with_resizable(false)
            .with_active(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(px, py))
            .with_inner_size(PhysicalSize::new(ow, oh))
            .with_skip_taskbar(true);
        let window = Rc::new(event_loop.create_window(attrs).ok()?);
        let _ = window.request_inner_size(PhysicalSize::new(ow, oh));
        window.set_outer_position(PhysicalPosition::new(px, py));
        let context = Context::new(window.clone()).ok()?;
        let surface = Surface::new(&context, window.clone()).ok()?;
        window.set_cursor(CursorIcon::Pointer);
        if let Ok(handle) = window.window_handle()
            && let RawWindowHandle::Win32(h) = handle.as_raw()
        {
            let hwnd = h.hwnd.get() as _;
            unsafe {
                let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
                SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize);
                SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            }
        }
        window.request_redraw();
        Some(Self {
            window,
            surface,
            pixels,
            path,
            secs,
            deadline: Some(Instant::now() + Duration::from_secs(secs as u64)),
        })
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn expired(&self) -> bool {
        self.deadline.is_some_and(|d| d <= Instant::now())
    }

    pub fn event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::CursorEntered { .. } => self.deadline = None,
            WindowEvent::CursorLeft { .. } => {
                self.deadline = Some(Instant::now() + Duration::from_secs(self.secs as u64));
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let _ = std::process::Command::new("explorer").arg(&self.path).spawn();
                return true;
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
        false
    }

    fn draw(&mut self) {
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        if self.surface.resize(w, h).is_err() {
            return;
        }
        if let Ok(mut buf) = self.surface.buffer_mut()
            && buf.len() == self.pixels.len()
        {
            buf.copy_from_slice(&self.pixels);
            let _ = buf.present();
        }
    }
}

fn position(w: i32, h: i32) -> (i32, i32) {
    let mut p = POINT { x: 0, y: 0 };
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    let ok = unsafe {
        GetCursorPos(&mut p);
        GetMonitorInfoW(MonitorFromPoint(p, MONITOR_DEFAULTTOPRIMARY), &mut info)
    };
    if ok == 0 {
        return (MARGIN, MARGIN);
    }
    (info.rcWork.right - w - MARGIN, info.rcWork.bottom - h - MARGIN)
}
