use crate::draw;
use crate::overlay::Rect;
use crate::toolbar;
use std::f32::consts::TAU;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tiny_skia::{FillRule, LineCap, PathBuilder, Pixmap, Point, Stroke, Transform};
use windows_sys::Win32::Foundation::{GetLastError, POINT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
    ReleaseDC, SelectObject,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, ULW_ALPHA, UpdateLayeredWindow, WS_EX_LAYERED, WS_EX_TOOLWINDOW,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const ORBIT: f32 = 90.0;
const BUTTON_RADIUS: f32 = 24.0;
const HOVER_SCALE: f32 = 1.15;
const ICON_SIZE: f32 = 22.0;
const CARD_GAP: f32 = 8.0;
const CARD_RADIUS: f32 = 6.0;
const CARD_PAD_X: f32 = 14.0;
const CARD_PAD_Y: f32 = 8.0;
const LABEL_SIZE: f32 = 14.0;
const SHADOW_SPREAD: f32 = 6.0;
const SHADOW_OFFSET: f32 = 2.0;
const SHADOW_LAYERS: u32 = 4;
const SHADOW_ALPHA: u8 = 18;
const SHADOW: [u8; 3] = [0x00, 0x00, 0x00];
const WINDOW_MARGIN: f32 = SHADOW_SPREAD + 10.0;
const CLOSE_RADIUS: f32 = 15.0;
const CLOSE_ARM: f32 = 3.5;
const CLOSE_WIDTH: f32 = 1.5;
const CLOSE_FILL: [u8; 3] = [0xee, 0xee, 0xee];
const CLOSE_MARK: [u8; 3] = [0x30, 0x30, 0x30];
const BUTTON_FILL: [u8; 3] = [0x00, 0x00, 0x00];
const ICON_COLOUR: [u8; 3] = [0xff, 0xff, 0xff];
const CARD_FILL: [u8; 3] = [0xff, 0xff, 0xff];
const CARD_TEXT: [u8; 3] = [0x20, 0x20, 0x20];
const OPAQUE: u8 = 0xff;
const FOCUS_GRACE: Duration = Duration::from_millis(300);
const FRAME_INTERVAL: Duration = Duration::from_millis(16);
const OPEN_DURATION: Duration = Duration::from_millis(150);
const HOVER_DURATION: Duration = Duration::from_millis(100);
const MAX_STEP: f32 = 0.05;
const OPEN_START_SCALE: f32 = 0.5;
const CARD_SLIDE: f32 = 6.0;
const BITS_PER_PIXEL: u16 = 32;
const STOP_RECORDING: (&str, &str, char) = ("Stop recording", "Dừng quay", '\u{E71A}');

#[derive(Clone, Copy, PartialEq)]
pub enum Wedge {
    Fullscreen,
    Window,
    Area,
    Text,
    Record,
    Draw,
    Gallery,
    Settings,
}

pub enum Outcome {
    Pending,
    Cancel,
    Pick(Wedge),
}

const WEDGES: [(Wedge, &str, &str, char); 8] = [
    (Wedge::Fullscreen, "Fullscreen", "Toàn màn hình", '\u{E740}'),
    (Wedge::Window, "Window", "Cửa sổ", '\u{E737}'),
    (Wedge::Area, "Area", "Vùng chọn", '\u{EF20}'),
    (Wedge::Text, "Text", "Văn bản", '\u{E8E9}'),
    (Wedge::Record, "Record", "Quay video", '\u{E7C8}'),
    (Wedge::Draw, "Draw", "Vẽ", '\u{E70F}'),
    (Wedge::Gallery, "Gallery", "Thư viện", '\u{E8B9}'),
    (Wedge::Settings, "Settings", "Cài đặt", '\u{E713}'),
];

type Card = (f32, f32, f32, f32);

struct Item {
    label: &'static str,
    glyph: char,
    button: Point,
    card: Card,
    dir: Point,
}

impl Item {
    fn reach(&self, scale: f32) -> f32 {
        let (x, y, w, h) = self.card;
        let button = self.button.x.abs().max(self.button.y.abs()) + BUTTON_RADIUS * HOVER_SCALE * scale;
        [x.abs(), y.abs(), (x + w).abs(), (y + h).abs()].into_iter().fold(button, f32::max)
    }
}

pub struct Ring {
    window: Rc<Window>,
    hwnd: isize,
    wedges: Vec<Wedge>,
    items: Vec<Item>,
    scale: f32,
    side: u32,
    centre: f32,
    hover: Option<usize>,
    glow: Vec<f32>,
    opened: Instant,
    frame: Instant,
}

impl Ring {
    pub fn open(event_loop: &ActiveEventLoop, wedges: Vec<Wedge>, recording: bool, vi: bool) -> Option<Self> {
        if wedges.is_empty() {
            return None;
        }
        let (cursor, monitor) = cursor_and_monitor()?;
        let attrs = Window::default_attributes()
            .with_title("Snapr")
            .with_decorations(false)
            .with_undecorated_shadow(false)
            .with_resizable(false)
            .with_visible(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(cursor.0, cursor.1))
            .with_skip_taskbar(true);
        let window = Rc::new(event_loop.create_window(attrs).ok()?);
        let scale = window.scale_factor() as f32;
        let items = layout(&wedges, recording, vi, scale);
        let reach = items.iter().map(|item| item.reach(scale)).fold(0.0, f32::max);
        let side = (2.0 * (reach + WINDOW_MARGIN * scale)).ceil() as i32;
        let (x, y) = (place(cursor.0, side, monitor.0, monitor.2), place(cursor.1, side, monitor.1, monitor.3));
        let _ = window.request_inner_size(PhysicalSize::new(side as u32, side as u32));
        window.set_outer_position(PhysicalPosition::new(x, y));
        let Ok(RawWindowHandle::Win32(handle)) = window.window_handle().map(|h| h.as_raw()) else { return None };
        let hwnd = handle.hwnd.get();
        unsafe {
            let style = GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd as _, GWL_EXSTYLE, style | (WS_EX_LAYERED | WS_EX_TOOLWINDOW) as isize);
            SetWindowPos(hwnd as _, std::ptr::null_mut(), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_NOACTIVATE | SWP_FRAMECHANGED);
        }
        let glow = vec![0.0; items.len()];
        let now = Instant::now();
        let mut ring = Self {
            window,
            hwnd,
            wedges,
            items,
            scale,
            side: side as u32,
            centre: side as f32 / 2.0,
            hover: None,
            glow,
            opened: now,
            frame: now,
        };
        ring.draw();
        unsafe {
            ShowWindow(hwnd as _, SW_SHOW);
            SetForegroundWindow(hwnd as _);
        }
        Some(ring)
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn event(&mut self, event: WindowEvent) -> Outcome {
        match event {
            WindowEvent::CloseRequested => Outcome::Cancel,
            WindowEvent::Focused(false) if self.opened.elapsed() < FOCUS_GRACE => Outcome::Pending,
            WindowEvent::Focused(false) => Outcome::Cancel,
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                Outcome::Cancel
            }
            WindowEvent::CursorMoved { position, .. } => {
                let hover = self.hit((position.x as f32, position.y as f32));
                self.set_hover(hover);
                Outcome::Pending
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let hover = self.cursor_hit();
                self.set_hover(hover);
                self.hover.map_or(Outcome::Cancel, |i| Outcome::Pick(self.wedges[i]))
            }
            WindowEvent::RedrawRequested => {
                self.draw();
                Outcome::Pending
            }
            _ => Outcome::Pending,
        }
    }

    pub fn deadline(&self) -> Instant {
        self.frame + FRAME_INTERVAL
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        let step = (now - self.frame).as_secs_f32().min(MAX_STEP) / HOVER_DURATION.as_secs_f32();
        self.frame = now;
        let hover = self.cursor_hit();
        let changed = self.set_hover_state(hover);
        let mut moving = self.opened.elapsed() < OPEN_DURATION;
        for (i, glow) in self.glow.iter_mut().enumerate() {
            let target = if self.hover == Some(i) { 1.0 } else { 0.0 };
            let next = if *glow < target { (*glow + step).min(target) } else { (*glow - step).max(target) };
            moving |= next != *glow;
            *glow = next;
        }
        if moving || changed {
            self.draw();
        }
    }

    fn cursor_hit(&self) -> Option<usize> {
        let mut p = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut p) };
        let origin = self.window.outer_position().ok()?;
        self.hit(((p.x - origin.x) as f32, (p.y - origin.y) as f32))
    }

    fn set_hover(&mut self, hover: Option<usize>) {
        if self.set_hover_state(hover) {
            self.draw();
        }
    }

    fn set_hover_state(&mut self, hover: Option<usize>) -> bool {
        if hover == self.hover {
            return false;
        }
        self.hover = hover;
        self.window.set_cursor(if hover.is_some() { CursorIcon::Pointer } else { CursorIcon::Default });
        true
    }

    fn hit(&self, (x, y): (f32, f32)) -> Option<usize> {
        let (x, y) = (x - self.centre, y - self.centre);
        let reach = BUTTON_RADIUS * self.scale;
        let on_card = |item: &Item| {
            let (cx, cy, w, h) = item.card;
            x >= cx && x <= cx + w && y >= cy && y <= cy + h
        };
        self.hover
            .filter(|&i| on_card(&self.items[i]))
            .or_else(|| self.items.iter().position(|item| (x - item.button.x).hypot(y - item.button.y) <= reach))
    }

    fn draw(&mut self) {
        let Some(mut pixmap) = Pixmap::new(self.side, self.side) else { return };
        let scale = self.scale;
        let face = draw::font();
        let origin = Point::from_xy(self.centre, self.centre);
        let open = ease_out((self.opened.elapsed().as_secs_f32() / OPEN_DURATION.as_secs_f32()).min(1.0));
        let opening = OPEN_START_SCALE + (1.0 - OPEN_START_SCALE) * open;
        let alpha = (open * OPAQUE as f32) as u8;
        for (item, &glow) in self.items.iter().zip(&self.glow) {
            let lift = ease_out(glow);
            if lift > 0.0 && let Some(face) = face {
                let shift = -(1.0 - lift) * CARD_SLIDE * scale;
                let from = Point::from_xy(origin.x + item.dir.x * shift, origin.y + item.dir.y * shift);
                paint_card(&mut pixmap, face, item, from, scale, lift);
            }
            let grow = opening * (1.0 + (HOVER_SCALE - 1.0) * lift);
            let at = Point::from_xy(origin.x + item.button.x * open, origin.y + item.button.y * open);
            if let Some(disc) = PathBuilder::from_circle(at.x, at.y, BUTTON_RADIUS * scale * grow) {
                pixmap.fill_path(&disc, &draw::paint(BUTTON_FILL, alpha), FillRule::Winding, Transform::identity(), None);
            }
            toolbar::icon(&mut pixmap, (item.glyph, 0.0), at, ICON_SIZE * scale * grow, &draw::paint(ICON_COLOUR, alpha));
        }
        paint_close(&mut pixmap, origin, scale * opening, alpha);
        let _ = present(self.hwnd, &pixmap);
    }
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn mix(from: [u8; 3], to: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|i| (from[i] as f32 + (to[i] as f32 - from[i] as f32) * t).round() as u8)
}

