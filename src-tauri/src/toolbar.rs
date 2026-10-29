use crate::draw::{self, Style, Tool};
use crate::overlay::{self, Rect};
use ab_glyph::{Font, FontVec, GlyphId, OutlineCurve, Point as AbPoint};
use std::sync::OnceLock;
use tiny_skia::{Color, FillRule, Path, PathBuilder, Pixmap, Point, Stroke, Transform};

const BUTTON: f32 = 32.0;
const GAP: f32 = 4.0;
const PADDING: f32 = 4.0;
const MARGIN: f32 = 8.0;
const ICON_SIZE: f32 = 16.0;
const PANEL_RADIUS: f32 = 8.0;
const BUTTON_RADIUS: f32 = 4.0;
const BORDER_WIDTH: f32 = 1.0;
const DIVIDER_WIDTH: f32 = 1.0;
const DIVIDER_GAP: f32 = 4.0;
const SIZE_SLOT: f32 = 32.0;
const SIZE_TEXT: f32 = 14.0;
const SWATCH_RADIUS: f32 = 8.0;
const SWATCH_RING: f32 = 1.0;
const FILL_RADIUS: f32 = 3.0;
const OUTLINE_WIDTH: f32 = 2.0;
const BAR_WIDTH: f32 = 16.0;
const BAR_HEIGHT: f32 = 3.0;
const BAR_INSET: f32 = 3.0;
const SHADOW_SPREAD: f32 = 8.0;
const SHADOW_OFFSET: f32 = 2.0;
const SHADOW_LAYERS: u32 = 4;
const SHADOW_ALPHA: u8 = 12;
const SHADOW: [u8; 3] = [0x00, 0x00, 0x00];
const KAPPA: f32 = 0.5523;
const PANEL: [u8; 3] = [0x2b, 0x2b, 0x2b];
const BORDER: [u8; 3] = [0x3d, 0x3d, 0x3d];
const HOVER: [u8; 3] = [0x3a, 0x3a, 0x3a];
const ACCENT: [u8; 3] = [0x4c, 0xc2, 0xff];
const ACCENT_HOVER: [u8; 3] = [0x70, 0xd0, 0xff];
const ICON: [u8; 3] = [0xff, 0xff, 0xff];
const ICON_ON_ACCENT: [u8; 3] = [0x00, 0x00, 0x00];
const DISABLED_ALPHA: u8 = 0x60;
const ICON_FILES: [&str; 2] = ["SegoeIcons.ttf", "segmdl2.ttf"];
const GLYPH_RECT: char = '\u{E739}';
const GLYPH_ELLIPSE: char = '\u{EA3A}';
const GLYPH_ARROW: char = '\u{F0AF}';
const GLYPH_TEXT: char = '\u{EF60}';
const GLYPH_SAVE: char = '\u{E73E}';
const GLYPH_CANCEL: char = '\u{E711}';
const GLYPH_SMALLER: char = '\u{E8E7}';
const GLYPH_LARGER: char = '\u{E8E8}';
const GLYPH_BOLD: char = '\u{E8DD}';
const GLYPH_ITALIC: char = '\u{E8DB}';
const GLYPH_UNDO: char = '\u{E7A7}';
const GLYPH_REDO: char = '\u{E7A6}';
const GLYPH_COPY: char = '\u{E8C8}';
const GLYPH_PIN: char = '\u{E718}';
const RING_WIDTH: f32 = 2.5;
const FILL_TINT: u8 = 0x90;
const ROW_GAP: f32 = 4.0;
const ARROW_ANGLE: f32 = -45.0;
const TIP_RADIUS: f32 = 4.0;
const TIP_PAD_X: f32 = 8.0;
const TIP_PAD_Y: f32 = 5.0;
const TIP_TEXT: f32 = 12.0;
const TIP_GAP: f32 = 6.0;
const TIP_TEXT_COLOUR: [u8; 3] = [0xff, 0xff, 0xff];
const COLOUR_NAMES: [(&str, &str); 8] = [
    ("Red", "Đỏ"),
    ("Orange", "Cam"),
    ("Yellow", "Vàng"),
    ("Green", "Xanh lá"),
    ("Blue", "Xanh dương"),
    ("Purple", "Tím"),
    ("Black", "Đen"),
    ("White", "Trắng"),
];
const WIDTH_NAMES: [(&str, &str); 3] =
    [("Thin ([ ])", "Mảnh ([ ])"), ("Medium ([ ])", "Vừa ([ ])"), ("Thick ([ ])", "Dày ([ ])")];
