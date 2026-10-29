use crate::config::RingItem;
use crate::overlay::{Pick, Rect};
use crate::settings::wide;
use crate::{Action, State, draw, gallery, show_settings, tooltip, toolbar, trigger};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use std::f32::consts::{PI, TAU};
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager};
use tiny_skia::{
    Color, ColorU8, FillRule, FilterQuality, GradientStop, LineCap, Paint, PathBuilder, Pixmap, PixmapPaint, Point, RadialGradient,
    SpreadMode, Stroke, Transform,
};
use windows_sys::Win32::UI::Shell::ShellExecuteW;
use windows_sys::Win32::Foundation::{GetLastError, POINT, SIZE};
use windows_sys::Win32::Graphics::Gdi::{
    AC_SRC_ALPHA, AC_SRC_OVER, BI_RGB, BITMAPINFO, BITMAPINFOHEADER, BLENDFUNCTION, CreateCompatibleDC, CreateDIBSection,
    DIB_RGB_COLORS, DeleteDC, DeleteObject, GetDC, GetMonitorInfoW, MONITOR_DEFAULTTOPRIMARY, MONITORINFO, MonitorFromPoint,
    ReleaseDC, SelectObject,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetCursorPos, GetWindowLongPtrW, SW_SHOW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SW_SHOWNORMAL, SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, ULW_ALPHA, UpdateLayeredWindow, WS_EX_LAYERED,
    WS_EX_TOOLWINDOW,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, KeyCode, NamedKey, PhysicalKey};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const ORBIT: f32 = 90.0;
const BUTTON_RADIUS: f32 = 24.0;
const HOVER_SCALE: f32 = 1.15;
const ICON_SIZE: f32 = 22.0;
const CARD_GAP: f32 = 8.0;
const CARD_SIDE_FRACTION: f32 = 0.5;
const CARD_RADIUS: f32 = 6.0;
const CARD_PAD_X: f32 = 14.0;
const CARD_PAD_Y: f32 = 8.0;
const LABEL_SIZE: f32 = 14.0;
const SHADOW_SPREAD: f32 = 6.0;
const SHADOW_OFFSET: f32 = 2.0;
const SHADOW_LAYERS: u32 = 4;
const SHADOW_ALPHA: u8 = 18;
const BUTTON_SHADOW_SPREAD: f32 = 4.0;
const BUTTON_SHADOW_OFFSET: f32 = 2.0;
const BUTTON_SHADOW_ALPHA: u8 = 48;
const SUPERSAMPLE: u32 = 2;
const SHADOW: [u8; 3] = [0x00, 0x00, 0x00];
const WINDOW_MARGIN: f32 = BUTTON_SHADOW_SPREAD + BUTTON_SHADOW_OFFSET + 10.0;
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
const CLOSE_DURATION: Duration = Duration::from_millis(110);
const OPEN_START_SCALE: f32 = 0.5;
const CARD_SLIDE: f32 = 6.0;
const BITS_PER_PIXEL: u16 = 32;
const STOP_RECORDING: (&str, &str, char) = ("Stop recording", "Dừng quay", '\u{E71A}');
const RING_HIDE_DELAY: Duration = Duration::from_millis(150);
const SHELL_EXECUTE_MAX_ERROR: usize = 32;
const SHELL_ACCESS_DENIED: usize = 5;
const VERB_OPEN: &str = "open";
const VERB_ELEVATE: &str = "runas";
const APP_ICON_SIZE: f32 = 32.0;
const APP_GLYPH: char = '\u{E8FC}';
const BUTTON_GAP: f32 = 20.0;
const BADGE_RADIUS: f32 = 10.0;
const BADGE_OUTLINE: f32 = 2.0;
const BADGE_RIM: f32 = std::f32::consts::FRAC_1_SQRT_2;
const BADGE_TEXT_SIZE: f32 = 13.0;
const BADGE_FILL: [u8; 3] = CARD_FILL;
const BADGE_TEXT: [u8; 3] = CARD_TEXT;
const POSITION_KEYS: usize = 9;
const DIGIT_KEYS: [(KeyCode, KeyCode); 10] = [
    (KeyCode::Digit0, KeyCode::Numpad0),
    (KeyCode::Digit1, KeyCode::Numpad1),
    (KeyCode::Digit2, KeyCode::Numpad2),
    (KeyCode::Digit3, KeyCode::Numpad3),
    (KeyCode::Digit4, KeyCode::Numpad4),
    (KeyCode::Digit5, KeyCode::Numpad5),
    (KeyCode::Digit6, KeyCode::Numpad6),
    (KeyCode::Digit7, KeyCode::Numpad7),
    (KeyCode::Digit8, KeyCode::Numpad8),
    (KeyCode::Digit9, KeyCode::Numpad9),
];
const LETTER_KEYS: [KeyCode; 26] = [
    KeyCode::KeyA,
    KeyCode::KeyB,
    KeyCode::KeyC,
    KeyCode::KeyD,
    KeyCode::KeyE,
    KeyCode::KeyF,
    KeyCode::KeyG,
    KeyCode::KeyH,
    KeyCode::KeyI,
    KeyCode::KeyJ,
    KeyCode::KeyK,
    KeyCode::KeyL,
    KeyCode::KeyM,
    KeyCode::KeyN,
    KeyCode::KeyO,
    KeyCode::KeyP,
    KeyCode::KeyQ,
    KeyCode::KeyR,
    KeyCode::KeyS,
    KeyCode::KeyT,
    KeyCode::KeyU,
    KeyCode::KeyV,
    KeyCode::KeyW,
    KeyCode::KeyX,
    KeyCode::KeyY,
    KeyCode::KeyZ,
];
const DECIMAL: u32 = 10;

