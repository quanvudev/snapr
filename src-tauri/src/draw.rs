use crate::config::RedactStyle;
use crate::overlay;
use ab_glyph::{Font, FontVec, PxScale, ScaleFont, point};
use image::RgbaImage;
use std::sync::OnceLock;
use tiny_skia::{FillRule, IntSize, LineCap, Paint, PathBuilder, Pixmap, Point, PremultipliedColorU8, Rect, Stroke, StrokeDash, Transform};

const RED: [u8; 3] = [0xE5, 0x39, 0x35];
const ORANGE: [u8; 3] = [0xFB, 0x8C, 0x00];
const YELLOW: [u8; 3] = [0xFD, 0xD8, 0x35];
const GREEN: [u8; 3] = [0x43, 0xA0, 0x47];
const BLUE: [u8; 3] = [0x1E, 0x88, 0xE5];
const PURPLE: [u8; 3] = [0x8E, 0x24, 0xAA];
const BLACK: [u8; 3] = [0x00, 0x00, 0x00];
const WHITE: [u8; 3] = [0xFF, 0xFF, 0xFF];
pub const PALETTE: [[u8; 3]; 8] = [RED, ORANGE, YELLOW, GREEN, BLUE, PURPLE, BLACK, WHITE];
pub const WIDTHS: [f32; 3] = [2.0, 4.0, 8.0];
pub const SIZES: [f32; 6] = [12.0, 16.0, 20.0, 28.0, 36.0, 48.0];
const DEFAULT_WIDTH: f32 = WIDTHS[1];
const DEFAULT_SIZE: f32 = SIZES[2];
const HEAD_RATIO: f32 = 4.0;
const HEAD_SPREAD: f32 = 0.6;
const CARET_WIDTH: f32 = 2.0;
const MIN_DRAG: f32 = 3.0;
const SNAP_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
const HIT_SLOP: f32 = 4.0;
const MARK: [u8; 3] = [0x4C, 0xC2, 0xFF];
const MARK_PAD: f32 = 4.0;
const MARK_WIDTH: f32 = 1.5;
const MARK_DASH: f32 = 4.0;
const MARK_GAP: f32 = 3.0;
const FONTS_DIR: &str = "Fonts";
const REGULAR: usize = 0;
const BOLD: usize = 1;
const ITALIC: usize = 2;
const BOLD_ITALIC: usize = 3;
const PIXEL_MIN: f32 = 16.0;
const COVER_SLOP: f32 = 1.0;
const FONT_FILES: [&str; 4] = ["segoeui.ttf", "segoeuib.ttf", "segoeuii.ttf", "segoeuiz.ttf"];

static FONTS: [OnceLock<Option<FontVec>>; 4] = [const { OnceLock::new() }; 4];

pub fn system_font(files: &[&str]) -> Option<FontVec> {
    let dir = std::path::Path::new(&std::env::var_os("WINDIR")?).join(FONTS_DIR);
    files.iter().find_map(|file| FontVec::try_from_vec_and_index(std::fs::read(dir.join(file)).ok()?, 0).ok())
}

fn variant(slot: usize) -> Option<&'static FontVec> {
    FONTS[slot].get_or_init(|| system_font(&[FONT_FILES[slot]])).as_ref()
}

pub fn font() -> Option<&'static FontVec> {
    variant(REGULAR)
}

fn styled_font(bold: bool, italic: bool) -> Option<&'static FontVec> {
    let slot = match (bold, italic) {
        (false, false) => REGULAR,
        (true, false) => BOLD,
        (false, true) => ITALIC,
        (true, true) => BOLD_ITALIC,
    };
    variant(slot).or_else(font)
}

pub fn text_available() -> bool {
    FONTS[REGULAR].get().is_none_or(Option::is_some)
}

#[derive(Clone, Copy, PartialEq)]
pub struct StrokeStyle {
    pub colour: [u8; 3],
    pub width: f32,
    pub fill: bool,
    pub fill_colour: Option<[u8; 3]>,
}

impl Default for StrokeStyle {
    fn default() -> Self {
        Self { colour: RED, width: DEFAULT_WIDTH, fill: false, fill_colour: None }
    }
}