static ICON_FONT: OnceLock<Option<FontVec>> = OnceLock::new();
const MAIN_ROW: [Slot; 13] = [
    Slot::Control(Button::Tool(Tool::Rect)),
    Slot::Control(Button::Tool(Tool::Ellipse)),
    Slot::Control(Button::Tool(Tool::Arrow)),
    Slot::Control(Button::Tool(Tool::Text)),
    Slot::Divider,
    Slot::Control(Button::Undo),
    Slot::Control(Button::Redo),
    Slot::Divider,
    Slot::Control(Button::Pin),
    Slot::Control(Button::Copy),
    Slot::Divider,
    Slot::Control(Button::Save),
    Slot::Control(Button::Cancel),
];
const PALETTE_SLOTS: [Slot; 8] = [
    Slot::Control(Button::Swatch(0)),
    Slot::Control(Button::Swatch(1)),
    Slot::Control(Button::Swatch(2)),
    Slot::Control(Button::Swatch(3)),
    Slot::Control(Button::Swatch(4)),
    Slot::Control(Button::Swatch(5)),
    Slot::Control(Button::Swatch(6)),
    Slot::Control(Button::Swatch(7)),
];
const WIDTH_SLOTS: [Slot; 4] = [
    Slot::Divider,
    Slot::Control(Button::Width(0)),
    Slot::Control(Button::Width(1)),
    Slot::Control(Button::Width(2)),
];
const TEXT_OPTIONS: [Slot; 6] = [
    Slot::Control(Button::Smaller),
    Slot::Size,
    Slot::Control(Button::Larger),
    Slot::Divider,
    Slot::Control(Button::Bold),
    Slot::Control(Button::Italic),
];

#[derive(Clone, Copy, PartialEq)]
pub enum Palette {
    Stroke,
    Fill,
}

impl Palette {
    fn button(self) -> Button {
        match self {
            Palette::Stroke => Button::Colour,
            Palette::Fill => Button::FillColour,
        }
    }

    fn colour(self, kind: Tool, style: &Style) -> [u8; 3] {
        match self {
            Palette::Stroke => style.colour(kind),
            Palette::Fill => style.stroke.fill_rgb(),
        }
    }

}

#[derive(Clone, Copy, PartialEq)]
pub enum Button {
    Tool(Tool),
    Undo,
    Redo,
    Pin,
    Copy,
    Save,
    Cancel,
    Swatch(usize),
    Width(usize),
    Colour,
    FillColour,
    Outline,
    Fill,
    Smaller,
    Larger,
    Bold,
    Italic,
}

#[derive(Clone, Copy, PartialEq)]
enum Slot {
    Control(Button),
    Divider,
    Size,
}

impl Slot {
    fn button(self) -> Option<Button> {
        match self {
            Slot::Control(button) => Some(button),
            Slot::Divider | Slot::Size => None,
        }
    }
}

struct Look {
    rest: Option<[u8; 3]>,
    hover: [u8; 3],
    icon: [u8; 3],
}

impl Button {
    fn look(self) -> Look {
        match self {
            Button::Save => Look { rest: Some(ACCENT), hover: ACCENT_HOVER, icon: ICON_ON_ACCENT },
            _ => Look { rest: None, hover: HOVER, icon: ICON },
        }
    }

