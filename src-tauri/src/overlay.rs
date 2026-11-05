use crate::draw::{self, Sketch, Style, Tool};
use crate::toolbar::{self, Button, Palette, Scene, Toolbar};
use image::{RgbaImage, imageops};
use softbuffer::{Context, Surface};
use crate::config::RedactStyle;
use std::num::NonZeroU32;
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use tiny_skia::Pixmap;
use winit::event::{ElementState, KeyEvent, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, ModifiersState, NamedKey};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const BORDER: u32 = 0x3090f0;
const SELECTION_BORDER: i32 = 2;
const HANDLE_SIZE: f32 = 8.0;
const HANDLE_HIT: f32 = 14.0;
const HANDLE_FILL: u32 = 0xffffff;
const MIN_SELECTION: i32 = 3;
const LABEL_GAP: f32 = 8.0;
const NOTICE: Duration = Duration::from_millis(1500);

static TOKEN: AtomicU64 = AtomicU64::new(0);

const HANDLES: [Handle; 8] = [
    Handle(-1, -1),
    Handle(0, -1),
    Handle(1, -1),
    Handle(-1, 0),
    Handle(1, 0),
    Handle(-1, 1),
    Handle(0, 1),
    Handle(1, 1),
];

#[derive(Clone, Copy)]
pub enum Pick {
    Window,
    Area,
}

pub enum Outcome {
    Pending,
    Cancel,
    Save(Shot),
    Copy(RgbaImage),
    Pin(RgbaImage),
    Detect { token: u64, image: RgbaImage },
}

#[derive(Clone, Copy, PartialEq)]
struct Handle(i32, i32);

impl Handle {
    fn point(self, (x, y, w, h): Rect) -> (i32, i32) {
        (x + (self.0 + 1) * w / 2, y + (self.1 + 1) * h / 2)
    }

    fn cursor(self) -> CursorIcon {
        match self {
            Handle(0, _) => CursorIcon::NsResize,
            Handle(_, 0) => CursorIcon::EwResize,
            Handle(x, y) if x == y => CursorIcon::NwseResize,
            _ => CursorIcon::NeswResize,
        }
    }
}

#[derive(Clone, Copy)]
enum Grip {
    Resize(Handle),
    Move,
}

#[derive(Clone, Copy)]
struct Adjust {
    grip: Grip,
    from: (i32, i32),
    start: Rect,
}

pub enum Shot {
    Ready(RgbaImage),
    Window { hwnd: isize, border: Option<[u8; 3]>, frame: RgbaImage, rect: Rect },
    Region { rect: Rect, start: (i32, i32), hwnd: Option<isize> },
}

pub type Rect = (i32, i32, i32, i32);

pub struct Overlay {
    pick: Pick,
    record: bool,
    origin: (i32, i32),
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    frame: RgbaImage,
    dim: Vec<u32>,
    bright: Vec<u32>,
    windows: Vec<(xcap::Window, Rect)>,
    cursor: (i32, i32),
    drag: Option<(i32, i32)>,
    hover: Option<usize>,
    monitors: Vec<Rect>,
    draw: Option<Style>,
    vi: bool,
    detectable: bool,
    modifiers: ModifiersState,
    annotation: Option<Annotation>,
}

struct Annotation {
    sketch: Sketch,
    toolbar: Toolbar,
    hover: Option<Button>,
    palette: Option<Palette>,
    icon: CursorIcon,
    adjust: Option<Adjust>,
    space: bool,
    label: Option<((i32, i32), Pixmap)>,
    detect: Option<u64>,
    detected: Option<Rect>,
    notice: Option<(Instant, Pixmap)>,
}