fn layout(wedges: &[Wedge], recording: bool, vi: bool, scale: f32) -> Vec<Item> {
    let face = draw::font();
    let step = TAU / wedges.len() as f32;
    wedges
        .iter()
        .enumerate()
        .map(|(i, &wedge)| {
            let (label, glyph) = entry(wedge, recording, vi);
            let angle = i as f32 * step;
            let (dx, dy) = (angle.sin(), -angle.cos());
            let button = Point::from_xy(dx * ORBIT * scale, dy * ORBIT * scale);
            let (tw, th) = face.map_or((0.0, 0.0), |f| draw::measure(f, LABEL_SIZE * scale, label));
            let (w, h) = (tw + 2.0 * CARD_PAD_X * scale, th + 2.0 * CARD_PAD_Y * scale);
            let anchor = (BUTTON_RADIUS + CARD_GAP) * scale;
            let (px, py) = (button.x + dx * anchor, button.y + dy * anchor);
            let card = (px + dx * w / 2.0 - w / 2.0, py + dy * h / 2.0 - h / 2.0, w, h);
            Item { label, glyph, button, card, dir: Point::from_xy(dx, dy) }
        })
        .collect()
}

fn entry(wedge: Wedge, recording: bool, vi: bool) -> (&'static str, char) {
    let found = WEDGES.iter().find(|(w, ..)| *w == wedge).copied().unwrap_or(WEDGES[0]);
    let (_, en, vn, glyph) = if recording && wedge == Wedge::Record {
        (wedge, STOP_RECORDING.0, STOP_RECORDING.1, STOP_RECORDING.2)
    } else {
        found
    };
    (if vi { vn } else { en }, glyph)
}

