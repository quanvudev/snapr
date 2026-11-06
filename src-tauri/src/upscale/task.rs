use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use image::imageops::{FilterType, resize};
use image::{GrayImage, ImageFormat, Rgba, RgbaImage};
use ndarray::Array4;
use ort::environment::Environment;
use ort::value::TensorRef;
use tauri::{AppHandle, Manager};

use super::{MODEL_FILE, acquire, model_dir, ready};
use crate::onnx::{self, Workload};
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::transcript::MODEL_MISSING;
use crate::{State, gallery, library};

pub const TOO_LARGE: &str = "TOO_LARGE";
const KIND: &str = "upscale";
const PHOTO_EXT: &str = "png";
const MAX_SIDE: u32 = 8192;
const MODEL_SCALE: u32 = 4;
const TILE: u32 = 256;
const PAD: u32 = 16;
const INPUT: &str = "input";
const OUTPUT: &str = "output";

pub struct UpscaleTask {
    path: PathBuf,
    scale: u32,
    out_dir: PathBuf,
    model: PathBuf,
    runtime: onnx::Target,
}

impl UpscaleTask {
    pub fn validated(app: &AppHandle, path: &str, scale: u32) -> Result<UpscaleTask, String> {
        let path = gallery::guard(app, path).ok_or("path outside the output folder")?;
        if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(PHOTO_EXT)) {
            return Err("not a PNG image".into());
        }
        if scale != 2 && scale != 4 {
            return Err("scale must be 2 or 4".into());
        }
        if !ready(app) {
            return Err(MODEL_MISSING.into());
        }
        let (w, h) = image::image_dimensions(&path).map_err(|e| e.to_string())?;
        if w.max(h).saturating_mul(scale) > MAX_SIDE {
            return Err(TOO_LARGE.into());
        }
        let out_dir = library::images(&app.state::<State>().cfg.read().unwrap());
        Ok(UpscaleTask { path, scale, out_dir, model: model_dir(app).join(MODEL_FILE), runtime: onnx::target(app) })
    }

    fn stem(&self) -> String {
        let stem = self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        format!("{stem}_x{}", self.scale)
    }
}

impl Task for UpscaleTask {
    fn kind(&self) -> &'static str {
        KIND
    }

    fn name(&self) -> String {
        self.path.file_name().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn source(&self) -> Option<&Path> {
        Some(&self.path)
    }

    fn run(&self, progress: &Progress, cancel: &AtomicBool) -> Result<Option<PathBuf>, String> {
        if !self.model.is_file() {
            return Err(MODEL_MISSING.into());
        }
        let _in_use = acquire();
        let env = onnx::init(&self.runtime)?;
        let img = image::open(&self.path).map_err(|e| e.to_string())?.to_rgba8();
        progress.set(10, 100);
        let report = |done: usize, total: usize| progress.set(10 + (80 * done / total.max(1)) as u64, 100);
        let out = onnx::retry_on_cpu(Workload::Upscale, || upscale(env, &self.model, &img, cancel, &report))?;
        let out = finish(out, &img, self.scale);
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        std::fs::create_dir_all(&self.out_dir).map_err(|e| e.to_string())?;
        let written = queue::write_part(&self.out_dir, &self.stem(), PHOTO_EXT, |part| {
            out.save_with_format(part, ImageFormat::Png).map_err(|e| e.to_string())
        });
        progress.set(100, 100);
        written
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

#[derive(Clone, Copy, Debug)]
struct Tile {
    core: Rect,
    context: Rect,
}

fn plan(width: u32, height: u32) -> Vec<Tile> {
    let mut tiles = Vec::new();
    for y in (0..height).step_by(TILE as usize) {
        for x in (0..width).step_by(TILE as usize) {
            let core = Rect { x, y, w: TILE.min(width - x), h: TILE.min(height - y) };
            let (cx, cy) = (x.saturating_sub(PAD), y.saturating_sub(PAD));
            let (right, bottom) = ((x + core.w + PAD).min(width), (y + core.h + PAD).min(height));
            tiles.push(Tile { core, context: Rect { x: cx, y: cy, w: right - cx, h: bottom - cy } });
        }
    }
    tiles
}

pub fn upscale(
    env: &Environment,
    model: &Path,
    img: &RgbaImage,
    cancel: &AtomicBool,
    report: &dyn Fn(usize, usize),
) -> Result<RgbaImage, String> {
    let tiles = plan(img.width(), img.height());
    let mut session = onnx::session(env, model, Workload::Upscale)?;
    let mut out = RgbaImage::new(img.width() * MODEL_SCALE, img.height() * MODEL_SCALE);
    for (i, tile) in tiles.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            return Err(CANCELLED.into());
        }
        let input = tensor(img, tile.context);
        let outputs = session
            .run(ort::inputs![INPUT => TensorRef::from_array_view(&input).map_err(|e| e.to_string())?])
            .map_err(|e| e.to_string())?;
        let (_, data) = outputs[OUTPUT].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        let expected = 3 * (tile.context.w * MODEL_SCALE) as usize * (tile.context.h * MODEL_SCALE) as usize;
        if data.len() != expected {
            return Err("unexpected output size".into());
        }
        paste(&mut out, data, tile);
        report(i + 1, tiles.len());
    }
    Ok(out)
}

