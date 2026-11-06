use crate::config::Config;
use image::imageops::{self, FilterType};
use image::{DynamicImage, RgbaImage};
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use std::ffi::c_void;
use windows_sys::Win32::Foundation::{HWND, POINT, RECT};
use windows_sys::Win32::Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute};
use windows_sys::Win32::Graphics::Gdi::{
    BI_RGB, BITMAPINFO, BITMAPINFOHEADER, CreateCompatibleBitmap, CreateCompatibleDC, DIB_RGB_COLORS, DeleteDC,
    DeleteObject, GetDIBits, GetWindowDC, HBITMAP, HDC, ReleaseDC, SelectObject,
};
use windows_sys::Win32::Storage::Xps::PrintWindow;
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetWindowRect, IsZoomed, SPI_GETDESKWALLPAPER, SystemParametersInfoW,
};
use xcap::Monitor;

pub fn fullscreen() -> Result<RgbaImage, Box<dyn Error>> {
    Ok(cursor_monitor()?.capture_image()?)
}

pub fn cursor_monitor() -> Result<Monitor, Box<dyn Error>> {
    let mut p = POINT { x: 0, y: 0 };
    let found = unsafe { GetCursorPos(&mut p) } != 0;
    let monitor = found
        .then(|| Monitor::from_point(p.x, p.y).ok())
        .flatten()
        .or_else(|| Monitor::all().ok()?.into_iter().find(|m| m.is_primary().unwrap_or(false)))
        .ok_or("no monitor")?;
    Ok(monitor)
}

pub fn pad(img: &RgbaImage, p: u32, bg: Option<&DynamicImage>) -> RgbaImage {
    let (w, h) = (img.width() + 2 * p, img.height() + 2 * p);
    let mut canvas = match bg {
        Some(bg) => bg.resize_to_fill(w, h, FilterType::Triangle).into_rgba8(),
        None => RgbaImage::new(w, h),
    };
    imageops::overlay(&mut canvas, img, p as i64, p as i64);
    canvas
}

pub fn write(img: &RgbaImage, cfg: &Config) -> Result<PathBuf, Box<dyn Error>> {
    let dir = crate::library::images(cfg);
    std::fs::create_dir_all(&dir)?;
    let path = unique_path(&dir, "png");
    img.save(&path)?;
    Ok(path)
}

pub fn copy(img: &RgbaImage) -> Result<(), Box<dyn Error>> {
    arboard::Clipboard::new()?.set_image(arboard::ImageData {
        width: img.width() as usize,
        height: img.height() as usize,
        bytes: img.as_raw().into(),
    })?;
    Ok(())
}

#[derive(Default)]
pub struct BackgroundCache(Option<(String, SystemTime, DynamicImage)>);

impl BackgroundCache {
    pub fn get(&mut self, path: &str) -> Result<Option<&DynamicImage>, Box<dyn Error>> {
        if path.is_empty() {
            self.0 = None;
            return Ok(None);
        }
        let modified = match std::fs::metadata(path).and_then(|m| m.modified()) {
            Ok(modified) => modified,
            Err(e) => {
                self.0 = None;
                return Err(e.into());
            }
        };
        let fresh = matches!(&self.0, Some((p, m, _)) if p == path && *m == modified);
        if !fresh {
            self.0 = None;
            self.0 = Some((path.to_string(), modified, image::ImageReader::open(path)?.with_guessed_format()?.decode()?));
        }
        Ok(self.0.as_ref().map(|(_, _, img)| img))
    }
}

pub fn wallpaper() -> String {
    let mut buf = [0u16; 260];
    let ok = unsafe { SystemParametersInfoW(SPI_GETDESKWALLPAPER, buf.len() as u32, buf.as_mut_ptr() as *mut c_void, 0) };
    if ok == 0 {
        return String::new();
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..len])
}

pub fn unique_path(dir: &Path, ext: &str) -> PathBuf {
    let stamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    stem_path(dir, &stamp, ext)
}

pub fn stem_path(dir: &Path, stem: &str, ext: &str) -> PathBuf {
    let mut path = dir.join(format!("{stem}.{ext}"));
    let mut n = 0;
    while path.exists() {
        n += 1;
        path = dir.join(format!("{stem}_{n}.{ext}"));
    }
    path
}