fn desktop() -> Option<(i32, i32, RgbaImage, Vec<Rect>)> {
    let bounds = xcap::Monitor::all()
        .ok()?
        .iter()
        .map(|m| Some((m.x().ok()?, m.y().ok()?, m.width().ok()?, m.height().ok()?)))
        .collect::<Option<Vec<_>>>()?;
    let ux = bounds.iter().map(|b| b.0).min()?;
    let uy = bounds.iter().map(|b| b.1).min()?;
    let right = bounds.iter().map(|b| b.0 + b.2 as i32).max()?;
    let bottom = bounds.iter().map(|b| b.1 + b.3 as i32).max()?;
    let (uw, uh) = ((right - ux) as u32, (bottom - uy) as u32);
    if uw == 0 || uh == 0 {
        return None;
    }
    let shots = std::thread::scope(|s| {
        let handles: Vec<_> = bounds
            .iter()
            .map(|&(x, y, _, _)| {
                s.spawn(move || xcap::Monitor::from_point(x, y).ok()?.capture_image().ok())
            })
            .collect();
        handles.into_iter().map(|h| h.join().ok().flatten()).collect::<Option<Vec<_>>>()
    })?;
    let mut canvas = RgbaImage::from_pixel(uw, uh, image::Rgba([0, 0, 0, 255]));
    for (&(x, y, _, _), img) in bounds.iter().zip(&shots) {
        imageops::replace(&mut canvas, img, (x - ux) as i64, (y - uy) as i64);
    }
    let monitors = bounds.iter().map(|&(x, y, w, h)| (x - ux, y - uy, w as i32, h as i32)).collect();
    Some((ux, uy, canvas, monitors))
}

impl Overlay {
    pub fn open(
        event_loop: &ActiveEventLoop,
        pick: Pick,
        record: bool,
        draw: Option<Style>,
        vi: bool,
        detectable: bool,
    ) -> Option<Self> {
        let (mx, my, frame, monitors) = desktop()?;
        let (w, h) = frame.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        let windows = match pick {
            Pick::Window => visible_windows(mx, my),
            Pick::Area => Vec::new(),
        };
        let attrs = Window::default_attributes()
            .with_title("Snapr")
            .with_decorations(false)
            .with_resizable(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(mx, my))
            .with_inner_size(PhysicalSize::new(w, h))
            .with_skip_taskbar(true);
        let window = Rc::new(event_loop.create_window(attrs).ok()?);
        let _ = window.request_inner_size(PhysicalSize::new(w, h));
        window.set_outer_position(PhysicalPosition::new(mx, my));
        let context = Context::new(window.clone()).ok()?;
        let surface = Surface::new(&context, window.clone()).ok()?;
        window.set_cursor(CursorIcon::Crosshair);
        window.focus_window();
        let pixels = frame.pixels();
        let bright: Vec<u32> = pixels.map(|p| u32::from_be_bytes([0, p[0], p[1], p[2]])).collect();
        let dim = bright.iter().map(|&p| (p >> 1) & 0x7f7f7f).collect();
        window.request_redraw();
        Some(Self {
            pick,
            record,
            origin: (mx, my),
            window,
            surface,
            frame,
            dim,
            bright,
            windows,
            cursor: (0, 0),
            drag: None,
            hover: None,
            monitors,
            draw,
            vi,
            detectable,
            modifiers: ModifiersState::empty(),
            annotation: None,
        })
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn event(&mut self, event: WindowEvent) -> Outcome {
        if self.draw.is_some()
            && let WindowEvent::ModifiersChanged(m) = &event
        {
            self.modifiers = m.state();
        }
        if self.annotation.is_some() {
            return self.annotate(event);
        }
        match event {
            WindowEvent::CloseRequested => Outcome::Cancel,
            WindowEvent::KeyboardInput { event, .. }
                if event.state == ElementState::Pressed && event.logical_key == Key::Named(NamedKey::Escape) =>
            {
                Outcome::Cancel
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as i32, position.y as i32);
                match self.pick {
                    Pick::Area if self.drag.is_some() => self.window.request_redraw(),
                    Pick::Window => {
                        let hover = self.hit();
                        if hover != self.hover {
                            self.hover = hover;
                            self.window.request_redraw();
                        }
                    }
                    Pick::Area => {}
                }
                Outcome::Pending
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Right, .. } => Outcome::Cancel,
            WindowEvent::MouseInput { state, button: MouseButton::Left, .. } => match (self.pick, state) {
                (Pick::Area, ElementState::Pressed) => {
                    self.drag = Some(self.cursor);
                    Outcome::Pending
                }
                (Pick::Area, ElementState::Released) => {
                    let Some(start) = self.drag.take() else { return Outcome::Pending };
                    let r = span(start, self.cursor);
                    if self.record {
                        let (ox, oy) = self.origin;
                        let hwnd = self.window_at(start);
                        return Outcome::Save(Shot::Region { rect: (r.0 + ox, r.1 + oy, r.2, r.3), start: (start.0 + ox, start.1 + oy), hwnd });
                    }
                    if r.2 >= 3 && r.3 >= 3 {
                        if let Some(style) = self.draw {
                            return self.begin_annotation(r, style);
                        }
                        return crop(&self.frame, r).map_or(Outcome::Cancel, |img| Outcome::Save(Shot::Ready(img)));
                    }
                    self.window.request_redraw();
                    Outcome::Pending
                }
                (Pick::Window, ElementState::Pressed) => self.pick_window(),
                (Pick::Window, ElementState::Released) => Outcome::Pending,
            },
            WindowEvent::RedrawRequested => self.draw().map_or(Outcome::Cancel, |_| Outcome::Pending),
            _ => Outcome::Pending,
        }
    }

