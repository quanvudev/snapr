use image::{RgbaImage, imageops};
use softbuffer::{Context, Surface};
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, MouseButton, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{Key, NamedKey};
use winit::platform::windows::WindowAttributesExtWindows;
use winit::window::{CursorIcon, Window, WindowId, WindowLevel};

const BORDER: u32 = 0x3090f0;

#[derive(Clone, Copy)]
pub enum Pick {
    Window,
    Area,
}

pub enum Outcome {
    Pending,
    Cancel,
    Save(Shot),
}

pub enum Shot {
    Ready(RgbaImage),
    Window { hwnd: isize, border: Option<[u8; 3]>, frame: RgbaImage, rect: Rect },
}

pub type Rect = (i32, i32, i32, i32);

pub struct Overlay {
    pick: Pick,
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
    frame: RgbaImage,
    dim: Vec<u32>,
    bright: Vec<u32>,
    windows: Vec<(xcap::Window, Rect)>,
    cursor: (i32, i32),
    drag: Option<(i32, i32)>,
    hover: Option<usize>,
}

fn desktop() -> Option<(i32, i32, RgbaImage)> {
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
    Some((ux, uy, canvas))
}

impl Overlay {
    pub fn open(event_loop: &ActiveEventLoop, pick: Pick) -> Option<Self> {
        let (mx, my, frame) = desktop()?;
        let (w, h) = frame.dimensions();
        if w == 0 || h == 0 {
            return None;
        }
        let windows = match pick {
            Pick::Window => visible_windows(mx, my),
            Pick::Area => Vec::new(),
        };
        let attrs = Window::default_attributes()
            .with_title("snapr")
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
            window,
            surface,
            frame,
            dim,
            bright,
            windows,
            cursor: (0, 0),
            drag: None,
            hover: None,
        })
    }

    pub fn id(&self) -> WindowId {
        self.window.id()
    }

    pub fn event(&mut self, event: WindowEvent) -> Outcome {
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
                    if r.2 >= 3 && r.3 >= 3 {
                        return crop(&self.frame, r).map_or(Outcome::Cancel, |img| Outcome::Save(Shot::Ready(img)));
                    }
                    self.window.request_redraw();
                    Outcome::Pending
                }
                (Pick::Window, ElementState::Pressed) => {
                    let Some(i) = self.hit() else { return Outcome::Pending };
                    let (win, r) = &self.windows[i];
                    let r = *r;
                    let hwnd = win.id().map_or(0, |id| id as i32 as isize);
                    let border = self.edge_color(r);
                    let frame = std::mem::take(&mut self.frame);
                    Outcome::Save(Shot::Window { hwnd, border, frame, rect: r })
                }
                (Pick::Window, ElementState::Released) => Outcome::Pending,
            },
            WindowEvent::RedrawRequested => self.draw().map_or(Outcome::Cancel, |_| Outcome::Pending),
            _ => Outcome::Pending,
        }
    }

    pub fn hovered(&self) -> Option<isize> {
        let (win, _) = &self.windows[self.hit()?];
        Some(win.id().map_or(0, |id| id as i32 as isize))
    }

    fn hit(&self) -> Option<usize> {
        let (x, y) = self.cursor;
        self.windows
            .iter()
            .position(|(_, r)| x >= r.0 && y >= r.1 && x < r.0 + r.2 && y < r.1 + r.3)
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