    fn label(self, kind: Tool, vi: bool) -> &'static str {
        let (en, vi_label) = match self {
            Button::Tool(Tool::Rect) => ("Rectangle (R)", "Hình chữ nhật (R)"),
            Button::Tool(Tool::Ellipse) => ("Ellipse (C)", "Hình elip (C)"),
            Button::Tool(Tool::Arrow) => ("Arrow (A)", "Mũi tên (A)"),
            Button::Tool(Tool::Text) => ("Text (T)", "Văn bản (T)"),
            Button::Undo => ("Undo (Ctrl+Z)", "Hoàn tác (Ctrl+Z)"),
            Button::Redo => ("Redo (Ctrl+Y)", "Làm lại (Ctrl+Y)"),
            Button::Pin => ("Pin (Ctrl+P)", "Ghim (Ctrl+P)"),
            Button::Copy => ("Copy (Ctrl+C)", "Sao chép (Ctrl+C)"),
            Button::Save => ("Done (Enter)", "Xong (Enter)"),
            Button::Cancel => ("Cancel (Esc)", "Hủy (Esc)"),
            Button::Swatch(i) => COLOUR_NAMES[i],
            Button::Width(i) => WIDTH_NAMES[i],
            Button::Colour if kind == Tool::Text => ("Text colour", "Màu chữ"),
            Button::Colour => ("Outline colour", "Màu viền"),
            Button::FillColour => ("Fill colour", "Màu tô"),
            Button::Outline => ("Outline only (F)", "Chỉ viền (F)"),
            Button::Fill => ("Outline and fill (F)", "Viền và tô (F)"),
            Button::Smaller => ("Smaller ([)", "Nhỏ hơn ([)"),
            Button::Larger => ("Larger (])", "Lớn hơn (])"),
            Button::Bold => ("Bold (Ctrl+B)", "Đậm (Ctrl+B)"),
            Button::Italic => ("Italic (Ctrl+I)", "Nghiêng (Ctrl+I)"),
        };
        if vi { vi_label } else { en }
    }

    fn glyph(self) -> Option<(char, f32)> {
        match self {
            Button::Tool(Tool::Rect) => Some((GLYPH_RECT, 0.0)),
            Button::Tool(Tool::Ellipse) => Some((GLYPH_ELLIPSE, 0.0)),
            Button::Tool(Tool::Arrow) => Some((GLYPH_ARROW, ARROW_ANGLE)),
            Button::Tool(Tool::Text) => Some((GLYPH_TEXT, 0.0)),
            Button::Undo => Some((GLYPH_UNDO, 0.0)),
            Button::Redo => Some((GLYPH_REDO, 0.0)),
            Button::Pin => Some((GLYPH_PIN, 0.0)),
            Button::Copy => Some((GLYPH_COPY, 0.0)),
            Button::Save => Some((GLYPH_SAVE, 0.0)),
            Button::Cancel => Some((GLYPH_CANCEL, 0.0)),
            Button::Smaller => Some((GLYPH_SMALLER, 0.0)),
            Button::Larger => Some((GLYPH_LARGER, 0.0)),
            Button::Bold => Some((GLYPH_BOLD, 0.0)),
            Button::Italic => Some((GLYPH_ITALIC, 0.0)),
            Button::Swatch(_)
            | Button::Width(_)
            | Button::Colour
            | Button::FillColour
            | Button::Outline
            | Button::Fill => None,
        }
    }

    fn current(self, tool: Tool, style: &Style) -> bool {
        match self {
            Button::Tool(t) => t == tool,
            Button::Width(i) => draw::WIDTHS[i] == style.stroke.width,
            Button::Outline => !style.stroke.fill,
            Button::Fill => style.stroke.fill,
            Button::Bold => style.text.bold,
            Button::Italic => style.text.italic,
            Button::Undo
            | Button::Redo
            | Button::Pin
            | Button::Copy
            | Button::Save
            | Button::Cancel
            | Button::Swatch(_)
            | Button::Colour
            | Button::FillColour
            | Button::Smaller
            | Button::Larger => false,
        }
    }
}

#[derive(Clone, Copy)]
struct State {
    hovered: bool,
    current: bool,
    alpha: u8,
}

pub fn shortcut(key: &str) -> Option<Tool> {
    match key.to_lowercase().as_str() {
        "r" => Some(Tool::Rect),
        "c" => Some(Tool::Ellipse),
        "a" => Some(Tool::Arrow),
        "t" => Some(Tool::Text),
        _ => None,
    }
}

fn px(v: f32, scale: f32) -> i32 {
    (v * scale).round() as i32
}