    fn pick_window(&mut self) -> Outcome {
        let Some(i) = self.hit() else { return Outcome::Pending };
        let (win, r) = &self.windows[i];
        let r = *r;
        let hwnd = win.id().map_or(0, |id| id as i32 as isize);
        let border = self.edge_color(r);
        let frame = std::mem::take(&mut self.frame);
        Outcome::Save(Shot::Window { hwnd, border, frame, rect: r })
    }

    pub fn style(&self) -> Option<Style> {
        self.annotation.as_ref().map(|a| a.sketch.style()).or(self.draw)
    }

    fn begin_annotation(&mut self, r: Rect, style: Style) -> Outcome {
        let (w, h) = self.frame.dimensions();
        let Some((x0, y0, x1, y1)) = clamp(r, w, h) else { return Outcome::Cancel };
        let selection = (x0 as i32, y0 as i32, (x1 - x0) as i32, (y1 - y0) as i32);
        let monitor = monitor_of(&self.monitors, selection, (w, h));
        let scale = self.window.scale_factor() as f32;
        let Some((sketch, toolbar)) =
            Sketch::new(&self.frame, selection, scale, style).zip(Toolbar::place(selection, monitor, scale, self.vi))
        else {
            return Outcome::Cancel;
        };
        self.annotation = Some(Annotation {
            sketch,
            toolbar,
            hover: None,
            palette: None,
            icon: CursorIcon::Crosshair,
            adjust: None,
            space: false,
            label: None,
            detect: None,
            detected: None,
            notice: None,
        });
        self.hover_toolbar();
        self.window.request_redraw();
        Outcome::Pending
    }

    pub fn redactions(&mut self, token: u64, rects: Vec<(f32, f32, f32, f32)>) {
        let Some(a) = self.annotation.as_mut().filter(|a| a.detect == Some(token)) else { return };
        a.detect = None;
        if a.detected == Some(a.sketch.selection()) {
            if rects.is_empty() {
                let label = if self.vi { "Không tìm thấy" } else { "Nothing found" };
                let scale = self.window.scale_factor() as f32;
                a.notice = toolbar::tag(label, scale).map(|pixmap| (Instant::now() + NOTICE, pixmap));
            } else {
                a.sketch.redactions(&rects);
            }
        }
        self.hover_toolbar();
        self.window.request_redraw();
    }

    pub fn deadline(&self) -> Option<Instant> {
        self.annotation.as_ref().and_then(|a| a.notice.as_ref()).map(|(at, _)| *at)
    }

    pub fn tick(&mut self) {
        if let Some(a) = self.annotation.as_mut()
            && a.notice.take_if(|(at, _)| *at <= Instant::now()).is_some()
        {
            self.window.request_redraw();
        }
    }