fn paint_card(pixmap: &mut Pixmap, face: &ab_glyph::FontVec, item: &Item, origin: Point, scale: f32, fade: f32) {
    let (x, y, w, h) = item.card;
    let (x, y) = (origin.x + x, origin.y + y);
    for layer in 0..SHADOW_LAYERS {
        let grow = SHADOW_SPREAD * scale * (SHADOW_LAYERS - layer) as f32 / SHADOW_LAYERS as f32;
        let rect = (x - grow, y - grow + SHADOW_OFFSET * scale, w + grow * 2.0, h + grow * 2.0);
        toolbar::fill(pixmap, rect, CARD_RADIUS * scale + grow, SHADOW, (SHADOW_ALPHA as f32 * fade) as u8);
    }
    let alpha = (fade * OPAQUE as f32) as u8;
    toolbar::fill(pixmap, (x, y, w, h), CARD_RADIUS * scale, CARD_FILL, alpha);
    let (tw, th) = draw::measure(face, LABEL_SIZE * scale, item.label);
    let at = Point::from_xy(x + (w - tw) / 2.0, y + (h - th) / 2.0);
    draw::text(pixmap, face, at, &[item.label.to_string()], LABEL_SIZE * scale, mix(CARD_FILL, CARD_TEXT, fade), None);
}