fn options_row(kind: Tool, fill: bool, palette: Option<Palette>) -> Vec<Slot> {
    if let Some(target) = palette {
        let mut row = vec![Slot::Control(target.button()), Slot::Divider];
        row.extend(PALETTE_SLOTS);
        return row;
    }
    let mut row = match kind {
        Tool::Rect | Tool::Ellipse => {
            vec![Slot::Control(Button::Outline), Slot::Control(Button::Fill), Slot::Divider, Slot::Control(Button::Colour)]
        }
        Tool::Arrow | Tool::Text => vec![Slot::Control(Button::Colour)],
    };
    if fill && matches!(kind, Tool::Rect | Tool::Ellipse) {
        row.push(Slot::Control(Button::FillColour));
    }
    match kind {
        Tool::Text => {
            row.push(Slot::Divider);
            row.extend(TEXT_OPTIONS);
        }
        _ => row.extend(WIDTH_SLOTS),
    }
    row
}

fn layout<'a>(rows: impl IntoIterator<Item = &'a Slot>, scale: f32) -> (Vec<(Slot, Rect)>, i32) {
    let mut x = PADDING;
    let slots = rows
        .into_iter()
        .map(|&slot| {
            let width = match slot {
                Slot::Control(_) => BUTTON,
                Slot::Divider => DIVIDER_GAP * 2.0 + DIVIDER_WIDTH - GAP,
                Slot::Size => SIZE_SLOT,
            };
            let rect = (px(x, scale), px(PADDING, scale), px(width, scale), px(BUTTON, scale));
            x += width + GAP;
            (slot, rect)
        })
        .collect();
    (slots, px(x - GAP + PADDING, scale))
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<Path> {
    let r = r.min(w / 2.0).min(h / 2.0);
    let k = r * KAPPA;
    let (right, bottom) = (x + w, y + h);
    let mut p = PathBuilder::new();
    p.move_to(x + r, y);
    p.line_to(right - r, y);
    p.cubic_to(right - r + k, y, right, y + r - k, right, y + r);
    p.line_to(right, bottom - r);
    p.cubic_to(right, bottom - r + k, right - r + k, bottom, right - r, bottom);
    p.line_to(x + r, bottom);
    p.cubic_to(x + r - k, bottom, x, bottom - r + k, x, bottom - r);
    p.line_to(x, y + r);
    p.cubic_to(x, y + r - k, x + r - k, y, x + r, y);
    p.close();
    p.finish()
}

pub fn fill(pixmap: &mut Pixmap, (x, y, w, h): (f32, f32, f32, f32), radius: f32, rgb: [u8; 3], alpha: u8) {
    if let Some(path) = rounded(x, y, w, h, radius) {
        pixmap.fill_path(&path, &draw::paint(rgb, alpha), tiny_skia::FillRule::Winding, Transform::identity(), None);
    }
}

fn circle(pixmap: &mut Pixmap, c: Point, radius: f32, rgb: [u8; 3]) {
    if let Some(path) = PathBuilder::from_circle(c.x, c.y, radius) {
        pixmap.fill_path(&path, &draw::paint(rgb, u8::MAX), FillRule::Winding, Transform::identity(), None);
    }
}

fn paint_panel(pixmap: &mut Pixmap, (x, y, w, h): (f32, f32, f32, f32), scale: f32) {
    let offset = SHADOW_OFFSET * scale;
    let radius = PANEL_RADIUS * scale;
    let border = BORDER_WIDTH * scale;
    for layer in 0..SHADOW_LAYERS {
        let grow = SHADOW_SPREAD * scale * (SHADOW_LAYERS - layer) as f32 / SHADOW_LAYERS as f32;
        let r = (x - grow, y - grow + offset, w + grow * 2.0, h + grow * 2.0);
        fill(pixmap, r, radius + grow, SHADOW, SHADOW_ALPHA);
    }
    fill(pixmap, (x, y, w, h), radius, BORDER, u8::MAX);
    fill(pixmap, (x + border, y + border, w - border * 2.0, h - border * 2.0), radius - border, PANEL, u8::MAX);
}

fn paint_control(
    pixmap: &mut Pixmap,
    scale: f32,
    button: Button,
    (x, y, w, h): (f32, f32, f32, f32),
    state: State,
    style: &Style,
    kind: Tool,
) {
    let look = button.look();
    let background = match (state.hovered, state.current) {
        (true, _) => Some(look.hover),
        (_, true) => Some(HOVER),
        _ => look.rest,
    };
    if let Some(rgb) = background {
        fill(pixmap, (x, y, w, h), BUTTON_RADIUS * scale, rgb, u8::MAX);
    }
    if state.current {
        let (bw, bh) = (BAR_WIDTH * scale, BAR_HEIGHT * scale);
        let bar = (x + (w - bw) / 2.0, y + h - BAR_INSET * scale - bh, bw, bh);
        fill(pixmap, bar, bh / 2.0, ACCENT, u8::MAX);
    }
    let centre = Point::from_xy(x + w / 2.0, y + h / 2.0);
    content(pixmap, scale, button, centre, state.alpha, style, kind);
}

fn content(pixmap: &mut Pixmap, scale: f32, button: Button, centre: Point, alpha: u8, style: &Style, kind: Tool) {
    let size = ICON_SIZE * scale;
    let rgb = button.look().icon;
    let square = (centre.x - size / 2.0, centre.y - size / 2.0, size, size);
    let radius = FILL_RADIUS * scale;
    match button {
        Button::Swatch(i) => swatch(pixmap, scale, centre, draw::PALETTE[i]),
        Button::Colour if kind == Tool::Text => swatch(pixmap, scale, centre, style.colour(kind)),
        Button::Colour => ring(pixmap, scale, centre, style.stroke.colour, alpha),
        Button::FillColour => {
            let ring = SWATCH_RING * scale;
            let outer = (square.0 - ring, square.1 - ring, size + ring * 2.0, size + ring * 2.0);
            fill(pixmap, outer, radius + ring, BORDER, alpha);
            fill(pixmap, square, radius, style.stroke.fill_rgb(), alpha);
        }
        Button::Outline => {
            let width = OUTLINE_WIDTH * scale;
            let inset = width / 2.0;
            if let Some(path) = rounded(square.0 + inset, square.1 + inset, size - width, size - width, radius) {
                let stroke = Stroke { width, ..Stroke::default() };
                pixmap.stroke_path(&path, &draw::paint(rgb, alpha), &stroke, Transform::identity(), None);
            }
        }
        Button::Fill => {
            fill(pixmap, square, radius, rgb, FILL_TINT.min(alpha));
            let width = OUTLINE_WIDTH * scale;
            let inset = width / 2.0;
            if let Some(path) = rounded(square.0 + inset, square.1 + inset, size - width, size - width, radius) {
                let stroke = Stroke { width, ..Stroke::default() };
                pixmap.stroke_path(&path, &draw::paint(rgb, alpha), &stroke, Transform::identity(), None);
            }
        }
        Button::Width(i) => {
            let thickness = (draw::WIDTHS[i] * scale).min(size);
            let line = (centre.x - size / 2.0, centre.y - thickness / 2.0, size, thickness);
            fill(pixmap, line, thickness / 2.0, rgb, alpha);
        }
        _ => {
            if let Some(glyph) = button.glyph() {
                icon(pixmap, glyph, centre, size, &draw::paint(rgb, alpha));
            }
        }
    }
}

fn ring(pixmap: &mut Pixmap, scale: f32, centre: Point, rgb: [u8; 3], alpha: u8) {
    let width = RING_WIDTH * scale;
    let radius = SWATCH_RADIUS * scale - width / 2.0;
    if let Some(path) = PathBuilder::from_circle(centre.x, centre.y, radius) {
        let stroke = Stroke { width, ..Stroke::default() };
        pixmap.stroke_path(&path, &draw::paint(rgb, alpha), &stroke, Transform::identity(), None);
    }
}

fn swatch(pixmap: &mut Pixmap, scale: f32, centre: Point, rgb: [u8; 3]) {
    let radius = SWATCH_RADIUS * scale;
    circle(pixmap, centre, radius + SWATCH_RING * scale, BORDER);
    circle(pixmap, centre, radius, rgb);
}

fn size_label(pixmap: &mut Pixmap, scale: f32, centre: Point, value: f32) {
    let Some(face) = draw::font() else { return };
    let label = format!("{value}");
    let size = SIZE_TEXT * scale;
    let (w, h) = draw::measure(face, size, &label);
    let at = Point::from_xy(centre.x - w / 2.0, centre.y - h / 2.0);
    draw::text(pixmap, face, at, &[label], size, ICON, None);
}

fn paint_row(pixmap: &mut Pixmap, slots: &[(Slot, Rect)], (w, h): (i32, i32), pad: f32, scale: f32) {
    pixmap.fill(Color::TRANSPARENT);
    paint_panel(pixmap, (pad, pad, w as f32, h as f32), scale);
    let line = px(DIVIDER_WIDTH, scale).max(1) as f32;
    for &(_, (x, y, sw, sh)) in slots.iter().filter(|&&(slot, _)| slot == Slot::Divider) {
        let mid = pad + x as f32 + sw as f32 / 2.0;
        fill(pixmap, (mid - line / 2.0, pad + y as f32, line, sh as f32), 0.0, BORDER, u8::MAX);
    }
}

struct Scene<'a> {
    tool: Tool,
    kind: Tool,
    style: &'a Style,
    hover: Option<Button>,
    enabled: &'a dyn Fn(Button) -> bool,
    palette: Option<Palette>,
}