    fn detect(&mut self) -> Outcome {
        let Some(a) = self.annotation.as_mut() else { return Outcome::Pending };
        if !self.detectable || a.detect.is_some() {
            return Outcome::Pending;
        }
        let selection = a.sketch.selection();
        let Some(image) = crop(&self.frame, selection) else { return Outcome::Pending };
        let token = TOKEN.fetch_add(1, Ordering::Relaxed) + 1;
        a.detect = Some(token);
        a.detected = Some(selection);
        a.notice = None;
        self.hover_toolbar();
        self.window.request_redraw();
        Outcome::Detect { token, image }
    }

    fn hover_toolbar(&mut self) {
        let scale = self.window.scale_factor() as f32;
        let Some(a) = self.annotation.as_mut() else { return };
        let (kind, style) = a.sketch.options();
        let fill = style.stroke.fill;
        let selection = a.sketch.selection();
        let ready = self.detectable && a.detect.is_none();
        let hover = a.toolbar.hit(self.cursor, kind, fill, a.palette).filter(|&b| enabled(&a.sketch, ready, b));
        let icon = if let Some(adjust) = a.adjust {
            match adjust.grip {
                Grip::Resize(handle) => handle.cursor(),
                Grip::Move => CursorIcon::Move,
            }
        } else if hover.is_some() {
            CursorIcon::Pointer
        } else if let Some(handle) = handle_at(selection, self.cursor, scale) {
            handle.cursor()
        } else if (a.space && contains(selection, self.cursor)) || a.sketch.grabs(self.cursor) {
            CursorIcon::Move
        } else {
            CursorIcon::Crosshair
        };
        if icon != a.icon {
            a.icon = icon;
            self.window.set_cursor(icon);
        }
        if hover != a.hover {
            a.hover = hover;
            self.window.request_redraw();
        }
    }

