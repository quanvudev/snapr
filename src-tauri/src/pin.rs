use crate::overlay::Rect;
use crate::{draw, ring, toolbar};
use image::{RgbaImage, imageops};
use std::time::{Duration, Instant};
use tiny_skia::{Pixmap, PixmapPaint, Point, Transform};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint};
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{ReleaseCapture, SetCapture};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, HWND_TOPMOST, SW_HIDE, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE,
    SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER, SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_EX_LAYERED, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const MARGIN: i32 = 16;
const BUTTON: f32 = 26.0;
const GAP: f32 = 4.0;
const PAD: f32 = 4.0;
const TOOL_MARGIN: f32 = 6.0;
const RADIUS: f32 = 7.0;
const ICON: f32 = 14.0;
const BORDER: u32 = 2;
const BORDER_COLOUR: [u8; 3] = [0x30, 0x90, 0xf0];
const MAX_PIN: f32 = 250.0;
const PANEL: [u8; 3] = [0x2b, 0x2b, 0x2b];
const HOVER: [u8; 3] = [0x3a, 0x3a, 0x3a];
const WHITE: [u8; 3] = [0xff, 0xff, 0xff];
const COPY: char = '\u{E8C8}';
const CLOSE: char = '\u{E711}';
const CHECK: char = '\u{E73E}';
const CHECK_SECS: u64 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Button {
    Copy,
    Close,
}

pub(crate) enum Outcome {
    Pending,
    Close,
    Copy,
}

struct Toolbar {
    panel: (f32, f32, f32, f32),
    copy: (f32, f32, f32, f32),
    close: (f32, f32, f32, f32),
    radius: f32,
    icon: f32,
}

struct Pin {
    window: Window,
    hwnd: isize,
    image: RgbaImage,
    base: Pixmap,
    size: (u32, u32),
    toolbar: Toolbar,
    hover: bool,
    button: Option<Button>,
    drag: Option<(i32, i32)>,
    check_until: Option<Instant>,
    visible: bool,
}

