mod paddle;

use std::ops::Range;
use std::path::PathBuf;
use std::sync::{Mutex, PoisonError};

use image::imageops::{FilterType, grayscale, resize};
use image::{GrayImage, RgbaImage};
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::{OcrEngine, OcrResult};
use windows::Security::Cryptography::CryptographicBuffer;

use crate::config::PaddleSize;

type Row = (f32, f32, Vec<(f32, String)>);

pub type Bounds = (f32, f32, f32, f32);

pub struct OcrLine {
    pub text: String,
    pub rect: Bounds,
    pub spans: Vec<(Range<usize>, Bounds)>,
}

const QR_MAX: u32 = 2000;

static ENGINE: Mutex<Option<(PaddleSize, paddle::Engine)>> = Mutex::new(None);

pub struct PaddleModel {
    pub size: PaddleSize,
    pub det: PathBuf,
    pub rec: PathBuf,
    pub dict: PathBuf,
    pub runtime: PathBuf,
    pub params: (f32, f32, f32),
}

pub fn windows_available() -> bool {
    OcrEngine::TryCreateFromUserProfileLanguages().is_ok()
}

pub fn available(paddle: Option<&PaddleModel>) -> bool {
    paddle.is_some() || windows_available()
}

pub fn qr(img: &RgbaImage) -> Vec<String> {
    let luma = grayscale(img);
    let longest = luma.width().max(luma.height());
    let luma: GrayImage = if longest > QR_MAX {
        let scale = QR_MAX as f64 / longest as f64;
        let w = ((luma.width() as f64 * scale) as u32).max(1);
        let h = ((luma.height() as f64 * scale) as u32).max(1);
        resize(&luma, w, h, FilterType::Triangle)
    } else {
        luma
    };
    let mut prepared = rqrr::PreparedImage::prepare(luma);
    let mut codes: Vec<(i32, String)> = prepared
        .detect_grids()
        .into_iter()
        .filter_map(|grid| {
            let top = grid.bounds.iter().map(|p| p.y).min().unwrap_or(0);
            grid.decode().ok().map(|(_, text)| (top, text))
        })
        .filter(|(_, text)| !text.trim().is_empty())
        .collect();
    codes.sort_by_key(|(top, _)| *top);
    codes.into_iter().map(|(_, text)| text).collect()
}

pub fn extract(img: &RgbaImage, paddle: Option<&PaddleModel>) -> Result<String, String> {
    match paddle.map(|model| paddle_extract(img, model)) {
        Some(Ok(text)) => Ok(text),
        Some(Err(e)) => recognize(img).map_err(|_| e),
        None => recognize(img).map_err(|e| e.message()),
    }
}

pub fn words(img: &RgbaImage, paddle: Option<&PaddleModel>, prefer_paddle: bool) -> Result<Vec<OcrLine>, String> {
    if prefer_paddle
        && let Some(model) = paddle
    {
        return paddle_lines(img, model).or_else(|e| windows_words(img).map_err(|_| e));
    }
    let windows = if windows_available() { Some(windows_words(img).map_err(|e| e.message())) } else { None };
    match (windows, paddle) {
        (Some(Ok(lines)), _) => Ok(lines),
        (Some(Err(_)) | None, Some(model)) => paddle_lines(img, model),
        (Some(Err(e)), None) => Err(e),
        (None, None) => windows_words(img).map_err(|e| e.message()),
    }
}

fn paddle_lines(img: &RgbaImage, model: &PaddleModel) -> Result<Vec<OcrLine>, String> {
    let lines = paddle_run(img, model)?;
    Ok(lines.into_iter().map(|(x, y, w, h, text)| OcrLine { text, rect: (x, y, w, h), spans: Vec::new() }).collect())
}

fn paddle_run(img: &RgbaImage, model: &PaddleModel) -> Result<Vec<paddle::Line>, String> {
    let mut cache = ENGINE.lock().unwrap_or_else(PoisonError::into_inner);
    let engine = match &mut *cache {
        Some((size, engine)) if *size == model.size => engine,
        slot => {
            *slot = None;
            let engine = paddle::Engine::load(&model.det, &model.rec, &model.dict, &model.runtime)?;
            &mut slot.insert((model.size, engine)).1
        }
    };
    engine.extract(img, model.params)
}