#[derive(Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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

#[derive(Clone, PartialEq, Deserialize, Serialize)]
pub struct AppEntry {
    pub label: String,
    pub path: String,
    #[serde(default)]
    pub args: String,
    #[serde(default)]
    pub admin: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
}

#[derive(Clone)]
pub enum Entry {
    Builtin(Wedge, Option<char>),
    App(AppEntry),
}

pub fn key_char(text: &str) -> Option<char> {
    let mut chars = text.chars();
    match (chars.next(), chars.next()) {
        (Some(key), None) if key.is_ascii_alphanumeric() => Some(key.to_ascii_uppercase()),
        _ => None,
    }
}

fn pressed_char(code: KeyCode) -> Option<char> {
    let digit = DIGIT_KEYS.iter().position(|&(main, pad)| code == main || code == pad).and_then(|i| char::from_digit(i as u32, DECIMAL));
    digit.or_else(|| LETTER_KEYS.iter().position(|&letter| letter == code).map(|i| (b'A' + i as u8) as char))
}

fn resolve_keys(entries: &[Entry]) -> Vec<Option<char>> {
    let mut taken = Vec::new();
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let position = if i < POSITION_KEYS { char::from_digit(i as u32 + 1, DECIMAL) } else { None };
            let key = entry.key().or(position).filter(|key| !taken.contains(key));
            taken.extend(key);
            key
        })
        .collect()
}

pub enum Outcome {
    Pending,
    Cancel,
    Pick(Entry),
}

enum Icon {
    Glyph(char),
    Bitmap(Pixmap),
}

const WEDGES: [(Wedge, &str, &str, char); 8] = [
    (Wedge::Fullscreen, "Fullscreen", "Toàn màn hình", '\u{E740}'),
    (Wedge::Window, "Window", "Cửa sổ", '\u{E737}'),
    (Wedge::Area, "Area", "Vùng chọn", '\u{EF20}'),
    (Wedge::Text, "Text", "Quét chữ", '\u{E8E9}'),
    (Wedge::Record, "Record", "Quay video", '\u{E7C8}'),
    (Wedge::Draw, "Draw", "Vẽ", '\u{E70F}'),
    (Wedge::Gallery, "Gallery", "Thư viện", '\u{E8B9}'),
    (Wedge::Settings, "Settings", "Cài đặt", '\u{E713}'),
];

impl Wedge {
    pub const ALL: [Wedge; 8] = [
        Wedge::Fullscreen,
        Wedge::Window,
        Wedge::Area,
        Wedge::Text,
        Wedge::Record,
        Wedge::Draw,
        Wedge::Gallery,
        Wedge::Settings,
    ];

    fn face(self, recording: bool) -> (&'static str, &'static str, char) {
        let (_, en, vn, glyph) = WEDGES.iter().find(|(w, ..)| *w == self).copied().unwrap_or(WEDGES[0]);
        if recording && self == Wedge::Record { (STOP_RECORDING.0, STOP_RECORDING.1, STOP_RECORDING.2) } else { (en, vn, glyph) }
    }