fn tensor(img: &RgbaImage, region: Rect) -> Array4<f32> {
    let mut input = Array4::<f32>::zeros((1, 3, region.h as usize, region.w as usize));
    for y in 0..region.h {
        for x in 0..region.w {
            let px = img.get_pixel(region.x + x, region.y + y);
            for c in 0..3 {
                input[[0, c, y as usize, x as usize]] = px[c] as f32 / 255.0;
            }
        }
    }
    input
}

fn paste(out: &mut RgbaImage, data: &[f32], tile: &Tile) {
    let (core, context) = (tile.core, tile.context);
    let (w, h) = ((context.w * MODEL_SCALE) as usize, (context.h * MODEL_SCALE) as usize);
    let plane = w * h;
    let (ox, oy) = (((core.x - context.x) * MODEL_SCALE) as usize, ((core.y - context.y) * MODEL_SCALE) as usize);
    let byte = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    for y in 0..(core.h * MODEL_SCALE) as usize {
        for x in 0..(core.w * MODEL_SCALE) as usize {
            let at = (oy + y) * w + ox + x;
            let px = Rgba([byte(data[at]), byte(data[plane + at]), byte(data[2 * plane + at]), 255]);
            out.put_pixel(core.x * MODEL_SCALE + x as u32, core.y * MODEL_SCALE + y as u32, px);
        }
    }
}

