use crate::{draw, toolbar};
use image::{RgbaImage, imageops};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::time::{Duration, Instant};
use tiny_skia::{Pixmap, Point};
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
const PIN_WIDTH: u32 = 26;
const PIN_HEIGHT: u32 = 22;
const PIN_MARGIN: u32 = 4;
const PIN_RADIUS: f32 = 6.0;
const PIN_ICON: f32 = 13.0;
const PIN_PANEL: [u8; 3] = [0x2b, 0x2b, 0x2b];
const PIN_HOVER: [u8; 3] = [0x3a, 0x3a, 0x3a];
const PIN_ICON_COLOUR: [u8; 3] = [0xff, 0xff, 0xff];
const PIN_GLYPH: char = '\u{E718}';

pub(crate) enum Outcome {
    Pending,
    Close,
    Pin,
}

pub struct Toast {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    pixels: Vec<u32>,
    path: PathBuf,
    secs: u32,
    deadline: Option<Instant>,
    pinnable: bool,
    hover: bool,
    pin_hover: bool,
    pin_rect: (i32, i32, i32, i32),
}

impl Toast {
    pub fn open(event_loop: &ActiveEventLoop, img: &RgbaImage, path: PathBuf, secs: u32, pinnable: bool) -> Option<Self> {
        let (w, h) = img.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        let ratio = (MAX.0 as f32 / w as f32).min(MAX.1 as f32 / h as f32).min(1.0);
        let tw = ((w as f32 * ratio) as u32).max(1);
        let th = ((h as f32 * ratio) as u32).max(1);
        let thumb = imageops::thumbnail(img, tw, th);
        let pinnable = pinnable && toolbar::icon_font().is_some();
        let mut ow = tw + 2 * EDGE;
        let mut oh = th + 2 * EDGE;
        if pinnable {
            ow = ow.max(PIN_WIDTH + 2 * PIN_MARGIN);
            oh = oh.max(PIN_HEIGHT + 2 * PIN_MARGIN);
        }
        let mut pixels = vec![BORDER; (ow * oh) as usize];
        for (x, y, p) in thumb.enumerate_pixels() {
            let a = p[3] as u32;
            let [r, g, b] = [p[0], p[1], p[2]].map(|c| (c as u32 * a + 0x20 * (255 - a)) / 255);
            pixels[((y + EDGE) * ow + x + EDGE) as usize] = r << 16 | g << 8 | b;
        }
        let pin_rect = (
            ow as i32 - PIN_MARGIN as i32 - PIN_WIDTH as i32,
            PIN_MARGIN as i32,
            PIN_WIDTH as i32,
            PIN_HEIGHT as i32,
        );
        let (px, py) = position(ow as i32, oh as i32);
        let attrs = Window::default_attributes()
            .with_title("Snapr")
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
            pinnable,
            hover: false,
            pin_hover: false,
            pin_rect,
        })
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub fn expired(&self) -> bool {
        self.deadline.is_some_and(|d| d <= Instant::now())
    }

    pub fn event(&mut self, event: WindowEvent) -> Outcome {
        match event {
            WindowEvent::CursorEntered { .. } => {
                self.hover = true;
                self.deadline = None;
                self.window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } => {
                self.hover = false;
                self.pin_hover = false;
                self.deadline = Some(Instant::now() + Duration::from_secs(self.secs as u64));
                self.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                let (x, y) = (position.x as i32, position.y as i32);
                let size = self.window.inner_size();
                let hover = x >= 0 && y >= 0 && x < size.width as i32 && y < size.height as i32;
                let pin_hover = hover && inside(self.pin_rect, x, y);
                let changed = hover != self.hover || pin_hover != self.pin_hover;
                self.hover = hover;
                self.pin_hover = pin_hover;
                self.deadline = if hover { None } else { Some(Instant::now() + Duration::from_secs(self.secs as u64)) };
                if changed {
                    self.window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let (x, y) = self.local_cursor();
                if self.pinnable && self.hover && inside(self.pin_rect, x, y) {
                    return Outcome::Pin;
                }
                let _ = std::process::Command::new("explorer").arg(&self.path).spawn();
                return Outcome::Close;
            }
            WindowEvent::RedrawRequested => self.draw(),
            _ => {}
        }
        Outcome::Pending
    }

    fn local_cursor(&self) -> (i32, i32) {
        let mut point = POINT { x: 0, y: 0 };
        let position = self.window.outer_position().map_or((0, 0), |p| (p.x, p.y));
        unsafe { GetCursorPos(&mut point) };
        (point.x - position.0, point.y - position.1)
    }

    fn draw(&mut self) {
        let size = self.window.inner_size();
        let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) else { return };
        if self.surface.resize(w, h).is_err() {
            return;
        }
        let frame = if self.pinnable && self.hover {
            self.compose().unwrap_or_else(|| self.pixels.clone())
        } else {
            self.pixels.clone()
        };
        if let Ok(mut buf) = self.surface.buffer_mut()
            && buf.len() == frame.len()
        {
            buf.copy_from_slice(&frame);
            let _ = buf.present();
        }
    }

    fn compose(&self) -> Option<Vec<u32>> {
        let (x, y, w, h) = self.pin_rect;
        if w <= 0 || h <= 0 {
            return None;
        }
        let mut pixmap = Pixmap::new(w as u32, h as u32)?;
        let colour = if self.pin_hover { PIN_HOVER } else { PIN_PANEL };
        toolbar::fill(&mut pixmap, (0.0, 0.0, w as f32, h as f32), PIN_RADIUS, colour, 0xe6);
        toolbar::icon(
            &mut pixmap,
            (PIN_GLYPH, 0.0),
            Point::from_xy(w as f32 / 2.0, h as f32 / 2.0),
            PIN_ICON,
            &draw::paint(PIN_ICON_COLOUR, 0xff),
        );
        let width = self.window.inner_size().width as i32;
        if width <= 0 {
            return None;
        }
        let mut frame = self.pixels.clone();
        for (index, pixel) in pixmap.pixels().iter().enumerate() {
            let alpha = pixel.alpha() as u32;
            if alpha == 0 {
                continue;
            }
            let bx = x + index as i32 % w;
            let by = y + index as i32 / w;
            if bx < 0 || by < 0 {
                continue;
            }
            let Some(dst) = frame.get_mut(by as usize * width as usize + bx as usize) else { continue };
            let r = pixel.red() as u32 + ((*dst >> 16 & 0xff) * (255 - alpha) / 255);
            let g = pixel.green() as u32 + ((*dst >> 8 & 0xff) * (255 - alpha) / 255);
            let b = pixel.blue() as u32 + ((*dst & 0xff) * (255 - alpha) / 255);
            *dst = (r.min(255) << 16) | (g.min(255) << 8) | b.min(255);
        }
        Some(frame)
    }
}

fn inside((x, y, w, h): (i32, i32, i32, i32), px: i32, py: i32) -> bool {
    px >= x && py >= y && px < x + w && py < y + h
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