    fn annotate(&mut self, event: WindowEvent) -> Outcome {
        let scale = self.window.scale_factor() as f32;
        let Some(a) = self.annotation.as_mut() else { return Outcome::Pending };
        let shift = self.modifiers.shift_key();
        match event {
            WindowEvent::CloseRequested => return Outcome::Cancel,
            WindowEvent::ModifiersChanged(_) => {
                if a.sketch.drag_to(self.cursor, shift) {
                    self.window.request_redraw();
                }
                return Outcome::Pending;
            }
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor = (position.x as i32, position.y as i32);
                if let Some(adjust) = a.adjust {
                    let r = adjusted(adjust, self.cursor, self.frame.dimensions());
                    if r != a.sketch.selection() && a.sketch.reselect(&self.frame, r).is_some() {
                        self.window.request_redraw();
                    }
                } else if a.sketch.drag_to(self.cursor, shift) {
                    self.window.request_redraw();
                }
                self.hover_toolbar();
                return Outcome::Pending;
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                let (kind, style) = a.sketch.options();
                let fill = style.stroke.fill;
                let selection = a.sketch.selection();
                let palette = a.palette.take();
                let ready = self.detectable && a.detect.is_none();
                if let Some(button) = a.toolbar.hit(self.cursor, kind, fill, palette).filter(|&b| enabled(&a.sketch, ready, b)) {
                    match button {
                        Button::Swatch(i) => match palette {
                            Some(Palette::Stroke) => a.sketch.set_colour(draw::PALETTE[i]),
                            Some(Palette::Fill) => a.sketch.set_fill_colour(draw::PALETTE[i]),
                            None => {}
                        },
                        Button::Tool(tool) => a.sketch.set_tool(tool),
                        Button::Black => a.sketch.set_redact(RedactStyle::Black),
                        Button::Pixelate => a.sketch.set_redact(RedactStyle::Pixelate),
                        Button::Detect => return self.detect(),
                        Button::Undo => a.sketch.undo(),
                        Button::Redo => a.sketch.redo(),
                        Button::Pin => return self.pin(),
                        Button::Copy => return self.copy(),
                        Button::Save => return self.save(),
                        Button::Cancel => return Outcome::Cancel,
                        Button::Colour => a.palette = (palette != Some(Palette::Stroke)).then_some(Palette::Stroke),
                        Button::FillColour => a.palette = (palette != Some(Palette::Fill)).then_some(Palette::Fill),
                        Button::Outline => a.sketch.set_fill(false),
                        Button::Fill => a.sketch.set_fill(true),
                        Button::Width(i) => a.sketch.set_width(draw::WIDTHS[i]),
                        Button::Smaller => a.sketch.step(-1),
                        Button::Larger => a.sketch.step(1),
                        Button::Bold => a.sketch.toggle_bold(),
                        Button::Italic => a.sketch.toggle_italic(),
                    }
                } else if palette.is_some() {
                } else if let Some(handle) = handle_at(selection, self.cursor, scale) {
                    a.adjust = Some(Adjust { grip: Grip::Resize(handle), from: self.cursor, start: selection });
                } else if a.space && contains(selection, self.cursor) {
                    a.adjust = Some(Adjust { grip: Grip::Move, from: self.cursor, start: selection });
                } else {
                    a.sketch.press(self.cursor);
                }
                self.hover_toolbar();
            }
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                if a.adjust.take().is_some() {
                    let selection = a.sketch.selection();
                    let monitor = monitor_of(&self.monitors, selection, self.frame.dimensions());
                    if let Some(toolbar) = Toolbar::place(selection, monitor, scale, self.vi) {
                        a.toolbar = toolbar;
                    }
                } else {
                    a.sketch.release(self.cursor, shift);
                }
                self.hover_toolbar();
            }
            WindowEvent::KeyboardInput { event, .. }
                if event.logical_key == Key::Named(NamedKey::Space) && !a.sketch.editing() =>
            {
                let space = event.state == ElementState::Pressed;
                if space != a.space {
                    a.space = space;
                    self.hover_toolbar();
                }
                return Outcome::Pending;
            }
            WindowEvent::KeyboardInput { event, .. } if event.state == ElementState::Pressed => return self.key(event),
            WindowEvent::RedrawRequested => return self.draw().map_or(Outcome::Cancel, |_| Outcome::Pending),
            _ => return Outcome::Pending,
        }
        self.window.request_redraw();
        Outcome::Pending
    }

    fn key(&mut self, event: KeyEvent) -> Outcome {
        let Some(a) = self.annotation.as_mut() else { return Outcome::Pending };
        let ctrl = self.modifiers.control_key();
        let shortcut = |c: &str| ctrl && matches!(&event.logical_key, Key::Character(k) if k.eq_ignore_ascii_case(c));
        if shortcut("s") {
            return self.save();
        }
        if shortcut("p") {
            return self.pin();
        }
        if shortcut("c") && !a.sketch.editing() {
            return self.copy();
        }
        let shift = self.modifiers.shift_key();
        let text_tool = a.sketch.options().0 == Tool::Text;
        let named = |key: NamedKey| event.logical_key == Key::Named(key);
        let step = match &event.logical_key {
            Key::Character(c) if !ctrl && c == "[" => Some(-1),
            Key::Character(c) if !ctrl && c == "]" => Some(1),
            _ => None,
        };
        if shortcut("y") || (shift && shortcut("z")) {
            a.sketch.redo();
        } else if shortcut("z") {
            a.sketch.undo();
        } else if text_tool && shortcut("b") {
            a.sketch.toggle_bold();
        } else if text_tool && shortcut("i") {
            a.sketch.toggle_italic();
        } else if let Some(delta) = step {
            a.sketch.step(delta);
        } else if named(NamedKey::Escape) && a.palette.take().is_some() {
        } else if named(NamedKey::Delete) {
            a.sketch.delete();
        } else if named(NamedKey::Escape) && a.sketch.deselect() {
        } else if a.sketch.editing() {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => a.sketch.newline(),
                Key::Named(NamedKey::Backspace) => a.sketch.backspace(),
                Key::Named(NamedKey::Escape) => a.sketch.commit_text(),
                _ => {
                    if let Some(text) = event.text.as_deref().filter(|t| !t.chars().any(char::is_control)) {
                        a.sketch.type_text(text);
                    }
                }
            }
        } else {
            match &event.logical_key {
                Key::Named(NamedKey::Enter) => return self.save(),
                Key::Named(NamedKey::Escape) => return Outcome::Cancel,
                Key::Character(c) if !ctrl => {
                    if let Some(tool) = toolbar::shortcut(c) {
                        a.palette = None;
                        a.sketch.set_tool(tool);
                        self.hover_toolbar();
                    } else if c.eq_ignore_ascii_case("d") {
                        return self.detect();
                    } else if c.eq_ignore_ascii_case("f") {
                        a.palette = None;
                        a.sketch.toggle_fill();
                        self.hover_toolbar();
                    }
                }
                _ => return Outcome::Pending,
            }
        }
        self.window.request_redraw();
        Outcome::Pending
    }

    fn finish(&mut self) -> Option<RgbaImage> {
        let a = self.annotation.take()?;
        self.draw = Some(a.sketch.style());
        a.sketch.finish()
    }

    fn save(&mut self) -> Outcome {
        self.finish().map_or(Outcome::Cancel, |img| Outcome::Save(Shot::Ready(img)))
    }

    fn copy(&mut self) -> Outcome {
        self.finish().map_or(Outcome::Cancel, Outcome::Copy)
    }

    fn pin(&mut self) -> Outcome {
        self.finish().map_or(Outcome::Cancel, Outcome::Pin)
    }

    fn window_at(&self, at: (i32, i32)) -> Option<isize> {
        let (win, _) = &self.windows[self.hit_at(at)?];
        Some(win.id().map_or(0, |id| id as i32 as isize))
    }

    fn hit(&self) -> Option<usize> {
        self.hit_at(self.cursor)
    }

    fn hit_at(&self, at: (i32, i32)) -> Option<usize> {
        self.windows.iter().position(|&(_, r)| contains(r, at))
    }

    fn edge_color(&self, r: Rect) -> Option<[u8; 3]> {
        let (x, y) = (r.0 + r.2 / 2, r.1);
        if x < 0 || y < 0 || x as u32 >= self.frame.width() || y as u32 >= self.frame.height() {
            return None;
        }
        let p = self.frame.get_pixel(x as u32, y as u32);
        Some([p[0], p[1], p[2]])
    }

    fn draw(&mut self) -> Option<()> {
        let size = self.window.inner_size();
        self.surface
            .resize(NonZeroU32::new(size.width)?, NonZeroU32::new(size.height)?)
            .ok()?;
        let mut buf = self.surface.buffer_mut().ok()?;
        let (fw, sw) = (self.frame.width() as usize, size.width as usize);
        let w = fw.min(sw);
        let h = self.frame.height().min(size.height);
        buf.fill(0);
        for y in 0..h as usize {
            buf[y * sw..y * sw + w].copy_from_slice(&self.dim[y * fw..y * fw + w]);
        }
        let scale = self.window.scale_factor() as f32;
        if let Some(a) = self.annotation.as_mut() {
            let selection = a.sketch.selection();
            outline(&mut buf, sw, (w as u32, h), selection);
            blit(&mut buf, sw, (w as u32, h), a.sketch.view(), (selection.0, selection.1));
            handles(&mut buf, sw, (w as u32, h), selection, scale);
            let monitor = monitor_of(&self.monitors, selection, self.frame.dimensions());
            if let Some((label, at)) = size_tag(&mut a.label, selection, monitor, scale) {
                blit(&mut buf, sw, (w as u32, h), label, at);
            }
            let (kind, style) = a.sketch.options();
            let sketch = &a.sketch;
            let ready = self.detectable && a.detect.is_none();
            let busy = a.detect.is_some();
            let scene = Scene {
                tool: sketch.tool(),
                kind,
                style: &style,
                hover: a.hover,
                enabled: &|b| enabled(sketch, ready, b),
                palette: a.palette,
                busy,
            };
            for (view, at) in a.toolbar.render(&scene) {
                blit(&mut buf, sw, (w as u32, h), view, at);
            }
            if let Some((_, notice)) = &a.notice
                && let Some(at) = a.toolbar.anchor(
                    Button::Detect,
                    (notice.width() as i32, notice.height() as i32),
                    kind,
                    style.stroke.fill,
                    a.palette,
                )
            {
                blit(&mut buf, sw, (w as u32, h), notice, at);
            }
            if let Some(button) = a.hover
                && let Some((tip, at)) = a.toolbar.tooltip(button, kind, style.stroke.fill, a.palette)
            {
                blit(&mut buf, sw, (w as u32, h), tip, at);
            }
            return buf.present().ok();
        }
        let highlight = match self.pick {
            Pick::Area => self.drag.map(|s| span(s, self.cursor)),
            Pick::Window => self.hover.map(|i| self.windows[i].1),
        };
        if let Some((x0, y0, x1, y1)) = highlight.and_then(|r| clamp(r, w as u32, h)) {
            for y in y0..y1 {
                for x in x0..x1 {
                    let edge = x < x0 + 2 || y < y0 + 2 || x + 2 >= x1 || y + 2 >= y1;
                    let (x, y) = (x as usize, y as usize);
                    buf[y * sw + x] = if edge { BORDER } else { self.bright[y * fw + x] };
                }
            }
        }
        buf.present().ok()
    }
}

