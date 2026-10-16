use image::RgbaImage;
use image::imageops::{FilterType, resize};
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Security::Cryptography::CryptographicBuffer;

type Row = (f32, f32, Vec<(f32, String)>);

pub fn available() -> bool {
    OcrEngine::TryCreateFromUserProfileLanguages().is_ok()
}

pub fn extract(img: &RgbaImage) -> Result<String, String> {
    recognize(img).map_err(|e| e.message())
}

fn recognize(img: &RgbaImage) -> windows::core::Result<String> {
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
    let result = engine.RecognizeAsync(&bitmap)?.join()?;
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
    Ok(rows.join("\n"))
}