fn paint_slots(pixmap: &mut Pixmap, slots: &[(Slot, Rect)], pad: f32, scale: f32, scene: &Scene) {
    for &(slot, (x, y, w, h)) in slots {
        let rect = (x as f32 + pad, y as f32 + pad, w as f32, h as f32);
        let state = |button: Button| {
            let enabled = (scene.enabled)(button);
            let current = match (button, scene.palette) {
                (Button::Swatch(i), Some(target)) => draw::PALETTE[i] == target.colour(scene.kind, scene.style),
                (_, Some(target)) => button == target.button(),
                _ => button.current(scene.tool, scene.style),
            };
            State {
                hovered: enabled && scene.hover == Some(button),
                current,
                alpha: if enabled { u8::MAX } else { DISABLED_ALPHA },
            }
        };
        match slot {
            Slot::Divider => {}
            Slot::Size => {
                let centre = Point::from_xy(rect.0 + rect.2 / 2.0, rect.1 + rect.3 / 2.0);
                size_label(pixmap, scale, centre, scene.style.text.size);
            }
            Slot::Control(button) => paint_control(pixmap, scale, button, rect, state(button), scene.style, scene.kind),
        }
    }
}

fn offset(slots: Vec<(Slot, Rect)>, (x, y): (i32, i32)) -> Vec<(Slot, Rect)> {
    slots.into_iter().map(|(slot, (sx, sy, sw, sh))| (slot, (sx + x, sy + y, sw, sh))).collect()
}