pub fn window(hwnd: isize, border: Option<[u8; 3]>) -> Option<RgbaImage> {
    let hwnd = hwnd as HWND;
    let mut rect: RECT = unsafe { std::mem::zeroed() };
    let mut bounds = rect;
    unsafe {
        if GetWindowRect(hwnd, &mut rect) == 0 {
            return None;
        }
        let frame = DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS as u32,
            (&mut bounds as *mut RECT).cast(),
            size_of::<RECT>() as u32,
        );
        if frame != 0 {
            bounds = rect;
        }
    }
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    if w <= 0 || h <= 0 {
        return None;
    }
    let raw = unsafe { grab(hwnd, w, h) }?;
    let ox = (bounds.left - rect.left).max(0);
    let oy = (bounds.top - rect.top).max(0);
    let cw = (bounds.right - bounds.left).min(w - ox);
    let ch = (bounds.bottom - bounds.top).min(h - oy);
    if cw <= 0 || ch <= 0 {
        return None;
    }
    let mut lit = false;
    let mut img = RgbaImage::from_fn(cw as u32, ch as u32, |x, y| {
        let i = (((oy as u32 + y) * w as u32 + ox as u32 + x) * 4) as usize;
        let (b, g, r) = (raw[i], raw[i + 1], raw[i + 2]);
        lit |= (b | g | r) != 0;
        image::Rgba([r, g, b, 255])
    });
    if !lit {
        return None;
    }
    if unsafe { IsZoomed(hwnd) } == 0 {
        let dpi = unsafe { GetDpiForWindow(hwnd) };
        let radius = if dpi == 0 { 8.0 } else { (8 * dpi) as f32 / 96.0 };
        round_corners(&mut img, radius.round(), border.unwrap_or([0x3c, 0x3c, 0x3c]));
    }
    Some(img)
}

pub fn mask_corners(img: &mut RgbaImage, radius: f32) {
    let (hw, hh) = (img.width() as f32 / 2.0, img.height() as f32 / 2.0);
    let radius = radius.min(hw.min(hh));
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (px, py) = ((x as f32 + 0.5 - hw).abs() - (hw - radius), (y as f32 + 0.5 - hh).abs() - (hh - radius));
        let d = px.max(0.0).hypot(py.max(0.0)) + px.max(py).min(0.0) - radius;
        p[3] = (p[3] as f32 * (0.5 - d).clamp(0.0, 1.0)).round() as u8;
    }
}

fn round_corners(img: &mut RgbaImage, radius: f32, border: [u8; 3]) {
    let (hw, hh) = (img.width() as f32 / 2.0, img.height() as f32 / 2.0);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (px, py) = ((x as f32 + 0.5 - hw).abs() - (hw - radius), (y as f32 + 0.5 - hh).abs() - (hh - radius));
        let d = px.max(0.0).hypot(py.max(0.0)) + px.max(py).min(0.0) - radius;
        let alpha = (0.5 - d).clamp(0.0, 1.0);
        let stroke = (d + 1.5).clamp(0.0, 1.0);
        for i in 0..3 {
            let mixed = p[i] as f32 * (1.0 - stroke) + border[i] as f32 * stroke;
            p[i] = if alpha > 0.0 { mixed.round() as u8 } else { 0 };
        }
        p[3] = (alpha * 255.0).round() as u8;
    }
}

pub unsafe fn dib(dc: HDC, bmp: HBITMAP, w: i32, h: i32) -> Option<Vec<u8>> {
    unsafe {
        let mut bmi: BITMAPINFO = std::mem::zeroed();
        bmi.bmiHeader = BITMAPINFOHEADER {
            biSize: size_of::<BITMAPINFOHEADER>() as u32,
            biWidth: w,
            biHeight: -h,
            biPlanes: 1,
            biBitCount: 32,
            biCompression: BI_RGB,
            ..std::mem::zeroed()
        };
        let mut buf = vec![0u8; (w as usize) * (h as usize) * 4];
        let lines = GetDIBits(dc, bmp, 0, h as u32, buf.as_mut_ptr().cast::<c_void>(), &mut bmi, DIB_RGB_COLORS);
        (lines != 0).then_some(buf)
    }
}

unsafe fn grab(hwnd: HWND, w: i32, h: i32) -> Option<Vec<u8>> {
    unsafe {
        let window_dc = GetWindowDC(hwnd);
        if window_dc.is_null() {
            return None;
        }
        let mem = CreateCompatibleDC(window_dc);
        let bmp = CreateCompatibleBitmap(window_dc, w, h);
        let mut out = None;
        if !mem.is_null() && !bmp.is_null() {
            let old = SelectObject(mem, bmp);
            let printed = PrintWindow(hwnd, mem, 2) != 0 || PrintWindow(hwnd, mem, 0) != 0;
            SelectObject(mem, old);
            if printed {
                out = dib(window_dc, bmp, w, h);
            }
        }
        if !bmp.is_null() {
            DeleteObject(bmp);
        }
        if !mem.is_null() {
            DeleteDC(mem);
        }
        ReleaseDC(hwnd, window_dc);
        out
    }
}
