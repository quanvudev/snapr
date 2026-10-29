use crate::overlay::Rect;
use crate::{draw, ring, toolbar};
use tiny_skia::{Pixmap, Point};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromPoint};
use windows_sys::Win32::UI::HiDpi::{GetDpiForMonitor, MDT_EFFECTIVE_DPI};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, SW_SHOWNOACTIVATE, SWP_FRAMECHANGED, SWP_NOACTIVATE, SetWindowDisplayAffinity,
    SetWindowLongPtrW, SetWindowPos, ShowWindow, WDA_EXCLUDEFROMCAPTURE, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT,
};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::platform::windows::WindowAttributesExtWindows;
use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};
use winit::window::{Window, WindowLevel};

const EDGE: i32 = 2;
const EDGE_COLOUR: [u8; 3] = [0x30, 0x90, 0xf0];
const DIGIT_SIDE: f32 = 160.0;
const DIGIT_RADIUS: f32 = 24.0;
const DIGIT_TEXT: f32 = 96.0;
const DIGIT_FILL: [u8; 3] = [0x20, 0x20, 0x20];
const DIGIT_ALPHA: u8 = 0xe0;
const WHITE: [u8; 3] = [0xff, 0xff, 0xff];

pub(crate) fn monitor_at((x, y): (i32, i32)) -> Option<(isize, Rect, f32)> {
    let mut info: MONITORINFO = unsafe { std::mem::zeroed() };
    info.cbSize = size_of::<MONITORINFO>() as u32;
    let monitor = unsafe { MonitorFromPoint(POINT { x, y }, MONITOR_DEFAULTTONEAREST) };
    if unsafe { GetMonitorInfoW(monitor, &mut info) } == 0 {
        return None;
    }
    let (mut dpi_x, mut dpi_y) = (96, 96);
    let dpi_ok = unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y) } >= 0;
    let scale = if dpi_ok { dpi_x as f32 / 96.0 } else { 1.0 };
    let r = info.rcMonitor;
    Some((monitor as isize, (r.left, r.top, r.right - r.left, r.bottom - r.top), scale))
}

struct Layered {
    _window: Window,
    hwnd: isize,
}

impl Layered {
    fn open(event_loop: &ActiveEventLoop, (x, y, w, h): Rect) -> Option<Self> {
        let attrs = Window::default_attributes()
            .with_title("Snapr")
            .with_decorations(false)
            .with_undecorated_shadow(false)
            .with_resizable(false)
            .with_active(false)
            .with_visible(false)
            .with_window_level(WindowLevel::AlwaysOnTop)
            .with_position(PhysicalPosition::new(x, y))
            .with_inner_size(PhysicalSize::new(w as u32, h as u32))
            .with_skip_taskbar(true);
        let window = event_loop.create_window(attrs).ok()?;
        let Ok(RawWindowHandle::Win32(handle)) = window.window_handle().map(|h| h.as_raw()) else { return None };
        let hwnd = handle.hwnd.get();
        unsafe {
            let style = GetWindowLongPtrW(hwnd as _, GWL_EXSTYLE);
            let extra = WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
            SetWindowLongPtrW(hwnd as _, GWL_EXSTYLE, style | extra as isize);
            SetWindowPos(hwnd as _, HWND_TOPMOST, x, y, w, h, SWP_NOACTIVATE | SWP_FRAMECHANGED);
            SetWindowDisplayAffinity(hwnd as _, WDA_EXCLUDEFROMCAPTURE);
        }
        Some(Self { _window: window, hwnd })
    }

    fn show(&self, pixmap: &Pixmap) -> Option<()> {
        ring::present(self.hwnd, pixmap).ok()?;
        unsafe { ShowWindow(self.hwnd as _, SW_SHOWNOACTIVATE) };
        Some(())
    }
}

pub(crate) struct Frame {
    _layered: Layered,
}

impl Frame {
    pub(crate) fn open(event_loop: &ActiveEventLoop, (x, y, w, h): Rect) -> Option<Self> {
        let outer = (x - EDGE, y - EDGE, w + 2 * EDGE, h + 2 * EDGE);
        let layered = Layered::open(event_loop, outer)?;
        let mut pixmap = Pixmap::new(outer.2 as u32, outer.3 as u32)?;
        let (ow, oh, e) = (outer.2 as f32, outer.3 as f32, EDGE as f32);
        for strip in [(0.0, 0.0, ow, e), (0.0, oh - e, ow, e), (0.0, e, e, oh - 2.0 * e), (ow - e, e, e, oh - 2.0 * e)] {
            toolbar::fill(&mut pixmap, strip, 0.0, EDGE_COLOUR, u8::MAX);
        }
        layered.show(&pixmap)?;
        Some(Self { _layered: layered })
    }
}

pub(crate) struct Digit {
    layered: Layered,
    side: u32,
    scale: f32,
}

impl Digit {
    pub(crate) fn open(event_loop: &ActiveEventLoop, (x, y, w, h): Rect, value: u32) -> Option<Self> {
        let centre = (x + w / 2, y + h / 2);
        let (_, (mx, my, mw, mh), scale) = monitor_at(centre)?;
        let side = (DIGIT_SIDE * scale).round().max(1.0) as i32;
        let left = (centre.0 - side / 2).clamp(mx, (mx + mw - side).max(mx));
        let top = (centre.1 - side / 2).clamp(my, (my + mh - side).max(my));
        let layered = Layered::open(event_loop, (left, top, side, side))?;
        let digit = Self { layered, side: side as u32, scale };
        digit.show(value)?;
        Some(digit)
    }

    pub(crate) fn show(&self, value: u32) -> Option<()> {
        let mut pixmap = Pixmap::new(self.side, self.side)?;
        let side = self.side as f32;
        toolbar::fill(&mut pixmap, (0.0, 0.0, side, side), DIGIT_RADIUS * self.scale, DIGIT_FILL, DIGIT_ALPHA);
        if let Some(face) = draw::font() {
            let label = value.to_string();
            let size = DIGIT_TEXT * self.scale;
            let (tw, th) = draw::measure(face, size, &label);
            let corner = Point::from_xy((side - tw) / 2.0, (side - th) / 2.0);
            draw::text(&mut pixmap, face, corner, &[label], size, WHITE, None);
        }
        self.layered.show(&pixmap)
    }
}