fn finish(out: RgbaImage, source: &RgbaImage, scale: u32) -> RgbaImage {
    let (w, h) = (source.width() * scale, source.height() * scale);
    let mut out = if scale == MODEL_SCALE { out } else { resize(&out, w, h, FilterType::Lanczos3) };
    if source.pixels().all(|p| p[3] == 255) {
        return out;
    }
    let alpha = GrayImage::from_fn(source.width(), source.height(), |x, y| image::Luma([source.get_pixel(x, y)[3]]));
    let alpha = resize(&alpha, w, h, FilterType::Lanczos3);
    for (px, a) in out.pixels_mut().zip(alpha.pixels()) {
        px[3] = a[0];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plans_tiles() {
        for (w, h) in [(1, 1), (256, 256), (257, 100), (600, 300), (100, 700)] {
            let tiles = plan(w, h);
            let mut hits = vec![0u8; (w * h) as usize];
            for Tile { core, context } in &tiles {
                for y in core.y..core.y + core.h {
                    for x in core.x..core.x + core.w {
                        hits[(y * w + x) as usize] += 1;
                    }
                }
                assert!(context.x <= core.x && context.y <= core.y);
                assert!(context.x + context.w >= core.x + core.w && context.y + context.h >= core.y + core.h);
                assert!(context.x + context.w <= w && context.y + context.h <= h);
                assert_eq!(core.x - context.x, core.x.min(PAD));
                assert_eq!(core.y - context.y, core.y.min(PAD));
                assert_eq!(context.x + context.w - core.x - core.w, (w - core.x - core.w).min(PAD));
                assert_eq!(context.y + context.h - core.y - core.h, (h - core.y - core.h).min(PAD));
                assert!(core.w <= TILE && core.h <= TILE);
            }
            assert!(hits.iter().all(|&n| n == 1), "{w}x{h}");
        }
    }

    #[test]
    fn builds_rgb_tensor() {
        let img = RgbaImage::from_fn(4, 3, |x, y| Rgba([(x * 50) as u8, (y * 100) as u8, 255, 7]));
        let input = tensor(&img, Rect { x: 1, y: 1, w: 3, h: 2 });
        assert_eq!(input.shape(), &[1, 3, 2, 3]);
        let close = |a: f32, b: f32| (a - b).abs() < 1e-6;
        assert!(close(input[[0, 0, 0, 0]], 50.0 / 255.0));
        assert!(close(input[[0, 1, 0, 0]], 100.0 / 255.0));
        assert!(close(input[[0, 2, 0, 0]], 1.0));
        assert!(close(input[[0, 0, 1, 2]], 150.0 / 255.0));
        assert!(close(input[[0, 1, 1, 2]], 200.0 / 255.0));
    }

    #[test]
    fn pastes_only_the_core() {
        let tile = Tile { core: Rect { x: 2, y: 1, w: 2, h: 1 }, context: Rect { x: 1, y: 0, w: 4, h: 3 } };
        let (w, h) = ((tile.context.w * MODEL_SCALE) as usize, (tile.context.h * MODEL_SCALE) as usize);
        let plane = w * h;
        let mut data = vec![-1.0f32; 3 * plane];
        for y in 4..8 {
            for x in 4..12 {
                data[y * w + x] = 2.0;
                data[plane + y * w + x] = 0.5;
                data[2 * plane + y * w + x] = 0.0;
            }
        }
        let mut out = RgbaImage::new(24, 12);
        paste(&mut out, &data, &tile);
        for (x, y, px) in out.enumerate_pixels() {
            if (8..16).contains(&x) && (4..8).contains(&y) {
                assert_eq!(px.0, [255, 128, 0, 255]);
            } else {
                assert_eq!(px.0, [0, 0, 0, 0]);
            }
        }
    }

    #[test]
    fn finishes_scale_and_alpha() {
        let opaque = RgbaImage::from_pixel(10, 6, Rgba([1, 2, 3, 255]));
        let clear = RgbaImage::from_pixel(10, 6, Rgba([1, 2, 3, 0]));
        let model_out = RgbaImage::from_pixel(40, 24, Rgba([9, 9, 9, 255]));
        let x4 = finish(model_out.clone(), &opaque, 4);
        assert_eq!(x4.dimensions(), (40, 24));
        assert!(x4.pixels().all(|p| p[3] == 255));
        let x2 = finish(model_out.clone(), &opaque, 2);
        assert_eq!(x2.dimensions(), (20, 12));
        assert!(x2.pixels().all(|p| p[3] == 255));
        let clear2 = finish(model_out.clone(), &clear, 2);
        assert_eq!(clear2.dimensions(), (20, 12));
        assert!(clear2.pixels().all(|p| p[3] == 0));
        assert!(finish(model_out, &clear, 4).pixels().all(|p| p[3] == 0));
    }

    #[test]
    #[ignore]
    fn renders_upscale() {
        let (Ok(model), Ok(source), Ok(out)) = (
            std::env::var("SNAPR_UPSCALE_MODEL"),
            std::env::var("SNAPR_UPSCALE_SOURCE"),
            std::env::var("SNAPR_UPSCALE_OUT"),
        ) else {
            return;
        };
        let scale = std::env::var("SNAPR_UPSCALE_SCALE").ok().and_then(|s| s.parse().ok()).unwrap_or(MODEL_SCALE);
        let runtime = match std::env::var("SNAPR_ORT_EP").as_deref() {
            Ok("directml") => onnx::Runtime::DirectMl,
            Ok("cuda") => onnx::Runtime::Cuda,
            _ => onnx::Runtime::Cpu,
        };
        let dll = std::env::var("SNAPR_ORT_DLL").map(PathBuf::from).unwrap_or_default();
        let device = crate::gpu::best().filter(|_| runtime == onnx::Runtime::DirectMl).map_or(0, |a| a.index as i32);
        let env = onnx::init(&onnx::Target { runtime, dll, device }).unwrap();
        eprintln!("placed on {:?}", onnx::gpu_for(Workload::Upscale));
        let img = image::open(&source).unwrap().to_rgba8();
        let cancel = AtomicBool::new(false);
        eprintln!("{} tiles", plan(img.width(), img.height()).len());
        for run in 0..2 {
            let started = std::time::Instant::now();
            let result = upscale(env, Path::new(&model), &img, &cancel, &|_, _| {}).unwrap();
            let result = finish(result, &img, scale);
            eprintln!(
                "run {run}: {}x{} -> {}x{} in {:?}",
                img.width(),
                img.height(),
                result.width(),
                result.height(),
                started.elapsed()
            );
            result.save(&out).unwrap();
        }
        eprintln!("gpu error: {:?}", onnx::gpu_error());
    }
}
