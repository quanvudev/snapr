use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use image::imageops::{FilterType, resize};
use image::{GrayImage, ImageFormat, RgbaImage};
use ndarray::Array4;
use ort::environment::Environment;
use ort::value::TensorRef;
use tauri::{AppHandle, Manager};

use super::{MODEL_FILE, acquire, model_dir, ready};
use crate::onnx::{self, Workload};
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::transcript::MODEL_MISSING;
use crate::{State, gallery, library};

const KIND: &str = "cutout";
const PHOTO_EXT: &str = "png";
const SIDE: usize = 1024;
const INPUT: &str = "input.1";
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

pub struct CutoutTask {
    path: PathBuf,
    out_dir: PathBuf,
    model: PathBuf,
    runtime: onnx::Target,
}

impl CutoutTask {
    pub fn validated(app: &AppHandle, path: &str) -> Result<CutoutTask, String> {
        let path = gallery::guard(app, path).ok_or("path outside the output folder")?;
        if !path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(PHOTO_EXT)) {
            return Err("not a PNG image".into());
        }
        if !ready(app) {
            return Err(MODEL_MISSING.into());
        }
        let out_dir = library::images(&app.state::<State>().cfg.read().unwrap());
        Ok(CutoutTask { path, out_dir, model: model_dir(app).join(MODEL_FILE), runtime: onnx::target(app) })
    }

    fn stem(&self) -> String {
        let stem = self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default();
        format!("{stem}_{KIND}")
    }
}

impl Task for CutoutTask {
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
        let out = onnx::retry_on_cpu(Workload::Cutout, || cutout(env, &self.model, &img, cancel))?;
        progress.set(80, 100);
        std::fs::create_dir_all(&self.out_dir).map_err(|e| e.to_string())?;
        let written = queue::write_part(&self.out_dir, &self.stem(), PHOTO_EXT, |part| {
            out.save_with_format(part, ImageFormat::Png).map_err(|e| e.to_string())
        });
        progress.set(100, 100);
        written
    }
}

pub fn cutout(env: &Environment, model: &Path, img: &RgbaImage, cancel: &AtomicBool) -> Result<RgbaImage, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err(CANCELLED.into());
    }
    let input = tensor(img);
    let mut session = onnx::session(env, model, Workload::Cutout)?;
    let outputs = session
        .run(ort::inputs![INPUT => TensorRef::from_array_view(&input).map_err(|e| e.to_string())?])
        .map_err(|e| e.to_string())?;
    let (_, alpha) = outputs[0].try_extract_tensor::<half::f16>().map_err(|e| e.to_string())?;
    let alpha: Vec<f32> = alpha.iter().map(|v| v.to_f32()).collect();
    if alpha.len() != SIDE * SIDE {
        return Err("unexpected mask size".into());
    }
    let out = with_mask(img, &mask(&alpha));
    if cancel.load(Ordering::Relaxed) {
        return Err(CANCELLED.into());
    }
    Ok(out)
}

fn tensor(img: &RgbaImage) -> Array4<f32> {
    let resized = resize(img, SIDE as u32, SIDE as u32, FilterType::Triangle);
    let mut input = Array4::<f32>::zeros((1, 3, SIDE, SIDE));
    for (x, y, px) in resized.enumerate_pixels() {
        for c in 0..3 {
            input[[0, c, y as usize, x as usize]] = (px[c] as f32 / 255.0 - MEAN[c]) / STD[c];
        }
    }
    input
}

fn mask(alpha: &[f32]) -> GrayImage {
    let (min, max) = alpha.iter().fold((f32::MAX, f32::MIN), |(lo, hi), &v| (lo.min(v), hi.max(v)));
    let range = (max - min).max(f32::EPSILON);
    let pixels = alpha.iter().map(|&v| ((v - min) / range * 255.0).round() as u8).collect();
    GrayImage::from_raw(SIDE as u32, SIDE as u32, pixels).unwrap_or_default()
}