fn outline(buf: &mut [u32], stride: usize, (w, h): (u32, u32), (x, y, rw, rh): Rect) {
    let t = SELECTION_BORDER;
    let strips = [(x - t, y - t, rw + 2 * t, t), (x - t, y + rh, rw + 2 * t, t), (x - t, y, t, rh), (x + rw, y, t, rh)];
    for strip in strips {
        let Some((x0, y0, x1, y1)) = clamp(strip, w, h) else { continue };
        for row in y0..y1 {
            buf[row as usize * stride + x0 as usize..row as usize * stride + x1 as usize].fill(BORDER);
        }
    }
}

fn handles(buf: &mut [u32], stride: usize, (w, h): (u32, u32), selection: Rect, scale: f32) {
    let size = (HANDLE_SIZE * scale).round() as i32;
    let ring = SELECTION_BORDER.min(size / 2);
    for handle in HANDLES {
        let (x, y) = handle.point(selection);
        let outer = (x - size / 2, y - size / 2, size, size);
        let inner = (outer.0 + ring, outer.1 + ring, size - ring * 2, size - ring * 2);
        for (r, colour) in [(outer, BORDER), (inner, HANDLE_FILL)] {
            let Some((x0, y0, x1, y1)) = clamp(r, w, h) else { continue };
            for row in y0..y1 {
                buf[row as usize * stride + x0 as usize..row as usize * stride + x1 as usize].fill(colour);
            }
        }
    }
}