pub fn paddle_extract(img: &RgbaImage, model: &PaddleModel) -> Result<String, String> {
    let lines = paddle_run(img, model)?;
    Ok(rows(lines.into_iter().map(|(x, y, _, h, text)| (x, y, y + h, text)).collect()))
}

pub fn paddle_unload(size: Option<PaddleSize>) {
    let mut cache = ENGINE.lock().unwrap_or_else(PoisonError::into_inner);
    if size.is_none() || cache.as_ref().map(|(cached, _)| *cached) == size {
        *cache = None;
    }
}

fn windows_words(img: &RgbaImage) -> windows::core::Result<Vec<OcrLine>> {
    let (result, back) = windows_ocr(img)?;
    let mut lines = Vec::new();
    for line in result.Lines()? {
        let mut text = String::new();
        let mut spans = Vec::new();
        for word in line.Words()? {
            let r = word.BoundingRect()?;
            if !text.is_empty() {
                text.push(' ');
            }
            let start = text.len();
            text.push_str(&word.Text()?.to_string());
            spans.push((start..text.len(), (r.X * back, r.Y * back, r.Width * back, r.Height * back)));
        }
        let Some(rect) = spans.iter().map(|(_, r)| *r).reduce(union) else { continue };
        lines.push(OcrLine { text, rect, spans });
    }
    Ok(lines)
}

pub fn union(a: Bounds, b: Bounds) -> Bounds {
    let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
    let (x1, y1) = ((a.0 + a.2).max(b.0 + b.2), (a.1 + a.3).max(b.1 + b.3));
    (x0, y0, x1 - x0, y1 - y0)
}

fn windows_ocr(img: &RgbaImage) -> windows::core::Result<(OcrResult, f32)> {
    let engine = OcrEngine::TryCreateFromUserProfileLanguages()?;
    let max = OcrEngine::MaxImageDimension()?;
    let longest = img.width().max(img.height());
    let scaled;
    let img = if longest > max {
        let scale = max as f64 / longest as f64;
        let w = ((img.width() as f64 * scale) as u32).max(1);
        let h = ((img.height() as f64 * scale) as u32).max(1);
        scaled = resize(img, w, h, FilterType::Triangle);
        &scaled
    } else {
        img
    };
    let mut bytes = img.as_raw().clone();
    for px in bytes.as_chunks_mut::<4>().0 {
        px.swap(0, 2);
    }
    let buffer = CryptographicBuffer::CreateFromByteArray(&bytes)?;
    let bitmap = SoftwareBitmap::Create(BitmapPixelFormat::Bgra8, img.width() as i32, img.height() as i32)?;
    bitmap.CopyFromBuffer(&buffer)?;
    let back = longest as f32 / img.width().max(img.height()) as f32;
    Ok((engine.RecognizeAsync(&bitmap)?.join()?, back))
}

fn recognize(img: &RgbaImage) -> windows::core::Result<String> {
    let (result, _) = windows_ocr(img)?;
    let mut lines = Vec::new();
    for line in result.Lines()? {
        let (mut x, mut top, mut bottom) = (f32::MAX, f32::MAX, f32::MIN);
        for word in line.Words()? {
            let r = word.BoundingRect()?;
            x = x.min(r.X);
            top = top.min(r.Y);
            bottom = bottom.max(r.Y + r.Height);
        }
        lines.push((x, top, bottom, line.Text()?.to_string()));
    }
    Ok(rows(lines))
}

pub fn rows(mut lines: Vec<(f32, f32, f32, String)>) -> String {
    lines.sort_by(|a, b| a.1.total_cmp(&b.1));
    let mut rows: Vec<Row> = Vec::new();
    for (x, top, bottom, text) in lines {
        let center = (top + bottom) / 2.0;
        match rows.last_mut() {
            Some(row) if (row.0..=row.1).contains(&center) => {
                row.0 = row.0.min(top);
                row.1 = row.1.max(bottom);
                row.2.push((x, text));
            }
            _ => rows.push((top, bottom, vec![(x, text)])),
        }
    }
    let rows: Vec<String> = rows
        .into_iter()
        .map(|(_, _, mut items)| {
            items.sort_by(|a, b| a.0.total_cmp(&b.0));
            items.into_iter().map(|(_, text)| text).collect::<Vec<_>>().join(" ")
        })
        .collect();
    rows.join("\n")
}