impl Pin {
    fn open(event_loop: &ActiveEventLoop, image: RgbaImage, offset: (i32, i32)) -> Result<Self, String> {
        let (w, h) = image.dimensions();
        if w == 0 || h == 0 {
            return Err("empty image".into());
        }
        let ((cx, cy), (left, top, right, bottom), dpi_scale) = cursor_and_work().ok_or("no monitor")?;
        let (cx, cy) = (cx + offset.0, cy + offset.1);
        let cap = (MAX_PIN * dpi_scale)
            .min((right - left - 2 * MARGIN).max(1) as f32)
            .min((bottom - top - 2 * MARGIN).max(1) as f32);
        let ratio = (cap / w.max(h) as f32).min(1.0);
        let dw = ((w as f32 * ratio).round() as u32).max(1);
        let dh = ((h as f32 * ratio).round() as u32).max(1);
        let source = if dw == w && dh == h {
            ring::bitmap(&image)
        } else {
            let scaled = imageops::thumbnail(&image, dw, dh);
            ring::bitmap(&scaled)
        }
        .ok_or("pin surface failed")?;
        let attrs = Window::default_attributes()
            .with_title("Snapr")
            .with_decorations(false)
            .with_undecorated_shadow(false)
            .with_resizable(false)
            .with_active(false)
            .with_visible(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(0, 0))
            .with_inner_size(PhysicalSize::new(dw, dh))
            .with_skip_taskbar(true);
        let window = event_loop.create_window(attrs).map_err(|e| e.to_string())?;
        let scale = window.scale_factor() as f32;
        let button = (BUTTON * scale).round().max(1.0);
        let gap = (GAP * scale).round().max(1.0);
        let pad = (PAD * scale).round().max(1.0);
        let margin = (TOOL_MARGIN * scale).round().max(1.0);
        let toolbar_w = 2.0 * button + gap + 2.0 * pad;
        let toolbar_h = button + 2.0 * pad;
        let win_w = (dw + 2 * BORDER).max((toolbar_w + 2.0 * margin).ceil() as u32).max(1);
        let win_h = (dh + 2 * BORDER).max((toolbar_h + 2.0 * margin).ceil() as u32).max(1);
        let _ = window.request_inner_size(PhysicalSize::new(win_w, win_h));
        let x = (cx - dw as i32 / 2 - BORDER as i32).clamp(left, (right - win_w as i32).max(left));
        let y = (cy - dh as i32 / 2 - BORDER as i32).clamp(top, (bottom - win_h as i32).max(top));
        window.set_outer_position(PhysicalPosition::new(x, y));
        let Ok(RawWindowHandle::Win32(handle)) = window.window_handle().map(|h| h.as_raw()) else {
            return Err("pin window failed".into());
        };
        let hwnd = handle.hwnd.get();
        unsafe {
            let style = GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE);
            SetWindowLongPtrW(
                hwnd as _,
                GWL_EXSTYLE,
                style | (WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE) as isize,
            );
            SetWindowPos(
                hwnd as _,
                std::ptr::null_mut(),
                0,
                0,
                0,
                0,
                SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE,
            );
        }
        let mut base = Pixmap::new(win_w, win_h).ok_or("pin surface failed")?;
        let border = BORDER as f32;
        let frame_w = dw as f32 + 2.0 * border;
        toolbar::fill(&mut base, (0.0, 0.0, frame_w, border), 0.0, BORDER_COLOUR, 0xff);
        toolbar::fill(&mut base, (0.0, border + dh as f32, frame_w, border), 0.0, BORDER_COLOUR, 0xff);
        toolbar::fill(&mut base, (0.0, border, border, dh as f32), 0.0, BORDER_COLOUR, 0xff);
        toolbar::fill(&mut base, (border + dw as f32, border, border, dh as f32), 0.0, BORDER_COLOUR, 0xff);
        base.draw_pixmap(
            BORDER as i32,
            BORDER as i32,
            source.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
        let panel = (win_w as f32 - margin - toolbar_w, margin, toolbar_w, toolbar_h);
        let copy = (panel.0 + pad, panel.1 + pad, button, button);
        let close = (copy.0 + button + gap, copy.1, button, button);
        let toolbar = Toolbar { panel, copy, close, radius: (RADIUS * scale).max(1.0), icon: (ICON * scale).max(1.0) };
        window.set_cursor(CursorIcon::Default);
        let mut pin = Self {
            window,
            hwnd,
            image,
            base,
            size: (win_w, win_h),
            toolbar,
            hover: false,
            button: None,
            drag: None,
            check_until: None,
            visible: false,
        };
        pin.draw()?;
        Ok(pin)
    }

    fn id(&self) -> WindowId {
        self.window.id()
    }

    fn image(&self) -> &RgbaImage {
        &self.image
    }

    fn deadline(&self) -> Option<Instant> {
        self.check_until
    }

    fn copied(&mut self) {
        self.check_until = Some(Instant::now() + Duration::from_secs(CHECK_SECS));
        self.window.request_redraw();
    }

    fn tick(&mut self) {
        if self.check_until.is_some_and(|deadline| deadline <= Instant::now()) {
            self.check_until = None;
            self.window.request_redraw();
        }
    }

    fn set_visible(&mut self, visible: bool) {
        if self.visible == visible {
            return;
        }
        if !visible {
            if self.drag.take().is_some() {
                unsafe { ReleaseCapture() };
            }
            self.hover = false;
            self.button = None;
        }
        unsafe { ShowWindow(self.hwnd as _, if visible { SW_SHOWNOACTIVATE } else { SW_HIDE }) };
        self.visible = visible;
        if visible {
            self.window.request_redraw();
        }
    }