pub fn tag(label: &str, scale: f32) -> Option<Pixmap> {
    let face = draw::font()?;
    let size = TIP_TEXT * scale;
    let (text_w, text_h) = draw::measure(face, size, label);
    let (pad_x, pad_y) = (TIP_PAD_X * scale, TIP_PAD_Y * scale);
    let (w, h) = ((text_w + pad_x * 2.0).ceil(), (text_h + pad_y * 2.0).ceil());
    let mut pixmap = Pixmap::new(w as u32, h as u32)?;
    let (radius, border) = (TIP_RADIUS * scale, BORDER_WIDTH * scale);
    fill(&mut pixmap, (0.0, 0.0, w, h), radius, BORDER, u8::MAX);
    fill(&mut pixmap, (border, border, w - border * 2.0, h - border * 2.0), radius - border, PANEL, u8::MAX);
    draw::text(&mut pixmap, face, Point::from_xy(pad_x, pad_y), &[label.to_string()], size, TIP_TEXT_COLOUR, None);
    Some(pixmap)
}

pub struct Toolbar {
    main: Rect,
    down: bool,
    monitor: Rect,
    scale: f32,
    vi: bool,
    pad: i32,
    main_base: Pixmap,
    main_pixmap: Pixmap,
    options_base: Pixmap,
    options_row: Vec<Slot>,
    options_pixmap: Pixmap,
    tip: Option<(Button, Rect, Pixmap, (i32, i32))>,
}