impl StrokeStyle {
    pub fn fill_rgb(&self) -> [u8; 3] {
        self.fill_colour.unwrap_or(self.colour)
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct TextStyle {
    pub colour: [u8; 3],
    pub size: f32,
    pub bold: bool,
    pub italic: bool,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self { colour: RED, size: DEFAULT_SIZE, bold: false, italic: false }
    }
}

#[derive(Clone, Copy, Default)]
pub struct Style {
    pub stroke: StrokeStyle,
    pub text: TextStyle,
    pub redact: RedactStyle,
}

impl Style {
    pub fn colour(&self, tool: Tool) -> [u8; 3] {
        match tool {
            Tool::Text => self.text.colour,
            _ => self.stroke.colour,
        }
    }
}

fn step(values: &[f32], current: f32, delta: isize) -> f32 {
    let i = values.iter().position(|&v| v == current).unwrap_or_default();
    values[i.saturating_add_signed(delta).min(values.len() - 1)]
}

fn constrain(tool: Tool, from: Point, to: Point) -> Point {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    match tool {
        Tool::Rect | Tool::Ellipse | Tool::Redact => {
            let side = dx.abs().max(dy.abs());
            Point::from_xy(from.x + side * dx.signum(), from.y + side * dy.signum())
        }
        Tool::Arrow => {
            let length = dx.hypot(dy);
            let angle = (dy.atan2(dx) / SNAP_ANGLE).round() * SNAP_ANGLE;
            Point::from_xy(from.x + length * angle.cos(), from.y + length * angle.sin())
        }
        Tool::Text => to,
    }
}

pub fn paint(rgb: [u8; 3], alpha: u8) -> Paint<'static> {
    let mut paint = Paint::default();
    paint.set_color_rgba8(rgb[0], rgb[1], rgb[2], alpha);
    paint
}

pub fn arrow(pixmap: &mut Pixmap, from: Point, to: Point, width: f32, paint: &Paint) {
    let length = from.distance(to);
    if length <= 0.0 {
        return;
    }
    let dir = Point::from_xy((to.x - from.x) / length, (to.y - from.y) / length);
    let head = (width * HEAD_RATIO).min(length);
    let spread = head * HEAD_SPREAD;
    let base = Point::from_xy(to.x - dir.x * head, to.y - dir.y * head);
    let mut shaft = PathBuilder::new();
    shaft.move_to(from.x, from.y);
    shaft.line_to(base.x, base.y);
    if let Some(path) = shaft.finish() {
        let stroke = Stroke { width, line_cap: LineCap::Round, ..Stroke::default() };
        pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
    }
    let mut tip = PathBuilder::new();
    tip.move_to(to.x, to.y);
    tip.line_to(base.x + dir.y * spread, base.y - dir.x * spread);
    tip.line_to(base.x - dir.y * spread, base.y + dir.x * spread);
    tip.close();
    if let Some(path) = tip.finish() {
        pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
    }
}

pub fn measure(face: &FontVec, size: f32, line: &str) -> (f32, f32) {
    let font = face.as_scaled(PxScale::from(size));
    let mut width = 0.0;
    let mut prev = None;
    for c in line.chars() {
        let id = font.glyph_id(c);
        if let Some(prev) = prev {
            width += font.kern(prev, id);
        }
        width += font.h_advance(id);
        prev = Some(id);
    }
    (width, font.height())
}

pub fn text(pixmap: &mut Pixmap, face: &FontVec, at: Point, lines: &[String], size: f32, rgb: [u8; 3], caret: Option<f32>) {
    let font = face.as_scaled(PxScale::from(size));
    let line_height = font.height() + font.line_gap();
    let mut end = at;
    for (i, line) in lines.iter().enumerate() {
        let top = at.y + i as f32 * line_height;
        let mut x = at.x;
        let mut prev = None;
        for c in line.chars() {
            let id = font.glyph_id(c);
            if let Some(prev) = prev {
                x += font.kern(prev, id);
            }
            let glyph = id.with_scale_and_position(font.scale(), point(x, top + font.ascent()));
            x += font.h_advance(id);
            prev = Some(id);
            if let Some(outline) = face.outline_glyph(glyph) {
                let min = outline.px_bounds().min;
                outline.draw(|gx, gy, coverage| blend(pixmap, min.x as i32 + gx as i32, min.y as i32 + gy as i32, coverage, rgb));
            }
        }
        end = Point::from_xy(x, top);
    }
    if let Some(width) = caret
        && let Some(bar) = Rect::from_xywh(end.x, end.y, width, font.height())
    {
        pixmap.fill_rect(bar, &paint(rgb, u8::MAX), Transform::identity(), None);
    }
}

