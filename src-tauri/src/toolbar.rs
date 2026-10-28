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
const TOOL_SLOTS: [Slot; 4] = [
    Slot::Control(Button::Tool(Tool::Rect)),
    Slot::Control(Button::Tool(Tool::Ellipse)),
    Slot::Control(Button::Tool(Tool::Arrow)),
    Slot::Control(Button::Tool(Tool::Text)),
];
const ACTION_SLOTS: [Slot; 3] = [Slot::Divider, Slot::Control(Button::Save), Slot::Control(Button::Cancel)];
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
const TEXT_OPTIONS: [Slot; 7] = [
    Slot::Divider,
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

    fn label(self, vi: bool) -> &'static str {
        let (en, vi_label) = match self {
            Button::Tool(Tool::Rect) => ("Rectangle (R)", "Hình chữ nhật (R)"),
            Button::Tool(Tool::Ellipse) => ("Ellipse (C)", "Hình elip (C)"),
            Button::Tool(Tool::Arrow) => ("Arrow (A)", "Mũi tên (A)"),
            Button::Tool(Tool::Text) => ("Text (T)", "Văn bản (T)"),
            Button::Save => ("Save (Ctrl+S)", "Lưu (Ctrl+S)"),
            Button::Cancel => ("Cancel (Esc)", "Hủy (Esc)"),
            Button::Swatch(i) => COLOUR_NAMES[i],
            Button::Width(i) => WIDTH_NAMES[i],
            Button::Colour => ("Colour", "Màu"),
            Button::FillColour => ("Fill colour", "Màu tô"),
            Button::Outline => ("Outline (F)", "Viền (F)"),
            Button::Fill => ("Fill (F)", "Tô màu (F)"),
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
            Button::Save
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

fn row(kind: Tool, fill: bool) -> Vec<Slot> {
    let mut row = TOOL_SLOTS.to_vec();
    row.extend([Slot::Divider, Slot::Control(Button::Colour)]);
    match kind {
        Tool::Rect | Tool::Ellipse => {
            row.extend([Slot::Control(Button::Outline), Slot::Control(Button::Fill)]);
            if fill {
                row.push(Slot::Control(Button::FillColour));
            }
            row.extend(WIDTH_SLOTS);
        }
        Tool::Arrow => row.extend(WIDTH_SLOTS),
        Tool::Text => row.extend(TEXT_OPTIONS),
    }
    row.extend(ACTION_SLOTS);
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
        Button::Colour => swatch(pixmap, scale, centre, style.colour(kind)),
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
        Button::Fill => fill(pixmap, square, radius, rgb, alpha),
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

fn swatch(pixmap: &mut Pixmap, scale: f32, centre: Point, rgb: [u8; 3]) {
    let radius = SWATCH_RADIUS * scale;
    circle(pixmap, centre, radius + SWATCH_RING * scale, BORDER);
    circle(pixmap, centre, radius, rgb);
}

pub struct Toolbar {
    rect: Rect,
    monitor: Rect,
    scale: f32,
    pad: i32,
    base: Pixmap,
    base_row: Vec<Slot>,
    pixmap: Pixmap,
    tip: Option<(Button, Rect, Pixmap, (i32, i32))>,
    popup: Option<(PopupKey, Pixmap, (i32, i32))>,
}

#[derive(Clone, Copy, PartialEq)]
struct PopupKey {
    target: Palette,
    kind: Tool,
    fill: bool,
    chosen: [u8; 3],
    hover: Option<Button>,
}

impl Toolbar {
    pub fn place(selection: Rect, monitor: Rect, scale: f32) -> Option<Self> {
        let w = [(Tool::Rect, true), (Tool::Text, false)]
            .into_iter()
            .map(|(kind, fill)| layout(&row(kind, fill), scale).1)
            .max()?;
        let h = px(PADDING * 2.0 + BUTTON, scale);
        let margin = px(MARGIN, scale);
        let (sx, sy, sw, sh) = selection;
        let (mx, my, mw, mh) = monitor;
        let below = sy + sh + margin;
        let above = sy - margin - h;
        let y = if below + h <= my + mh {
            below
        } else if above >= my {
            above
        } else {
            sy + margin
        };
        let fit = |v: i32, lo: i32, room: i32| v.min(lo + room).max(lo);
        let x = fit(sx + sw - w, mx, mw - w);
        let y = fit(y, my, mh - h);
        let pad = px(SHADOW_SPREAD + SHADOW_OFFSET, scale);
        let size = ((w + pad * 2) as u32, (h + pad * 2) as u32);
        Some(Self {
            rect: (x, y, w, h),
            monitor,
            scale,
            pad,
            base: Pixmap::new(size.0, size.1)?,
            base_row: Vec::new(),
            pixmap: Pixmap::new(size.0, size.1)?,
            tip: None,
            popup: None,
        })
    }

    pub fn origin(&self) -> (i32, i32) {
        (self.rect.0 - self.pad, self.rect.1 - self.pad)
    }

    fn panel(&self, kind: Tool, fill: bool) -> (Rect, Vec<(Slot, Rect)>) {
        let (slots, w) = layout(&row(kind, fill), self.scale);
        let x = self.rect.2 - w;
        let slots = slots.into_iter().map(|(slot, (sx, sy, sw, sh))| (slot, (sx + x, sy, sw, sh))).collect();
        ((x, 0, w, self.rect.3), slots)
    }

    fn slot(&self, button: Button, kind: Tool, fill: bool) -> Option<Rect> {
        self.panel(kind, fill).1.into_iter().find(|&(slot, _)| slot == Slot::Control(button)).map(|(_, r)| r)
    }

    pub fn hit(&self, (x, y): (i32, i32), kind: Tool, fill: bool) -> Option<Button> {
        let p = (x - self.rect.0, y - self.rect.1);
        self.panel(kind, fill).1.into_iter().find_map(|(slot, r)| match slot {
            Slot::Control(button) if overlay::contains(r, p) => Some(button),
            _ => None,
        })
    }

    fn popup_at(&self, anchor: Rect, (w, h): (i32, i32)) -> (i32, i32) {
        let (bx, _, bw, _) = anchor;
        let (mx, my, mw, mh) = self.monitor;
        let gap = px(TIP_GAP, self.scale);
        let x = (self.rect.0 + bx + bw / 2 - w / 2).min(mx + mw - w).max(mx);
        let below = self.rect.1 + self.rect.3 + gap;
        let y = if below + h <= my + mh { below } else { self.rect.1 - gap - h };
        (x, y)
    }

    pub fn tooltip(&mut self, button: Button, kind: Tool, fill: bool, vi: bool) -> Option<(&Pixmap, (i32, i32))> {
        let slot = self.slot(button, kind, fill)?;
        if self.tip.as_ref().is_none_or(|t| t.0 != button || t.1 != slot) {
            self.tip = self.render_tip(button, slot, vi);
        }
        self.tip.as_ref().map(|(_, _, pixmap, at)| (pixmap, *at))
    }

    fn render_tip(&self, button: Button, slot: Rect, vi: bool) -> Option<(Button, Rect, Pixmap, (i32, i32))> {
        let face = draw::font()?;
        let label = button.label(vi);
        let size = TIP_TEXT * self.scale;
        let (text_w, text_h) = draw::measure(face, size, label);
        let (pad_x, pad_y) = (TIP_PAD_X * self.scale, TIP_PAD_Y * self.scale);
        let (w, h) = ((text_w + pad_x * 2.0).ceil(), (text_h + pad_y * 2.0).ceil());
        let mut pixmap = Pixmap::new(w as u32, h as u32)?;
        let (radius, border) = (TIP_RADIUS * self.scale, BORDER_WIDTH * self.scale);
        fill(&mut pixmap, (0.0, 0.0, w, h), radius, BORDER, u8::MAX);
        fill(&mut pixmap, (border, border, w - border * 2.0, h - border * 2.0), radius - border, PANEL, u8::MAX);
        draw::text(&mut pixmap, face, Point::from_xy(pad_x, pad_y), &[label.to_string()], size, TIP_TEXT_COLOUR, None);
        let at = self.popup_at(slot, (w as i32, h as i32));
        Some((button, slot, pixmap, at))
    }

    fn popup(&self, target: Palette, kind: Tool, fill: bool) -> Option<(Rect, Vec<(Slot, Rect)>)> {
        let anchor = self.slot(target.button(), kind, fill)?;
        let (slots, w) = layout(&PALETTE_SLOTS, self.scale);
        let h = px(PADDING * 2.0 + BUTTON, self.scale);
        let (x, y) = self.popup_at(anchor, (w, h));
        Some(((x, y, w, h), slots))
    }

    pub fn palette(
        &mut self,
        target: Palette,
        kind: Tool,
        style: &Style,
        hover: Option<Button>,
    ) -> Option<(&Pixmap, (i32, i32))> {
        let key = PopupKey {
            target,
            kind,
            fill: style.stroke.fill,
            chosen: target.colour(kind, style),
            hover,
        };
        if self.popup.as_ref().is_none_or(|p| p.0 != key) {
            self.popup = self.render_palette(key, style).map(|(pixmap, at)| (key, pixmap, at));
        }
        self.popup.as_ref().map(|(_, pixmap, at)| (pixmap, *at))
    }

    fn render_palette(&self, key: PopupKey, style: &Style) -> Option<(Pixmap, (i32, i32))> {
        let PopupKey { target, kind, chosen, hover, .. } = key;
        let ((x, y, w, h), slots) = self.popup(target, kind, style.stroke.fill)?;
        let size = ((w + self.pad * 2) as u32, (h + self.pad * 2) as u32);
        let mut pixmap = Pixmap::new(size.0, size.1)?;
        let pad = self.pad as f32;
        paint_panel(&mut pixmap, (pad, pad, w as f32, h as f32), self.scale);
        for (slot, (sx, sy, sw, sh)) in slots {
            if let Slot::Control(button @ Button::Swatch(i)) = slot {
                let state = State { hovered: hover == Some(button), current: draw::PALETTE[i] == chosen, alpha: u8::MAX };
                let rect = (sx as f32 + pad, sy as f32 + pad, sw as f32, sh as f32);
                paint_control(&mut pixmap, self.scale, button, rect, state, style, kind);
            }
        }
        Some((pixmap, (x - self.pad, y - self.pad)))
    }

    pub fn palette_hit(&self, (x, y): (i32, i32), target: Palette, kind: Tool, fill: bool) -> Option<usize> {
        let ((px, py, ..), slots) = self.popup(target, kind, fill)?;
        slots.into_iter().find_map(|(slot, r)| match slot {
            Slot::Control(Button::Swatch(i)) if overlay::contains(r, (x - px, y - py)) => Some(i),
            _ => None,
        })
    }

    fn paint_base(&mut self, kind: Tool, filled: bool) {
        self.base.fill(Color::TRANSPARENT);
        let pad = self.pad as f32;
        let line = px(DIVIDER_WIDTH, self.scale).max(1) as f32;
        let ((x, y, w, h), slots) = self.panel(kind, filled);
        paint_panel(&mut self.base, (x as f32 + pad, y as f32 + pad, w as f32, h as f32), self.scale);
        for (_, (dx, dy, dw, dh)) in slots.into_iter().filter(|&(slot, _)| slot == Slot::Divider) {
            let mid = pad + dx as f32 + dw as f32 / 2.0;
            fill(&mut self.base, (mid - line / 2.0, pad + dy as f32, line, dh as f32), 0.0, BORDER, u8::MAX);
        }
        self.base_row = row(kind, filled);
    }

    pub fn render(&mut self, tool: Tool, kind: Tool, style: &Style, hover: Option<Button>, text_enabled: bool) -> &Pixmap {
        let filled = style.stroke.fill;
        if self.base_row != row(kind, filled) {
            self.paint_base(kind, filled);
        }
        self.pixmap.data_mut().copy_from_slice(self.base.data());
        let pad = self.pad as f32;
        for (slot, (x, y, w, h)) in self.panel(kind, filled).1 {
            let (x, y, w, h) = (x as f32 + pad, y as f32 + pad, w as f32, h as f32);
            match slot {
                Slot::Divider => {}
                Slot::Size => self.size_label(Point::from_xy(x + w / 2.0, y + h / 2.0), style.text.size),
                Slot::Control(button) => {
                    let enabled = text_enabled || button != Button::Tool(Tool::Text);
                    let state = State {
                        hovered: enabled && hover == Some(button),
                        current: button.current(tool, style),
                        alpha: if enabled { u8::MAX } else { DISABLED_ALPHA },
                    };
                    paint_control(&mut self.pixmap, self.scale, button, (x, y, w, h), state, style, kind);
                }
            }
        }
        &self.pixmap
    }

    fn size_label(&mut self, centre: Point, value: f32) {
        let Some(face) = draw::font() else { return };
        let label = format!("{value}");
        let size = SIZE_TEXT * self.scale;
        let (w, h) = draw::measure(face, size, &label);
        let at = Point::from_xy(centre.x - w / 2.0, centre.y - h / 2.0);
        draw::text(&mut self.pixmap, face, at, &[label], size, ICON, None);
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