fn paint_close(pixmap: &mut Pixmap, c: Point, scale: f32, alpha: u8) {
    if let Some(disc) = PathBuilder::from_circle(c.x, c.y, CLOSE_RADIUS * scale) {
        pixmap.fill_path(&disc, &draw::paint(CLOSE_FILL, alpha), FillRule::Winding, Transform::identity(), None);
    }
    let arm = CLOSE_ARM * scale;
    let mut cross = PathBuilder::new();
    cross.move_to(c.x - arm, c.y - arm);
    cross.line_to(c.x + arm, c.y + arm);
    cross.move_to(c.x + arm, c.y - arm);
    cross.line_to(c.x - arm, c.y + arm);
    if let Some(path) = cross.finish() {
        let stroke = Stroke { width: CLOSE_WIDTH * scale, line_cap: LineCap::Round, ..Stroke::default() };
        pixmap.stroke_path(&path, &draw::paint(CLOSE_MARK, alpha), &stroke, Transform::identity(), None);
    }
}

fn place(cursor: i32, side: i32, low: i32, high: i32) -> i32 {
    (cursor - side / 2).min(high - side).max(low)
}

fn present(hwnd: isize, pixmap: &Pixmap) -> Result<(), String> {
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    let mut header: BITMAPINFOHEADER = unsafe { std::mem::zeroed() };
    header.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    header.biWidth = w;
    header.biHeight = -h;
    header.biPlanes = 1;
    header.biBitCount = BITS_PER_PIXEL;
    header.biCompression = BI_RGB;
    let info = BITMAPINFO { bmiHeader: header, bmiColors: unsafe { std::mem::zeroed() } };
    unsafe {
        let screen = GetDC(std::ptr::null_mut());
        let dc = CreateCompatibleDC(screen);
        let mut bits = std::ptr::null_mut();
        let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
        let result = if bitmap.is_null() || bits.is_null() {
            Err(format!("CreateDIBSection failed: {}", GetLastError()))
        } else {
            let target = std::slice::from_raw_parts_mut(bits as *mut u8, pixmap.data().len());
            for (dst, px) in target.as_chunks_mut::<4>().0.iter_mut().zip(pixmap.pixels()) {
                *dst = [px.blue(), px.green(), px.red(), px.alpha()];
            }
            let old = SelectObject(dc, bitmap);
            let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: OPAQUE, AlphaFormat: AC_SRC_ALPHA as u8 };
            let (origin, size) = (POINT { x: 0, y: 0 }, SIZE { cx: w, cy: h });
            let result = if old.is_null() {
                Err("SelectObject failed".to_string())
            } else if UpdateLayeredWindow(hwnd as _, screen, std::ptr::null(), &size, dc, &origin, 0, &blend, ULW_ALPHA) == 0 {
                let error = GetLastError();
                let style = GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE);
                Err(format!("UpdateLayeredWindow failed: {error} exstyle={style:#x} size={w}x{h}"))
            } else {
                Ok(())
            };
            SelectObject(dc, old);
            result
        };
        DeleteObject(bitmap);
        DeleteDC(dc);
        ReleaseDC(std::ptr::null_mut(), screen);
        result
    }
}

fn cursor_and_monitor() -> Option<((i32, i32), Rect)> {
    let mut p = POINT { x: 0, y: 0 };
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    let ok = unsafe {
        GetCursorPos(&mut p);
        GetMonitorInfoW(MonitorFromPoint(p, MONITOR_DEFAULTTOPRIMARY), &mut info)
    };
    let r = info.rcMonitor;
    (ok != 0).then_some(((p.x, p.y), (r.left, r.top, r.right, r.bottom)))
}