    fn event(&mut self, event: WindowEvent) -> Outcome {
        match event {
            WindowEvent::CursorEntered { .. } => {
                self.hover = true;
                self.window.request_redraw();
            }
            WindowEvent::CursorLeft { .. } if self.drag.is_none() => {
                self.hover = false;
                self.button = None;
                self.window.request_redraw();
            }
            WindowEvent::CursorMoved { position, .. } => {
                if let Some((dx, dy)) = self.drag {
                    if let Some((cx, cy)) = cursor_pos() {
                        self.window.set_outer_position(PhysicalPosition::new(cx - dx, cy - dy));
                    }
                } else {
                    let local = (position.x as i32, position.y as i32);
                    let hover = self.inside(local);
                    let button = if hover { self.hit(local) } else { None };
                    if hover != self.hover || button != self.button {
                        self.hover = hover;
                        self.button = button;
                        self.window.request_redraw();
                    }
                    self.window.set_cursor(if button.is_some() { CursorIcon::Pointer } else { CursorIcon::Default });
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let local = self.local_cursor();
                if self.hover
                    && let Some(button) = self.hit(local)
                {
                    return match button {
                        Button::Copy => Outcome::Copy,
                        Button::Close => Outcome::Close,
                    };
                }
                if let (Some((cx, cy)), Ok(position)) = (cursor_pos(), self.window.outer_position()) {
                    self.drag = Some((cx - position.x, cy - position.y));
                    unsafe { SetCapture(self.hwnd as _) };
                    self.hover = false;
                    self.button = None;
                    self.raise();
                    self.window.set_cursor(CursorIcon::Grabbing);
                    self.window.request_redraw();
                }
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if self.drag.take().is_some() {
                    unsafe { ReleaseCapture() };
                    let local = self.local_cursor();
                    self.hover = self.inside(local);
                    self.button = self.hit(local);
                    self.window.set_cursor(if self.button.is_some() { CursorIcon::Pointer } else { CursorIcon::Default });
                    self.window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let _ = self.draw();
            }
            WindowEvent::CloseRequested => return Outcome::Close,
            _ => {}
        }
        Outcome::Pending
    }

    fn local_cursor(&self) -> (i32, i32) {
        let Some((x, y)) = cursor_pos() else { return (0, 0) };
        let position = self.window.outer_position().map_or((0, 0), |p| (p.x, p.y));
        (x - position.0, y - position.1)
    }

    fn inside(&self, (x, y): (i32, i32)) -> bool {
        x >= 0 && y >= 0 && x < self.size.0 as i32 && y < self.size.1 as i32
    }

    fn hit(&self, (x, y): (i32, i32)) -> Option<Button> {
        let (x, y) = (x as f32, y as f32);
        let inside = |(rx, ry, rw, rh): (f32, f32, f32, f32)| x >= rx && x <= rx + rw && y >= ry && y <= ry + rh;
        if inside(self.toolbar.close) {
            Some(Button::Close)
        } else if inside(self.toolbar.copy) {
            Some(Button::Copy)
        } else {
            None
        }
    }

    fn raise(&self) {
        unsafe {
            SetWindowPos(
                self.hwnd as _,
                HWND_TOPMOST as _,
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            );
        }
    }

    fn draw(&mut self) -> Result<(), String> {
        if !self.hover {
            return ring::present(self.hwnd, &self.base);
        }
        let mut pixmap = Pixmap::new(self.size.0, self.size.1).ok_or("pin surface failed")?;
        pixmap.draw_pixmap(0, 0, self.base.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        self.paint_toolbar(&mut pixmap);
        ring::present(self.hwnd, &pixmap)
    }

    fn paint_toolbar(&self, pixmap: &mut Pixmap) {
        toolbar::fill(pixmap, self.toolbar.panel, self.toolbar.radius, PANEL, 0xe6);
        let check = self.check_until.is_some_and(|deadline| deadline > Instant::now());
        for (button, rect, glyph) in [(Button::Copy, self.toolbar.copy, if check { CHECK } else { COPY }), (Button::Close, self.toolbar.close, CLOSE)] {
            if self.button == Some(button) {
                toolbar::fill(pixmap, rect, self.toolbar.radius * 0.7, HOVER, 0xff);
            }
            let centre = Point::from_xy(rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0);
            toolbar::icon(pixmap, (glyph, 0.0), centre, self.toolbar.icon, &draw::paint(WHITE, 0xff));
        }
    }
}

impl Drop for Pin {
    fn drop(&mut self) {
        if self.drag.take().is_some() {
            unsafe { ReleaseCapture() };
        }
    }
}

#[derive(Default)]
pub(crate) struct Pins {
    items: Vec<Pin>,
    capture: bool,
    recording: bool,
}

impl Pins {
    pub(crate) fn add(&mut self, event_loop: &ActiveEventLoop, image: RgbaImage) -> Result<(), String> {
        self.add_at(event_loop, image, (0, 0))
    }

    pub(crate) fn add_at(&mut self, event_loop: &ActiveEventLoop, image: RgbaImage, offset: (i32, i32)) -> Result<(), String> {
        let mut pin = Pin::open(event_loop, image, offset)?;
        let visible = self.visible();
        pin.set_visible(visible);
        if visible {
            pin.raise();
        }
        self.items.push(pin);
        Ok(())
    }

    pub(crate) fn contains(&self, id: WindowId) -> bool {
        self.items.iter().any(|pin| pin.id() == id)
    }

    pub(crate) fn event(&mut self, id: WindowId, event: WindowEvent) -> Outcome {
        self.items.iter_mut().find(|pin| pin.id() == id).map_or(Outcome::Pending, |pin| pin.event(event))
    }

    pub(crate) fn remove(&mut self, id: WindowId) {
        self.items.retain(|pin| pin.id() != id);
    }

    pub(crate) fn image(&self, id: WindowId) -> Option<&RgbaImage> {
        self.items.iter().find(|pin| pin.id() == id).map(Pin::image)
    }

    pub(crate) fn copied(&mut self, id: WindowId) {
        if let Some(pin) = self.items.iter_mut().find(|pin| pin.id() == id) {
            pin.copied();
        }
    }

    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.items.iter().filter_map(Pin::deadline).min()
    }

    pub(crate) fn tick(&mut self) {
        for pin in &mut self.items {
            pin.tick();
        }
    }

    pub(crate) fn hide_capture(&mut self) {
        self.capture = true;
        self.apply();
    }

    pub(crate) fn show_capture(&mut self) {
        self.capture = false;
        self.apply();
    }

    pub(crate) fn hide_recording(&mut self) {
        self.recording = true;
        self.apply();
    }

    pub(crate) fn show_recording(&mut self) {
        self.recording = false;
        self.apply();
    }

    fn visible(&self) -> bool {
        !self.capture && !self.recording
    }

    fn apply(&mut self) {
        let visible = self.visible();
        for pin in &mut self.items {
            pin.set_visible(visible);
        }
    }
}

fn cursor_pos() -> Option<(i32, i32)> {
    let mut point = POINT { x: 0, y: 0 };
    (unsafe { GetCursorPos(&mut point) } != 0).then_some((point.x, point.y))
}

fn cursor_and_work() -> Option<((i32, i32), Rect, f32)> {
    let mut point = POINT { x: 0, y: 0 };
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    let monitor = unsafe {
        GetCursorPos(&mut point);
        MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST)
    };
    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
    let mut dpi_x = 96;
    let mut dpi_y = 96;
    let dpi_ok = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) } >= 0;
    let scale = if dpi_ok { dpi_x as f32 / 96.0 } else { 1.0 };
    let work = info.rcWork;
    (ok != 0).then_some(((point.x, point.y), (work.left, work.top, work.right, work.bottom), scale))
}