fn handle_at(selection: Rect, (x, y): (i32, i32), scale: f32) -> Option<Handle> {
    let half = (HANDLE_HIT * scale / 2.0).round() as i32;
    HANDLES.into_iter().find(|h| {
        let (hx, hy) = h.point(selection);
        (x - hx).abs() <= half && (y - hy).abs() <= half
    })
}

fn adjusted(adjust: Adjust, cursor: (i32, i32), (fw, fh): (u32, u32)) -> Rect {
    let (fw, fh) = (fw as i32, fh as i32);
    let (dx, dy) = (cursor.0 - adjust.from.0, cursor.1 - adjust.from.1);
    let (x, y, w, h) = adjust.start;
    match adjust.grip {
        Grip::Move => ((x + dx).clamp(0, (fw - w).max(0)), (y + dy).clamp(0, (fh - h).max(0)), w, h),
        Grip::Resize(Handle(hx, hy)) => {
            let (mut l, mut t, mut r, mut b) = (x, y, x + w, y + h);
            match hx {
                -1 => l = (l + dx).clamp(0, r - MIN_SELECTION),
                1 => r = (r + dx).clamp(l + MIN_SELECTION, fw),
                _ => {}
            }
            match hy {
                -1 => t = (t + dy).clamp(0, b - MIN_SELECTION),
                1 => b = (b + dy).clamp(t + MIN_SELECTION, fh),
                _ => {}
            }
            (l, t, r - l, b - t)
        }
    }
}

fn monitor_of(monitors: &[Rect], r: Rect, (w, h): (u32, u32)) -> Rect {
    let centre = (r.0 + r.2 / 2, r.1 + r.3 / 2);
    monitors.iter().copied().find(|&m| contains(m, centre)).unwrap_or((0, 0, w as i32, h as i32))
}