impl Toolbar {
    pub fn place(selection: Rect, monitor: Rect, scale: f32, vi: bool) -> Option<Self> {
        let (main_slots, w) = layout(&MAIN_ROW, scale);
        let kinds = [Tool::Rect, Tool::Ellipse, Tool::Arrow, Tool::Text];
        let palettes = [None, Some(Palette::Stroke), Some(Palette::Fill)];
        let options_w = kinds
            .into_iter()
            .flat_map(|kind| palettes.into_iter().map(move |palette| (kind, palette)))
            .map(|(kind, palette)| layout(&options_row(kind, true, palette), scale).1)
            .max()?;
        let h = px(PADDING * 2.0 + BUTTON, scale);
        let total = h * 2 + px(ROW_GAP, scale);
        let margin = px(MARGIN, scale);
        let (sx, sy, sw, sh) = selection;
        let (mx, my, mw, mh) = monitor;
        let (y, down) = if sy + sh + margin + total <= my + mh {
            (sy + sh + margin, true)
        } else if sy - margin - total >= my {
            (sy - margin - h, false)
        } else {
            (sy + margin, true)
        };
        let fit = |v: i32, lo: i32, room: i32| v.min(lo + room).max(lo);
        let x = fit(sx + sw / 2 - w / 2, mx, mw - w);
        let y = fit(y, my, mh - h);
        let pad = px(SHADOW_SPREAD + SHADOW_OFFSET, scale);
        let mut main_base = Pixmap::new((w + pad * 2) as u32, (h + pad * 2) as u32)?;
        paint_row(&mut main_base, &main_slots, (w, h), pad as f32, scale);
        let options = ((options_w + pad * 2) as u32, (h + pad * 2) as u32);
        Some(Self {
            main: (x, y, w, h),
            down,
            monitor,
            scale,
            vi,
            pad,
            main_pixmap: main_base.clone(),
            main_base,
            options_base: Pixmap::new(options.0, options.1)?,
            options_row: Vec::new(),
            options_pixmap: Pixmap::new(options.0, options.1)?,
            tip: None,
        })
    }

    fn options(&self, kind: Tool, fill: bool, palette: Option<Palette>) -> Option<(Rect, Vec<(Slot, Rect)>)> {
        let row = options_row(kind, fill, palette);
        if row.is_empty() {
            return None;
        }
        let (slots, w) = layout(&row, self.scale);
        let (mx, _, mw, _) = self.monitor;
        let (x, y, _, h) = self.main;
        let gap = px(ROW_GAP, self.scale);
        let y = if self.down { y + h + gap } else { y - gap - h };
        Some(((x.min(mx + mw - w).max(mx), y, w, h), slots))
    }

    fn slots(&self, kind: Tool, fill: bool, palette: Option<Palette>) -> Vec<(Slot, Rect)> {
        let (x, y, ..) = self.main;
        let mut slots = offset(layout(&MAIN_ROW, self.scale).0, (x, y));
        if let Some(((ox, oy, ..), row)) = self.options(kind, fill, palette) {
            slots.extend(offset(row, (ox, oy)));
        }
        slots
    }

    fn block(&self, kind: Tool, fill: bool, palette: Option<Palette>) -> (i32, i32) {
        let (_, y, _, h) = self.main;
        match self.options(kind, fill, palette) {
            Some(((_, oy, _, oh), _)) => (y.min(oy), (y + h).max(oy + oh)),
            None => (y, y + h),
        }
    }

    pub fn hit(&self, at: (i32, i32), kind: Tool, fill: bool, palette: Option<Palette>) -> Option<Button> {
        self.slots(kind, fill, palette).into_iter().find_map(|(slot, r)| slot.button().filter(|_| overlay::contains(r, at)))
    }