fn blend(pixmap: &mut Pixmap, x: i32, y: i32, coverage: f32, rgb: [u8; 3]) {
    if x < 0 || y < 0 || x as u32 >= pixmap.width() || y as u32 >= pixmap.height() {
        return;
    }
    let i = (y as u32 * pixmap.width() + x as u32) as usize;
    let px = &mut pixmap.pixels_mut()[i];
    let coverage = coverage.min(1.0);
    let mix = |d: u8, s: u8| (d as f32 + (s as f32 - d as f32) * coverage).round() as u8;
    if let Some(c) = PremultipliedColorU8::from_rgba(
        mix(px.red(), rgb[0]),
        mix(px.green(), rgb[1]),
        mix(px.blue(), rgb[2]),
        mix(px.alpha(), u8::MAX),
    ) {
        *px = c;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Tool {
    Rect,
    Ellipse,
    Arrow,
    Text,
    Redact,
}

#[derive(Clone)]
enum Shape {
    Redact(Point, Point, RedactStyle),
    Rect(Point, Point, StrokeStyle),
    Ellipse(Point, Point, StrokeStyle),
    Arrow(Point, Point, StrokeStyle),
    Text { at: Point, lines: Vec<String>, style: TextStyle },
}

fn span(a: Point, b: Point) -> Option<Rect> {
    Rect::from_ltrb(a.x.min(b.x), a.y.min(b.y), a.x.max(b.x), a.y.max(b.y))
}

fn segment_distance(p: Point, a: Point, b: Point) -> f32 {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 { (((p.x - a.x) * dx + (p.y - a.y) * dy) / length).clamp(0.0, 1.0) } else { 0.0 };
    p.distance(Point::from_xy(a.x + t * dx, a.y + t * dy))
}

fn outline_distance(p: Point, r: Rect) -> f32 {
    let outside = (r.left() - p.x).max(p.x - r.right()).max(0.0).hypot((r.top() - p.y).max(p.y - r.bottom()).max(0.0));
    let inside = (p.x - r.left()).min(r.right() - p.x).min(p.y - r.top()).min(r.bottom() - p.y);
    if outside > 0.0 { outside } else { inside }
}

fn oval_distance(p: Point, r: Rect) -> Option<f32> {
    let (rx, ry) = (r.width() / 2.0, r.height() / 2.0);
    let centre = Point::from_xy(r.left() + rx, r.top() + ry);
    let n = ((p.x - centre.x) / rx).hypot((p.y - centre.y) / ry);
    (rx > 0.0 && ry > 0.0 && n > 0.0).then(|| (n - 1.0).abs() * p.distance(centre) / n)
}

fn text_bounds(at: Point, lines: &[String], style: &TextStyle, scale: f32) -> Option<Rect> {
    let face = styled_font(style.bold, style.italic)?;
    let size = style.size * scale;
    let font = face.as_scaled(PxScale::from(size));
    let width = lines.iter().map(|line| measure(face, size, line).0).fold(0.0, f32::max);
    let height = font.height() + (lines.len().max(1) - 1) as f32 * (font.height() + font.line_gap());
    Rect::from_xywh(at.x, at.y, width, height)
}

impl Shape {
    fn drawn(tool: Tool, from: Point, to: Point, style: &Style) -> Option<Self> {
        match tool {
            Tool::Rect => Some(Shape::Rect(from, to, style.stroke)),
            Tool::Ellipse => Some(Shape::Ellipse(from, to, style.stroke)),
            Tool::Arrow => Some(Shape::Arrow(from, to, style.stroke)),
            Tool::Redact => Some(Shape::Redact(from, to, style.redact)),
            Tool::Text => None,
        }
    }

    fn render(&self, pixmap: &mut Pixmap, scale: f32, background: &[u8]) {
        let pen = |style: &StrokeStyle| (paint(style.colour, u8::MAX), Stroke { width: style.width * scale, ..Stroke::default() });
        match self {
            Shape::Redact(a, b, style) => {
                if let Some(r) = span(*a, *b) {
                    redact(pixmap, background, r, *style);
                }
            }
            Shape::Rect(a, b, style) => {
                let (paint, stroke) = pen(style);
                if let Some(r) = span(*a, *b) {
                    let path = PathBuilder::from_rect(r);
                    if style.fill {
                        pixmap.fill_path(&path, &self::paint(style.fill_rgb(), u8::MAX), FillRule::Winding, Transform::identity(), None);
                    }
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Shape::Ellipse(a, b, style) => {
                let (paint, stroke) = pen(style);
                if let Some(path) = span(*a, *b).and_then(PathBuilder::from_oval) {
                    if style.fill {
                        pixmap.fill_path(&path, &self::paint(style.fill_rgb(), u8::MAX), FillRule::Winding, Transform::identity(), None);
                    }
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Shape::Arrow(a, b, style) => {
                let (paint, stroke) = pen(style);
                arrow(pixmap, *a, *b, stroke.width, &paint);
            }
            Shape::Text { at, lines, style } => text_box(pixmap, *at, lines, style, scale, false),
        }
    }

    fn hit(&self, p: Point, scale: f32) -> bool {
        let reach = |style: &StrokeStyle| (style.width / 2.0 + HIT_SLOP) * scale;
        match self {
            Shape::Redact(a, b, _) => {
                span(*a, *b).is_some_and(|r| p.x >= r.left() && p.x <= r.right() && p.y >= r.top() && p.y <= r.bottom())
            }
            Shape::Rect(a, b, style) => span(*a, *b).is_some_and(|r| {
                (style.fill && p.x >= r.left() && p.x <= r.right() && p.y >= r.top() && p.y <= r.bottom())
                    || outline_distance(p, r) <= reach(style)
            }),
            Shape::Ellipse(a, b, style) => span(*a, *b).is_some_and(|r| {
                let (rx, ry) = (r.width() / 2.0, r.height() / 2.0);
                let inside = rx > 0.0
                    && ry > 0.0
                    && ((p.x - r.left() - rx) / rx).powi(2) + ((p.y - r.top() - ry) / ry).powi(2) <= 1.0;
                (style.fill && inside) || oval_distance(p, r).is_some_and(|d| d <= reach(style))
            }),
            Shape::Arrow(a, b, style) => segment_distance(p, *a, *b) <= reach(style),
            Shape::Text { .. } => {
                let slop = HIT_SLOP * scale;
                self.bounds(scale)
                    .and_then(|r| r.outset(slop, slop))
                    .is_some_and(|r| p.x >= r.left() && p.x <= r.right() && p.y >= r.top() && p.y <= r.bottom())
            }
        }
    }

    fn bounds(&self, scale: f32) -> Option<Rect> {
        match self {
            Shape::Redact(a, b, _) => span(*a, *b),
            Shape::Rect(a, b, style) | Shape::Ellipse(a, b, style) | Shape::Arrow(a, b, style) => {
                let half = style.width * scale / 2.0;
                span(*a, *b)?.outset(half, half)
            }
            Shape::Text { at, lines, style } => text_bounds(*at, lines, style, scale),
        }
    }

    fn translate(&mut self, dx: f32, dy: f32) {
        let shift = |p: &mut Point| *p = Point::from_xy(p.x + dx, p.y + dy);
        match self {
            Shape::Rect(a, b, _) | Shape::Ellipse(a, b, _) | Shape::Arrow(a, b, _) | Shape::Redact(a, b, _) => {
                shift(a);
                shift(b);
            }
            Shape::Text { at, .. } => shift(at),
        }
    }

    fn kind(&self) -> Tool {
        match self {
            Shape::Rect(..) => Tool::Rect,
            Shape::Ellipse(..) => Tool::Ellipse,
            Shape::Arrow(..) => Tool::Arrow,
            Shape::Text { .. } => Tool::Text,
            Shape::Redact(..) => Tool::Redact,
        }
    }

    fn restyle(&mut self, style: &Style) {
        match self {
            Shape::Rect(.., s) | Shape::Ellipse(.., s) | Shape::Arrow(.., s) => *s = style.stroke,
            Shape::Text { style: s, .. } => *s = style.text,
            Shape::Redact(.., s) => *s = style.redact,
        }
    }

    fn styled(&self, mut style: Style) -> Style {
        match self {
            Shape::Rect(.., s) | Shape::Ellipse(.., s) | Shape::Arrow(.., s) => style.stroke = *s,
            Shape::Text { style: s, .. } => style.text = *s,
            Shape::Redact(.., s) => style.redact = *s,
        }
        style
    }
}

fn redact(pixmap: &mut Pixmap, background: &[u8], r: Rect, style: RedactStyle) {
    let (pw, ph) = (pixmap.width() as i32, pixmap.height() as i32);
    let x0 = (r.left().floor() as i32).clamp(0, pw);
    let y0 = (r.top().floor() as i32).clamp(0, ph);
    let x1 = (r.right().ceil() as i32).clamp(0, pw);
    let y1 = (r.bottom().ceil() as i32).clamp(0, ph);
    if x1 <= x0 || y1 <= y0 {
        return;
    }
    let block = match style {
        RedactStyle::Black => (x1 - x0).max(y1 - y0),
        RedactStyle::Pixelate => PIXEL_MIN.max(r.width().min(r.height()) / 2.0).round() as i32,
    }
    .max(1) as usize;
    let stride = pw as usize;
    let pixels = pixmap.pixels_mut();
    for by in (y0..y1).step_by(block) {
        for bx in (x0..x1).step_by(block) {
            let (ex, ey) = ((bx + block as i32).min(x1), (by + block as i32).min(y1));
            let rgb = match style {
                RedactStyle::Black => [0, 0, 0],
                RedactStyle::Pixelate => average(background, stride, (bx, by, ex, ey)),
            };
            let Some(colour) = PremultipliedColorU8::from_rgba(rgb[0], rgb[1], rgb[2], u8::MAX) else { continue };
            for y in by..ey {
                let row = y as usize * stride;
                pixels[row + bx as usize..row + ex as usize].fill(colour);
            }
        }
    }
}

pub fn redact_image(img: RgbaImage, rects: &[(f32, f32, f32, f32)], style: RedactStyle) -> RgbaImage {
    let (w, h) = img.dimensions();
    let Some(size) = IntSize::from_wh(w, h).filter(|_| !rects.is_empty()) else { return img };
    let background = img.into_raw();
    let Some(mut pixmap) = Pixmap::from_vec(background.clone(), size) else {
        return RgbaImage::from_raw(w, h, background).unwrap_or_default();
    };
    for r in rects.iter().filter_map(|&(x, y, rw, rh)| Rect::from_xywh(x, y, rw, rh)) {
        redact(&mut pixmap, &background, r, style);
    }
    RgbaImage::from_raw(w, h, pixmap.take()).unwrap_or_default()
}

fn average(background: &[u8], stride: usize, (x0, y0, x1, y1): (i32, i32, i32, i32)) -> [u8; 3] {
    let mut sum = [0u64; 3];
    let mut n = 0u64;
    for y in y0..y1 {
        for x in x0..x1 {
            let i = (y as usize * stride + x as usize) * 4;
            if let Some(px) = background.get(i..i + 3) {
                for (total, &v) in sum.iter_mut().zip(px) {
                    *total += u64::from(v);
                }
                n += 1;
            }
        }
    }
    let n = n.max(1);
    sum.map(|v| (v / n) as u8)
}

fn text_box(pixmap: &mut Pixmap, at: Point, lines: &[String], style: &TextStyle, scale: f32, caret: bool) {
    if let Some(face) = styled_font(style.bold, style.italic) {
        let caret = caret.then_some(CARET_WIDTH * scale);
        text(pixmap, face, at, lines, style.size * scale, style.colour, caret);
    }
}

#[derive(Clone, Copy)]
enum Drag {
    Draw { from: Point, to: Point },
    Move { index: usize, last: Point, moved: bool },
}

pub struct Sketch {
    selection: overlay::Rect,
    scale: f32,
    tool: Tool,
    style: Style,
    shapes: Vec<Shape>,
    history: Vec<Vec<Shape>>,
    redo: Vec<Vec<Shape>>,
    selected: Option<usize>,
    drag: Option<Drag>,
    text: Option<(Point, Vec<String>)>,
    background: Vec<u8>,
    base: Pixmap,
    view: Pixmap,
    dirty: bool,
}

impl Sketch {
    pub fn new(frame: &RgbaImage, selection: overlay::Rect, scale: f32, style: Style) -> Option<Self> {
        let crop = overlay::crop(frame, selection)?;
        let size = IntSize::from_wh(crop.width(), crop.height())?;
        let background = crop.into_raw();
        let base = Pixmap::from_vec(background.clone(), size)?;
        Some(Self {
            selection,
            scale,
            tool: Tool::Rect,
            style,
            shapes: Vec::new(),
            history: Vec::new(),
            redo: Vec::new(),
            selected: None,
            drag: None,
            text: None,
            background,
            view: base.clone(),
            base,
            dirty: false,
        })
    }

    pub fn selection(&self) -> overlay::Rect {
        self.selection
    }

    pub fn reselect(&mut self, frame: &RgbaImage, selection: overlay::Rect) -> Option<()> {
        let crop = overlay::crop(frame, selection)?;
        let size = IntSize::from_wh(crop.width(), crop.height())?;
        let background = crop.into_raw();
        self.base = Pixmap::from_vec(background.clone(), size)?;
        self.view = self.base.clone();
        self.background = background;
        let (dx, dy) = ((self.selection.0 - selection.0) as f32, (self.selection.1 - selection.1) as f32);
        let shift = |p: &mut Point| *p = Point::from_xy(p.x + dx, p.y + dy);
        for shape in self.shapes.iter_mut().chain(self.history.iter_mut().chain(self.redo.iter_mut()).flatten()) {
            shape.translate(dx, dy);
        }
        if let Some((at, _)) = self.text.as_mut() {
            shift(at);
        }
        match self.drag.as_mut() {
            Some(Drag::Draw { from, to }) => {
                shift(from);
                shift(to);
            }
            Some(Drag::Move { last, .. }) => shift(last),
            None => {}
        }
        self.selection = selection;
        self.dirty = true;
        Some(())
    }

    pub fn tool(&self) -> Tool {
        self.tool
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn options(&self) -> (Tool, Style) {
        match self.selected.and_then(|i| self.shapes.get(i)) {
            Some(shape) => (shape.kind(), shape.styled(self.style)),
            None => (self.tool, self.style),
        }
    }

    pub fn editing(&self) -> bool {
        self.text.is_some()
    }

    fn restyle(&mut self, stroke: impl FnOnce(&mut StrokeStyle), text: impl FnOnce(&mut TextStyle)) {
        let (kind, mut style) = self.options();
        let before = (style.stroke, style.text);
        match kind {
            Tool::Redact => return,
            Tool::Text => text(&mut style.text),
            _ => stroke(&mut style.stroke),
        }
        if let Some(i) = self.selected
            && before != (style.stroke, style.text)
        {
            self.snapshot();
            self.shapes[i].restyle(&style);
            self.dirty = true;
        }
        self.style = style;
    }

    pub fn set_colour(&mut self, colour: [u8; 3]) {
        self.restyle(|s| s.colour = colour, |t| t.colour = colour);
    }

    pub fn set_width(&mut self, width: f32) {
        self.restyle(|s| s.width = width, |_| {});
    }

    pub fn step(&mut self, delta: isize) {
        self.restyle(|s| s.width = step(&WIDTHS, s.width, delta), |t| t.size = step(&SIZES, t.size, delta));
    }

    fn restyle_fill(&mut self, fill: impl FnOnce(&mut StrokeStyle)) {
        if matches!(self.options().0, Tool::Rect | Tool::Ellipse) {
            self.restyle(fill, |_| {});
        }
    }

    pub fn set_fill(&mut self, on: bool) {
        self.restyle_fill(|s| s.fill = on);
    }

    pub fn toggle_fill(&mut self) {
        self.restyle_fill(|s| s.fill = !s.fill);
    }

    pub fn set_fill_colour(&mut self, colour: [u8; 3]) {
        self.restyle_fill(|s| {
            s.fill_colour = Some(colour);
            s.fill = true;
        });
    }

    pub fn toggle_bold(&mut self) {
        self.restyle(|_| {}, |t| t.bold = !t.bold);
    }

    pub fn toggle_italic(&mut self) {
        self.restyle(|_| {}, |t| t.italic = !t.italic);
    }

    pub fn set_redact(&mut self, redact: RedactStyle) {
        let (kind, mut style) = self.options();
        if kind != Tool::Redact {
            return;
        }
        if let Some(i) = self.selected
            && style.redact != redact
        {
            self.snapshot();
            style.redact = redact;
            self.shapes[i].restyle(&style);
            self.dirty = true;
        }
        style.redact = redact;
        self.style = style;
    }

    pub fn redactions(&mut self, rects: &[(f32, f32, f32, f32)]) -> usize {
        let covers = |shape: &Shape, r: Rect| match shape {
            Shape::Redact(a, b, _) => span(*a, *b)
                .and_then(|c| c.outset(COVER_SLOP, COVER_SLOP))
                .is_some_and(|c| c.left() <= r.left() && c.top() <= r.top() && c.right() >= r.right() && c.bottom() >= r.bottom()),
            _ => false,
        };
        let fresh: Vec<Shape> = rects
            .iter()
            .filter_map(|&(x, y, w, h)| Rect::from_xywh(x, y, w, h))
            .filter(|&r| !self.shapes.iter().any(|shape| covers(shape, r)))
            .map(|r| Shape::Redact(Point::from_xy(r.left(), r.top()), Point::from_xy(r.right(), r.bottom()), self.style.redact))
            .collect();
        let added = fresh.len();
        if added > 0 {
            self.snapshot();
            self.shapes.extend(fresh);
            self.dirty = true;
        }
        added
    }

    fn local(&self, (x, y): (i32, i32)) -> Point {
        Point::from_xy((x - self.selection.0) as f32, (y - self.selection.1) as f32)
    }

    fn shape_at(&self, p: Point) -> Option<usize> {
        self.shapes.iter().rposition(|shape| shape.hit(p, self.scale))
    }

    pub fn grabs(&self, at: (i32, i32)) -> bool {
        matches!(self.drag, Some(Drag::Move { .. })) || self.shape_at(self.local(at)).is_some()
    }

    pub fn press(&mut self, at: (i32, i32)) {
        let p = self.local(at);
        if let Some(index) = self.shape_at(p) {
            self.commit_text();
            self.selected = Some(index);
            self.drag = Some(Drag::Move { index, last: p, moved: false });
            return;
        }
        self.selected = None;
        if self.tool == Tool::Text {
            self.commit_text();
            self.text = Some((p, vec![String::new()]));
        } else {
            self.drag = Some(Drag::Draw { from: p, to: p });
        }
    }

    pub fn drag_to(&mut self, at: (i32, i32), shift: bool) -> bool {
        let p = self.local(at);
        match self.drag {
            Some(Drag::Draw { from, .. }) => {
                let to = if shift { constrain(self.tool, from, p) } else { p };
                self.drag = Some(Drag::Draw { from, to });
            }
            Some(Drag::Move { index, last, moved }) => {
                if p == last {
                    return false;
                }
                if !moved {
                    self.snapshot();
                }
                self.shapes[index].translate(p.x - last.x, p.y - last.y);
                self.drag = Some(Drag::Move { index, last: p, moved: true });
                self.dirty = true;
            }
            None => return false,
        }
        true
    }

    pub fn release(&mut self, at: (i32, i32), shift: bool) {
        self.drag_to(at, shift);
        if let Some(Drag::Draw { from, to }) = self.drag.take()
            && from.distance(to) >= MIN_DRAG
            && let Some(shape) = Shape::drawn(self.tool, from, to, &self.style)
        {
            self.push(shape);
        }
    }

    pub fn type_text(&mut self, s: &str) {
        if let Some(line) = self.text.as_mut().and_then(|(_, lines)| lines.last_mut()) {
            line.push_str(s);
        }
    }

    pub fn backspace(&mut self) {
        let Some((_, lines)) = self.text.as_mut() else { return };
        if lines.len() > 1 && lines.last().is_some_and(String::is_empty) {
            lines.pop();
        } else if let Some(line) = lines.last_mut() {
            line.pop();
        }
    }

    pub fn newline(&mut self) {
        if let Some((_, lines)) = self.text.as_mut() {
            lines.push(String::new());
        }
    }

    pub fn commit_text(&mut self) {
        if let Some((at, lines)) = self.text.take()
            && lines.iter().any(|line| !line.is_empty())
        {
            self.push(Shape::Text { at, lines, style: self.style.text });
        }
    }

    pub fn undo(&mut self) {
        if self.drag.is_some() || self.text.take().is_some() {
            return;
        }
        if let Some(shapes) = self.history.pop() {
            self.redo.push(std::mem::replace(&mut self.shapes, shapes));
            self.selected = self.selected.filter(|&i| i < self.shapes.len());
            self.dirty = true;
        }
    }

    pub fn redo(&mut self) {
        if self.drag.is_some() || self.text.is_some() {
            return;
        }
        if let Some(shapes) = self.redo.pop() {
            self.history.push(std::mem::replace(&mut self.shapes, shapes));
            self.selected = self.selected.filter(|&i| i < self.shapes.len());
            self.dirty = true;
        }
    }

    pub fn can_undo(&self) -> bool {
        self.text.is_some() || !self.history.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        self.text.is_none() && !self.redo.is_empty()
    }

    pub fn delete(&mut self) {
        if self.drag.is_some() {
            return;
        }
        if let Some(i) = self.selected.take() {
            self.snapshot();
            self.shapes.remove(i);
            self.dirty = true;
        }
    }

    pub fn deselect(&mut self) -> bool {
        self.selected.take().is_some()
    }

    pub fn set_tool(&mut self, tool: Tool) {
        if tool == Tool::Text && font().is_none() {
            return;
        }
        self.commit_text();
        self.tool = tool;
    }

    fn snapshot(&mut self) {
        self.history.push(self.shapes.clone());
        self.redo.clear();
    }

    fn push(&mut self, shape: Shape) {
        self.snapshot();
        self.shapes.push(shape);
        self.dirty = true;
    }

    fn refresh(&mut self) {
        if !self.dirty {
            return;
        }
        self.base.data_mut().copy_from_slice(&self.background);
        for shape in &self.shapes {
            shape.render(&mut self.base, self.scale, &self.background);
        }
        self.dirty = false;
    }

    pub fn view(&mut self) -> &Pixmap {
        self.refresh();
        self.view.data_mut().copy_from_slice(self.base.data());
        if let Some(Drag::Draw { from, to }) = self.drag
            && let Some(shape) = Shape::drawn(self.tool, from, to, &self.style)
        {
            shape.render(&mut self.view, self.scale, &self.background);
        }
        if let Some((at, lines)) = &self.text {
            text_box(&mut self.view, *at, lines, &self.style.text, self.scale, true);
        }
        if let Some(bounds) = self.selected.and_then(|i| self.shapes[i].bounds(self.scale)) {
            let pad = MARK_PAD * self.scale;
            let dash = StrokeDash::new(vec![MARK_DASH * self.scale, MARK_GAP * self.scale], 0.0);
            let stroke = Stroke { width: MARK_WIDTH * self.scale, dash, ..Stroke::default() };
            if let Some(r) = bounds.outset(pad, pad) {
                let path = PathBuilder::from_rect(r);
                self.view.stroke_path(&path, &paint(MARK, u8::MAX), &stroke, Transform::identity(), None);
            }
        }
        &self.view
    }

    pub fn finish(mut self) -> Option<RgbaImage> {
        self.commit_text();
        self.refresh();
        let (w, h) = (self.base.width(), self.base.height());
        RgbaImage::from_raw(w, h, self.base.take())
    }
}