    fn action(self) -> Result<Action, &'static str> {
        match self {
            Wedge::Fullscreen => Ok(Action::Fullscreen),
            Wedge::Window => Ok(Action::Pick(Pick::Window)),
            Wedge::Area => Ok(Action::Pick(Pick::Area)),
            Wedge::Text => Ok(Action::Text),
            Wedge::Record => Ok(Action::Record),
            Wedge::Draw => Ok(Action::Draw),
            Wedge::Gallery => Err("gallery"),
            Wedge::Settings => Err("settings"),
        }
    }

    fn run(self, app: &AppHandle) {
        match self.action() {
            Ok(action) => {
                let app = app.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(RING_HIDE_DELAY);
                    trigger(&app, &action);
                });
            }
            Err(section) => show_settings(app, section),
        }
    }
}

impl AppEntry {
    fn run(&self, app: &AppHandle) {
        let path = Path::new(&self.path);
        let folder = path.is_file().then(|| path.parent().and_then(Path::to_str)).flatten().map(wide);
        let elevate = self.admin && path.is_file();
        let verb = if elevate { VERB_ELEVATE } else { VERB_OPEN };
        let args = (!self.args.is_empty()).then(|| wide(&self.args));
        let optional = |text: &Option<Vec<u16>>| text.as_ref().map_or(std::ptr::null(), |t| t.as_ptr());
        let result = unsafe {
            ShellExecuteW(std::ptr::null_mut(), wide(verb).as_ptr(), wide(&self.path).as_ptr(), optional(&args), optional(&folder), SW_SHOWNORMAL)
        };
        let code = result as usize;
        let declined = elevate && code == SHELL_ACCESS_DENIED;
        if code <= SHELL_EXECUTE_MAX_ERROR && !declined {
            let vi = app.state::<State>().cfg.read().unwrap().language == "vi";
            let text = if vi { format!("Snapr: không thể mở {}", self.label) } else { format!("Snapr: could not launch {}", self.label) };
            tooltip(app, &text);
        }
    }

    fn icon(&self, pixels: i32) -> Icon {
        gallery::icon_rgba(Path::new(&self.path), pixels)
            .and_then(|img| bitmap(&img))
            .map_or(Icon::Glyph(APP_GLYPH), Icon::Bitmap)
    }
}

impl Entry {
    pub fn available(&self, text: bool) -> bool {
        text || !matches!(self, Entry::Builtin(Wedge::Text, _))
    }

    fn key(&self) -> Option<char> {
        match self {
            Entry::Builtin(_, key) => *key,
            Entry::App(app) => app.key.as_deref().and_then(key_char),
        }
    }

    pub fn run(&self, app: &AppHandle) {
        match self {
            Entry::Builtin(wedge, _) => wedge.run(app),
            Entry::App(app_entry) => app_entry.run(app),
        }
    }

    fn label(&self, recording: bool, vi: bool) -> String {
        match self {
            Entry::Builtin(wedge, _) => {
                let (en, vn, _) = wedge.face(recording);
                (if vi { vn } else { en }).to_string()
            }
            Entry::App(app) => app.label.clone(),
        }
    }

    fn icon(&self, recording: bool, pixels: i32) -> Icon {
        match self {
            Entry::Builtin(wedge, _) => Icon::Glyph(wedge.face(recording).2),
            Entry::App(app) => app.icon(pixels),
        }
    }
}

impl From<&RingItem> for Entry {
    fn from(item: &RingItem) -> Self {
        match item {
            RingItem::Builtin { id, key } => Entry::Builtin(*id, key.as_deref().and_then(key_char)),
            RingItem::App(app) => Entry::App(app.clone()),
        }
    }
}

pub(crate) fn bitmap(img: &RgbaImage) -> Option<Pixmap> {
    let mut pixmap = Pixmap::new(img.width(), img.height())?;
    for (dst, px) in pixmap.pixels_mut().iter_mut().zip(img.pixels()) {
        let [r, g, b, a] = px.0;
        *dst = ColorU8::from_rgba(r, g, b, a).premultiply();
    }
    Some(pixmap)
}

type Card = (f32, f32, f32, f32);