fn size_tag(cache: &mut Option<((i32, i32), Pixmap)>, selection: Rect, monitor: Rect, scale: f32) -> Option<(&Pixmap, (i32, i32))> {
    let (x, y, w, h) = selection;
    if cache.as_ref().is_none_or(|(size, _)| *size != (w, h)) {
        *cache = toolbar::tag(&format!("{w} × {h}"), scale).map(|pixmap| ((w, h), pixmap));
    }
    let (_, pixmap) = cache.as_ref()?;
    let gap = (LABEL_GAP * scale).round() as i32;
    let above = y - gap - pixmap.height() as i32;
    let at = if above >= monitor.1 { (x.max(monitor.0), above) } else { (x + gap, y + gap) };
    Some((pixmap, at))
}

fn enabled(sketch: &Sketch, ready: bool, button: Button) -> bool {
    match button {
        Button::Tool(Tool::Text) => draw::text_available(),
        Button::Detect => ready,
        Button::Undo => sketch.can_undo(),
        Button::Redo => sketch.can_redo(),
        _ => true,
    }
}

fn blit(buf: &mut [u32], stride: usize, (w, h): (u32, u32), pixmap: &Pixmap, (ox, oy): (i32, i32)) {
    let Some((x0, y0, x1, y1)) = clamp((ox, oy, pixmap.width() as i32, pixmap.height() as i32), w, h) else { return };
    let (pw, pixels) = (pixmap.width() as usize, pixmap.pixels());
    for y in y0..y1 {
        let row = (y as i32 - oy) as usize * pw;
        for x in x0..x1 {
            let p = pixels[row + (x as i32 - ox) as usize];
            let dst = &mut buf[y as usize * stride + x as usize];
            *dst = if p.alpha() == u8::MAX {
                u32::from_be_bytes([0, p.red(), p.green(), p.blue()])
            } else {
                let keep = u32::from(u8::MAX - p.alpha());
                let [_, r, g, b] = dst.to_be_bytes();
                let over = |d: u8, s: u8| (u32::from(d) * keep / u32::from(u8::MAX) + u32::from(s)).min(u32::from(u8::MAX)) as u8;
                u32::from_be_bytes([0, over(r, p.red()), over(g, p.green()), over(b, p.blue())])
            };
        }
    }
}

pub fn contains(r: Rect, (x, y): (i32, i32)) -> bool {
    x >= r.0 && y >= r.1 && x < r.0 + r.2 && y < r.1 + r.3
}

pub fn crop(frame: &RgbaImage, r: Rect) -> Option<RgbaImage> {
    let (x0, y0, x1, y1) = clamp(r, frame.width(), frame.height())?;
    Some(imageops::crop_imm(frame, x0, y0, x1 - x0, y1 - y0).to_image())
}

fn span(a: (i32, i32), b: (i32, i32)) -> Rect {
    (a.0.min(b.0), a.1.min(b.1), (a.0 - b.0).abs(), (a.1 - b.1).abs())
}

fn clamp(r: Rect, w: u32, h: u32) -> Option<(u32, u32, u32, u32)> {
    let x0 = r.0.clamp(0, w as i32) as u32;
    let y0 = r.1.clamp(0, h as i32) as u32;
    let x1 = (r.0 + r.2).clamp(0, w as i32) as u32;
    let y1 = (r.1 + r.3).clamp(0, h as i32) as u32;
    (x1 > x0 && y1 > y0).then_some((x0, y0, x1, y1))
}

fn visible_windows(mx: i32, my: i32) -> Vec<(xcap::Window, Rect)> {
    let me = std::process::id();
    xcap::Window::all()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|win| {
            if win.pid().ok()? == me || win.is_minimized().ok()? || win.title().ok()?.is_empty() {
                return None;
            }
            let (w, h) = (win.width().ok()? as i32, win.height().ok()? as i32);
            if w <= 0 || h <= 0 {
                return None;
            }
            let r = (win.x().ok()? - mx, win.y().ok()? - my, w, h);
            Some((win, r))
        })
        .collect()
}