    pub fn tooltip(&mut self, button: Button, kind: Tool, fill: bool, palette: Option<Palette>) -> Option<(&Pixmap, (i32, i32))> {
        let slot = self
            .slots(kind, fill, palette)
            .into_iter()
            .find(|&(slot, _)| slot.button() == Some(button))
            .map(|(_, r)| r)?;
        if self.tip.as_ref().is_none_or(|t| t.0 != button || t.1 != slot) {
            self.tip = tag(button.label(kind, self.vi), self.scale).map(|pixmap| {
                let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
                let (bx, _, bw, _) = slot;
                let (mx, my, mw, mh) = self.monitor;
                let gap = px(TIP_GAP, self.scale);
                let (top, bottom) = self.block(kind, fill, palette);
                let (below, above) = (bottom + gap, top - gap - h);
                let y = match (self.down, below + h <= my + mh, above >= my) {
                    (true, true, _) | (false, _, false) => below,
                    _ => above,
                };
                let at = ((bx + bw / 2 - w / 2).min(mx + mw - w).max(mx), y.min(my + mh - h).max(my));
                (button, slot, pixmap, at)
            });
        }
        self.tip.as_ref().map(|(_, _, pixmap, at)| (pixmap, *at))
    }

    pub fn render(
        &mut self,
        tool: Tool,
        kind: Tool,
        style: &Style,
        hover: Option<Button>,
        palette: Option<Palette>,
        enabled: &dyn Fn(Button) -> bool,
    ) -> Vec<(&Pixmap, (i32, i32))> {
        let scene = Scene { tool, kind, style, hover, enabled, palette };
        let pad = self.pad as f32;
        self.main_pixmap.data_mut().copy_from_slice(self.main_base.data());
        paint_slots(&mut self.main_pixmap, &layout(&MAIN_ROW, self.scale).0, pad, self.scale, &scene);
        let main_at = (self.main.0 - self.pad, self.main.1 - self.pad);
        let fill = style.stroke.fill;
        let Some(((ox, oy, ow, oh), slots)) = self.options(kind, fill, palette) else {
            return vec![(&self.main_pixmap, main_at)];
        };
        let row = options_row(kind, fill, palette);
        if self.options_row != row {
            paint_row(&mut self.options_base, &slots, (ow, oh), pad, self.scale);
            self.options_row = row;
        }
        self.options_pixmap.data_mut().copy_from_slice(self.options_base.data());
        paint_slots(&mut self.options_pixmap, &slots, pad, self.scale, &scene);
        vec![(&self.main_pixmap, main_at), (&self.options_pixmap, (ox - self.pad, oy - self.pad))]
    }
}

pub fn icon_font() -> Option<&'static FontVec> {
    ICON_FONT.get_or_init(|| draw::system_font(&ICON_FILES)).as_ref()
}

fn glyph_path(font: &FontVec, c: char) -> Option<(Path, (f32, f32), f32)> {
    let id = font.glyph_id(c);
    if id == GlyphId(0) {
        return None;
    }
    let outline = font.outline(id)?;
    let mut path = PathBuilder::new();
    let mut end: Option<AbPoint> = None;
    for curve in &outline.curves {
        let start = match curve {
            OutlineCurve::Line(a, _) | OutlineCurve::Quad(a, ..) | OutlineCurve::Cubic(a, ..) => *a,
        };
        if end != Some(start) {
            if end.is_some() {
                path.close();
            }
            path.move_to(start.x, start.y);
        }
        end = Some(match *curve {
            OutlineCurve::Line(_, b) => {
                path.line_to(b.x, b.y);
                b
            }
            OutlineCurve::Quad(_, b, c) => {
                path.quad_to(b.x, b.y, c.x, c.y);
                c
            }
            OutlineCurve::Cubic(_, b, c, d) => {
                path.cubic_to(b.x, b.y, c.x, c.y, d.x, d.y);
                d
            }
        });
    }
    path.close();
    let (min, max) = (outline.bounds.min, outline.bounds.max);
    Some((path.finish()?, ((min.x + max.x) / 2.0, (min.y + max.y) / 2.0), font.units_per_em()?))
}

pub fn icon(pixmap: &mut Pixmap, (glyph, angle): (char, f32), c: Point, size: f32, paint: &tiny_skia::Paint) {
    let Some(font) = icon_font() else { return };
    let Some((path, (cx, cy), units)) = glyph_path(font, glyph) else { return };
    let scale = size / units;
    let transform = Transform::from_translate(-cx, -cy).post_scale(scale, -scale).post_rotate(angle).post_translate(c.x, c.y);
    pixmap.fill_path(&path, paint, FillRule::Winding, transform, None);
}