struct Item {
    label: String,
    icon: Icon,
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
    entries: Vec<Entry>,
    items: Vec<Item>,
    scale: f32,
    keys: Vec<Option<char>>,
    side: u32,
    centre: f32,
    hover: Option<usize>,
    glow: Vec<f32>,
    opened: Instant,
    active: Instant,
    idle: Option<Duration>,
    closing: Option<Closing>,
    frame: Instant,
}

struct Closing {
    started: Instant,
    from: f32,
    picked: Option<Entry>,
}

impl Ring {
    pub fn open(event_loop: &ActiveEventLoop, entries: Vec<Entry>, recording: bool, vi: bool, keys_on: bool, idle: Option<Duration>) -> Option<Self> {
        if entries.is_empty() {
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
        let items = layout(&entries, recording, vi, scale);
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
        let keys = if keys_on { resolve_keys(&entries) } else { vec![None; entries.len()] };
        let now = Instant::now();
        let mut ring = Self {
            window,
            hwnd,
            entries,
            items,
            scale,
            keys,
            side: side as u32,
            centre: side as f32 / 2.0,
            hover: None,
            glow,
            opened: now,
            active: now,
            idle,
            closing: None,
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

    fn open_factor(&self) -> f32 {
        match &self.closing {
            Some(closing) => {
                let progress = (closing.started.elapsed().as_secs_f32() / CLOSE_DURATION.as_secs_f32()).min(1.0);
                closing.from * ease_out(1.0 - progress)
            }
            None => ease_out((self.opened.elapsed().as_secs_f32() / OPEN_DURATION.as_secs_f32()).min(1.0)),
        }
    }

    pub fn begin_close(&mut self, picked: Option<Entry>) {
        if self.closing.is_none() {
            self.closing = Some(Closing { started: Instant::now(), from: self.open_factor(), picked });
        }
    }

    pub fn finished(&self) -> bool {
        self.closing.as_ref().is_some_and(|closing| closing.started.elapsed() >= CLOSE_DURATION)
    }

    pub fn into_picked(self) -> Option<Entry> {
        self.closing.and_then(|closing| closing.picked)
    }

    pub fn event(&mut self, event: WindowEvent) -> Outcome {
        if self.closing.is_some() && !matches!(event, WindowEvent::RedrawRequested) {
            return Outcome::Pending;
        }
        if matches!(event, WindowEvent::CursorMoved { .. } | WindowEvent::KeyboardInput { .. } | WindowEvent::MouseInput { .. }) {
            self.active = Instant::now();
        }
        match event {
            WindowEvent::CloseRequested => Outcome::Cancel,
            WindowEvent::Focused(false) if self.opened.elapsed() < FOCUS_GRACE => Outcome::Pending,
            WindowEvent::Focused(false) => Outcome::Cancel,
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                Outcome::Cancel
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed && !event.repeat => {
                let PhysicalKey::Code(code) = event.physical_key else { return Outcome::Pending };
                let Some(pressed) = pressed_char(code) else { return Outcome::Pending };
                self.keys
                    .iter()
                    .position(|&key| key == Some(pressed))
                    .map_or(Outcome::Pending, |i| Outcome::Pick(self.entries[i].clone()))
            }
            WindowEvent::CursorMoved { position, .. } => {
                let hover = self.hit((position.x as f32, position.y as f32));
                self.set_hover(hover);
                Outcome::Pending
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let hover = self.cursor_hit();
                self.set_hover(hover);
                self.hover.map_or(Outcome::Cancel, |i| Outcome::Pick(self.entries[i].clone()))
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
        if self.hover.is_some() {
            self.active = now;
        } else if self.idle.is_some_and(|idle| self.active.elapsed() >= idle) {
            self.begin_close(None);
        }
        let mut moving = self.closing.is_some() || self.opened.elapsed() < OPEN_DURATION;
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
        let Some(mut pixmap) = Pixmap::new(self.side * SUPERSAMPLE, self.side * SUPERSAMPLE) else { return };
        let zoom = SUPERSAMPLE as f32;
        let scale = self.scale * zoom;
        let face = draw::font();
        let origin = Point::from_xy(self.centre * zoom, self.centre * zoom);
        let open = self.open_factor();
        let card_fade = if self.closing.is_some() { open } else { 1.0 };
        let opening = OPEN_START_SCALE + (1.0 - OPEN_START_SCALE) * open;
        let alpha = (open * OPAQUE as f32) as u8;
        for (index, (item, &glow)) in self.items.iter().zip(&self.glow).enumerate() {
            let lift = ease_out(glow) * card_fade;
            if lift > 0.0 && let Some(face) = face {
                let shift = -(1.0 - lift) * CARD_SLIDE * scale;
                let from = Point::from_xy(origin.x + item.dir.x * shift, origin.y + item.dir.y * shift);
                paint_card(&mut pixmap, face, item, from, (scale, zoom), lift);
            }
            let grow = opening * (1.0 + (HOVER_SCALE - 1.0) * lift);
            let at = Point::from_xy(origin.x + item.button.x * zoom * open, origin.y + item.button.y * zoom * open);
            paint_button_shadow(&mut pixmap, at, BUTTON_RADIUS * scale * grow, scale, alpha);
            if let Some(disc) = PathBuilder::from_circle(at.x, at.y, BUTTON_RADIUS * scale * grow) {
                pixmap.fill_path(&disc, &draw::paint(BUTTON_FILL, alpha), FillRule::Winding, Transform::identity(), None);
            }
            match &item.icon {
                Icon::Glyph(glyph) => toolbar::icon(&mut pixmap, (*glyph, 0.0), at, ICON_SIZE * scale * grow, &draw::paint(ICON_COLOUR, alpha)),
                Icon::Bitmap(icon) => paint_bitmap(&mut pixmap, icon, at, APP_ICON_SIZE * scale * grow, alpha),
            }
            if let (Some(key), Some(face)) = (self.keys[index], face) {
                let rim = BUTTON_RADIUS * scale * grow;
                let badge = Point::from_xy(at.x + BADGE_RIM * rim, at.y - BADGE_RIM * rim);
                paint_badge(&mut pixmap, face, badge, key, scale * grow, (alpha, open));
            }
        }
        paint_close(&mut pixmap, origin, scale * opening, alpha);
        if let Some(frame) = downsample(&pixmap) {
            let _ = present(self.hwnd, &frame);
        }
    }
}

fn downsample(source: &Pixmap) -> Option<Pixmap> {
    let side = source.width() / SUPERSAMPLE;
    let mut target = Pixmap::new(side, side)?;
    let paint = PixmapPaint { quality: FilterQuality::Bilinear, ..PixmapPaint::default() };
    let shrink = Transform::from_scale(1.0 / SUPERSAMPLE as f32, 1.0 / SUPERSAMPLE as f32);
    target.draw_pixmap(0, 0, source.as_ref(), &paint, shrink, None);
    Some(target)
}

fn ease_out(t: f32) -> f32 {
    1.0 - (1.0 - t).powi(3)
}

fn mix(from: [u8; 3], to: [u8; 3], t: f32) -> [u8; 3] {
    [0, 1, 2].map(|i| (from[i] as f32 + (to[i] as f32 - from[i] as f32) * t).round() as u8)
}

fn orbit(count: usize) -> f32 {
    let spacing = 2.0 * (PI / count.max(2) as f32).sin();
    ORBIT.max((2.0 * BUTTON_RADIUS + BUTTON_GAP) / spacing)
}

fn layout(entries: &[Entry], recording: bool, vi: bool, scale: f32) -> Vec<Item> {
    let face = draw::font();
    let step = TAU / entries.len() as f32;
    let orbit = orbit(entries.len());
    let icon_pixels = (APP_ICON_SIZE * HOVER_SCALE * scale * SUPERSAMPLE as f32).ceil() as i32;
    entries
        .iter()
        .enumerate()
        .map(|(i, entry)| {
            let label = entry.label(recording, vi);
            let angle = i as f32 * step;
            let (dx, dy) = (angle.sin(), -angle.cos());
            let button = Point::from_xy(dx * orbit * scale, dy * orbit * scale);
            let (tw, th) = face.map_or((0.0, 0.0), |f| draw::measure(f, LABEL_SIZE * scale, &label));
            let (w, h) = (tw + 2.0 * CARD_PAD_X * scale, th + 2.0 * CARD_PAD_Y * scale);
            let anchor = (BUTTON_RADIUS * HOVER_SCALE + CARD_GAP) * scale;
            let (px, py) = (button.x + dx * anchor, button.y + dy * anchor);
            let side = (dx / CARD_SIDE_FRACTION).clamp(-1.0, 1.0);
            let card = (px + side * w / 2.0 - w / 2.0, py + dy * h / 2.0 - h / 2.0, w, h);
            Item { label, icon: entry.icon(recording, icon_pixels), button, card, dir: Point::from_xy(dx, dy) }
        })
        .collect()
}

fn paint_badge(pixmap: &mut Pixmap, face: &ab_glyph::FontVec, at: Point, key: char, scale: f32, (alpha, fade): (u8, f32)) {
    if let Some(ring) = PathBuilder::from_circle(at.x, at.y, (BADGE_RADIUS + BADGE_OUTLINE) * scale) {
        pixmap.fill_path(&ring, &draw::paint(BUTTON_FILL, alpha), FillRule::Winding, Transform::identity(), None);
    }
    if let Some(disc) = PathBuilder::from_circle(at.x, at.y, BADGE_RADIUS * scale) {
        pixmap.fill_path(&disc, &draw::paint(BADGE_FILL, alpha), FillRule::Winding, Transform::identity(), None);
    }
    let digit = key.to_string();
    let size = BADGE_TEXT_SIZE * scale;
    let (tw, th) = draw::measure(face, size, &digit);
    let corner = Point::from_xy(at.x - tw / 2.0, at.y - th / 2.0);
    draw::text(pixmap, face, corner, &[digit], size, mix(BADGE_FILL, BADGE_TEXT, fade), None);
}

fn paint_bitmap(pixmap: &mut Pixmap, icon: &Pixmap, centre: Point, size: f32, alpha: u8) {
    let zoom = size / icon.width() as f32;
    let transform = Transform::from_scale(zoom, zoom).post_translate(centre.x - size / 2.0, centre.y - size / 2.0);
    let paint = PixmapPaint { opacity: alpha as f32 / OPAQUE as f32, quality: FilterQuality::Bicubic, ..PixmapPaint::default() };
    pixmap.draw_pixmap(0, 0, icon.as_ref(), &paint, transform, None);
}

fn paint_button_shadow(pixmap: &mut Pixmap, at: Point, radius: f32, scale: f32, alpha: u8) {
    let peak = (BUTTON_SHADOW_ALPHA as f32 * alpha as f32 / OPAQUE as f32) as u8;
    let centre = Point::from_xy(at.x, at.y + BUTTON_SHADOW_OFFSET * scale);
    let outer = radius + BUTTON_SHADOW_SPREAD * scale;
    let [r, g, b] = SHADOW;
    let stops = vec![GradientStop::new(0.0, Color::from_rgba8(r, g, b, peak)), GradientStop::new(1.0, Color::from_rgba8(r, g, b, 0))];
    let Some(shader) = RadialGradient::new(centre, radius, centre, outer, stops, SpreadMode::Pad, Transform::identity()) else { return };
    let paint = Paint { shader, anti_alias: true, ..Paint::default() };
    if let Some(disc) = PathBuilder::from_circle(centre.x, centre.y, outer) {
        pixmap.fill_path(&disc, &paint, FillRule::Winding, Transform::identity(), None);
    }
}

fn paint_card(pixmap: &mut Pixmap, face: &ab_glyph::FontVec, item: &Item, origin: Point, (scale, zoom): (f32, f32), fade: f32) {
    let (x, y, w, h) = item.card;
    let (x, y, w, h) = (origin.x + x * zoom, origin.y + y * zoom, w * zoom, h * zoom);
    for layer in 0..SHADOW_LAYERS {
        let grow = SHADOW_SPREAD * scale * (SHADOW_LAYERS - layer) as f32 / SHADOW_LAYERS as f32;
        let rect = (x - grow, y - grow + SHADOW_OFFSET * scale, w + grow * 2.0, h + grow * 2.0);
        toolbar::fill(pixmap, rect, CARD_RADIUS * scale + grow, SHADOW, (SHADOW_ALPHA as f32 * fade) as u8);
    }
    let alpha = (fade * OPAQUE as f32) as u8;
    toolbar::fill(pixmap, (x, y, w, h), CARD_RADIUS * scale, CARD_FILL, alpha);
    let (tw, th) = draw::measure(face, LABEL_SIZE * scale, &item.label);
    let at = Point::from_xy(x + (w - tw) / 2.0, y + (h - th) / 2.0);
    draw::text(pixmap, face, at, std::slice::from_ref(&item.label), LABEL_SIZE * scale, mix(CARD_FILL, CARD_TEXT, fade), None);
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

pub(crate) fn present(hwnd: isize, pixmap: &Pixmap) -> Result<(), String> {
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