fn with_mask(img: &RgbaImage, mask: &GrayImage) -> RgbaImage {
    let scaled = resize(mask, img.width(), img.height(), FilterType::Triangle);
    let mut out = img.clone();
    for (px, m) in out.pixels_mut().zip(scaled.pixels()) {
        px[3] = ((px[3] as u32 * m[0] as u32 + 127) / 255) as u8;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Luma, Rgba};

    #[test]
    fn builds_normalised_nchw() {
        let img = RgbaImage::from_fn(4, 2, |x, _| if x < 2 { Rgba([255, 0, 0, 255]) } else { Rgba([0, 0, 255, 255]) });
        let input = tensor(&img);
        assert_eq!(input.shape(), &[1, 3, SIDE, SIDE]);
        let close = |a: f32, b: f32| (a - b).abs() < 1e-4;
        assert!(close(input[[0, 0, 0, 0]], (1.0 - MEAN[0]) / STD[0]));
        assert!(close(input[[0, 1, 0, 0]], -MEAN[1] / STD[1]));
        assert!(close(input[[0, 2, 0, 0]], -MEAN[2] / STD[2]));
        assert!(close(input[[0, 0, SIDE - 1, SIDE - 1]], -MEAN[0] / STD[0]));
        assert!(close(input[[0, 2, SIDE - 1, SIDE - 1]], (1.0 - MEAN[2]) / STD[2]));
    }

    #[test]
    fn rescales_alpha() {
        let mut alpha = vec![0.1f32; SIDE * SIDE];
        alpha[0] = 0.9;
        alpha[1] = 0.5;
        let gray = mask(&alpha);
        assert_eq!(gray.get_pixel(0, 0)[0], 255);
        assert_eq!(gray.get_pixel(1, 0)[0], 128);
        assert_eq!(gray.get_pixel(2, 0)[0], 0);
    }

    #[test]
    fn applies_mask_to_alpha() {
        let img = RgbaImage::from_fn(8, 4, |x, _| Rgba([10, 20, 30, if x == 0 { 128 } else { 255 }]));
        let gray = GrayImage::from_fn(2, 2, |x, _| Luma([if x == 0 { 255 } else { 0 }]));
        let out = with_mask(&img, &gray);
        assert_eq!(out.dimensions(), (8, 4));
        assert_eq!(out.get_pixel(0, 0).0, [10, 20, 30, 128]);
        assert_eq!(out.get_pixel(1, 0)[3], 255);
        assert_eq!(out.get_pixel(7, 0)[3], 0);
        assert_eq!(&out.get_pixel(7, 0).0[..3], &[10, 20, 30]);
    }

    #[test]
    #[ignore]
    fn renders_cutout() {
        let (Ok(model), Ok(source), Ok(out)) = (
            std::env::var("SNAPR_CUTOUT_MODEL"),
            std::env::var("SNAPR_CUTOUT_SOURCE"),
            std::env::var("SNAPR_CUTOUT_OUT"),
        ) else {
            return;
        };
        let runtime = match std::env::var("SNAPR_ORT_EP").as_deref() {
            Ok("directml") => onnx::Runtime::DirectMl,
            Ok("cuda") => onnx::Runtime::Cuda,
            _ => onnx::Runtime::Cpu,
        };
        let dll = std::env::var("SNAPR_ORT_DLL").map(PathBuf::from).unwrap_or_default();
        let device = crate::gpu::best().filter(|_| runtime == onnx::Runtime::DirectMl).map_or(0, |a| a.index as i32);
        let env = onnx::init(&onnx::Target { runtime, dll, device }).unwrap();
        let started = std::time::Instant::now();
        let session = onnx::session(env, Path::new(&model), Workload::Cutout).unwrap();
        eprintln!("session built in {:?}", started.elapsed());
        for input in session.inputs() {
            eprintln!("input {} {:?}", input.name(), input.dtype());
        }
        for output in session.outputs() {
            eprintln!("output {} {:?}", output.name(), output.dtype());
        }
        drop(session);
        eprintln!("placed on {:?}", onnx::gpu_for(Workload::Cutout));
        let img = image::open(&source).unwrap().to_rgba8();
        let cancel = AtomicBool::new(false);
        for run in 0..2 {
            let started = std::time::Instant::now();
            let result = cutout(env, Path::new(&model), &img, &cancel).unwrap();
            eprintln!("run {run}: {}x{} in {:?}", img.width(), img.height(), started.elapsed());
            result.save(&out).unwrap();
        }
        eprintln!("gpu error: {:?}", onnx::gpu_error());
        if let Ok(other) = std::env::var("SNAPR_CUTOUT_COMPARE") {
            let (a, b) = (image::open(&out).unwrap().to_rgba8(), image::open(&other).unwrap().to_rgba8());
            let diffs: Vec<u8> = a.pixels().zip(b.pixels()).map(|(p, q)| p[3].abs_diff(q[3])).collect();
            let mean = diffs.iter().map(|&d| d as f64).sum::<f64>() / diffs.len() as f64;
            let over = diffs.iter().filter(|&&d| d > 32).count() as f64 / diffs.len() as f64;
            let max = diffs.iter().max();
            eprintln!("alpha vs {other}: mean diff {mean:.3}, max {max:?}, over 32: {:.4}%", over * 100.0);
        }
    }
}
