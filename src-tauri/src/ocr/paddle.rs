use std::path::Path;

use image::RgbaImage;
use image::imageops::{FilterType, crop_imm, resize};
use ndarray::{Array4, ArrayView2, Axis};
use ort::environment::Environment;
use ort::session::Session;
use ort::value::TensorRef;

const DET_MIN: u32 = 64;
const DET_MAX: u32 = 4000;
const REC_HEIGHT: u32 = 48;
const REC_MIN: u32 = 320;
const REC_MAX: u32 = 3200;
const BATCH: usize = 8;
const MEAN: [f32; 3] = [0.485, 0.456, 0.406];
const STD: [f32; 3] = [0.229, 0.224, 0.225];

pub type Line = (f32, f32, f32, f32, String);

struct Rect {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

pub struct Engine {
    det: Session,
    rec: Session,
    dict: Vec<String>,
}

fn session(env: &Environment, path: &Path) -> Result<Session, String> {
    let threads = std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1));
    Session::builder(env)
        .map_err(|e| e.to_string())?
        .with_intra_threads(threads)
        .map_err(|e| e.to_string())?
        .with_intra_op_spinning(false)
        .map_err(|e| e.to_string())?
        .with_inter_op_spinning(false)
        .map_err(|e| e.to_string())?
        .commit_from_file(path)
        .map_err(|e| e.to_string())
}

impl Engine {
    pub fn load(det: &Path, rec: &Path, dict: &Path, runtime_path: &Path) -> Result<Self, String> {
        let env = crate::onnx::init(runtime_path)?;
        let mut dict: Vec<String> = std::fs::read_to_string(dict)
            .map_err(|e| e.to_string())?
            .lines()
            .map(str::to_owned)
            .collect();
        dict.push(" ".into());
        Ok(Self { det: session(env, det)?, rec: session(env, rec)?, dict })
    }

    pub fn extract(&mut self, img: &RgbaImage, params: (f32, f32, f32)) -> Result<Vec<Line>, String> {
        let boxes = self.detect(img, params)?;
        let mut order: Vec<usize> = (0..boxes.len()).collect();
        order.sort_by(|&a, &b| (boxes[a].w / boxes[a].h).total_cmp(&(boxes[b].w / boxes[b].h)));
        let mut lines = Vec::new();
        for chunk in order.chunks(BATCH) {
            let texts = self.recognize(img, chunk.iter().map(|&i| &boxes[i]))?;
            for (&i, text) in chunk.iter().zip(texts) {
                let b = &boxes[i];
                if !text.trim().is_empty() {
                    lines.push((b.x, b.y, b.w, b.h, text));
                }
            }
        }
        Ok(lines)
    }

    fn detect(&mut self, img: &RgbaImage, (thresh, box_thresh, unclip): (f32, f32, f32)) -> Result<Vec<Rect>, String> {
        let (src_w, src_h) = img.dimensions();
        let ratio = (DET_MIN as f32 / src_w.min(src_h) as f32)
            .max(1.0)
            .min(DET_MAX as f32 / src_w.max(src_h) as f32);
        let side = |n: u32| (((n as f32 * ratio) / 32.0).round() as usize * 32).max(32);
        let (w, h) = (side(src_w), side(src_h));
        let resized = resize(img, w as u32, h as u32, FilterType::Triangle);
        let mut input = Array4::<f32>::zeros((1, 3, h, w));
        for (x, y, px) in resized.enumerate_pixels() {
            for c in 0..3 {
                input[[0, c, y as usize, x as usize]] = (px[2 - c] as f32 / 255.0 - MEAN[c]) / STD[c];
            }
        }
        let outputs = self
            .det
            .run(ort::inputs![TensorRef::from_array_view(&input).map_err(|e| e.to_string())?])
            .map_err(|e| e.to_string())?;
        let (_, prob) = outputs[0].try_extract_tensor::<f32>().map_err(|e| e.to_string())?;
        let (sx, sy) = (src_w as f32 / w as f32, src_h as f32 / h as f32);
        let mut seen: Vec<bool> = prob.iter().map(|&p| p <= thresh).collect();
        let mut boxes = Vec::new();
        let mut stack = Vec::new();
        for start in 0..seen.len() {
            if seen[start] {
                continue;
            }
            seen[start] = true;
            stack.push(start);
            let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
            while let Some(i) = stack.pop() {
                let (x, y) = (i % w, i / w);
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
                let next = [
                    (x > 0).then(|| i - 1),
                    (x + 1 < w).then(|| i + 1),
                    (y > 0).then(|| i - w),
                    (y + 1 < h).then(|| i + w),
                ];
                for j in next.into_iter().flatten() {
                    if !seen[j] {
                        seen[j] = true;
                        stack.push(j);
                    }
                }
            }
            let (bw, bh) = ((x1 - x0 + 1) as f32, (y1 - y0 + 1) as f32);
            if bw.min(bh) < 3.0 {
                continue;
            }
            let score: f32 = (y0..=y1).map(|y| prob[y * w + x0..=y * w + x1].iter().sum::<f32>()).sum::<f32>() / (bw * bh);
            if score < box_thresh {
                continue;
            }
            let d = bw * bh * unclip / (2.0 * (bw + bh));
            if (bw + 2.0 * d).min(bh + 2.0 * d) < 5.0 {
                continue;
            }
            let left = ((x0 as f32 - d) * sx).clamp(0.0, src_w as f32);
            let top = ((y0 as f32 - d) * sy).clamp(0.0, src_h as f32);
            let right = ((x1 as f32 + 1.0 + d) * sx).clamp(0.0, src_w as f32);
            let bottom = ((y1 as f32 + 1.0 + d) * sy).clamp(0.0, src_h as f32);
            if right - left >= 1.0 && bottom - top >= 1.0 {
                boxes.push(Rect { x: left, y: top, w: right - left, h: bottom - top });
            }
        }
        Ok(boxes)
    }

    fn recognize<'a>(&mut self, img: &RgbaImage, boxes: impl Iterator<Item = &'a Rect>) -> Result<Vec<String>, String> {
        let crops: Vec<RgbaImage> = boxes
            .map(|b| {
                let crop = crop_imm(img, b.x as u32, b.y as u32, b.w.round().max(1.0) as u32, b.h.round().max(1.0) as u32).to_image();
                let width = ((REC_HEIGHT as f32 * crop.width() as f32 / crop.height() as f32).ceil() as u32).clamp(1, REC_MAX);
                resize(&crop, width, REC_HEIGHT, FilterType::Triangle)
            })
            .collect();
        let width = crops.iter().map(RgbaImage::width).max().unwrap_or(0).max(REC_MIN) as usize;
        let mut input = Array4::<f32>::zeros((crops.len(), 3, REC_HEIGHT as usize, width));
        for (n, crop) in crops.iter().enumerate() {
            for (x, y, px) in crop.enumerate_pixels() {
                for c in 0..3 {
                    input[[n, c, y as usize, x as usize]] = px[2 - c] as f32 / 127.5 - 1.0;
                }
            }
        }
        let outputs = self
            .rec
            .run(ort::inputs![TensorRef::from_array_view(&input).map_err(|e| e.to_string())?])
            .map_err(|e| e.to_string())?;
        let probs = outputs[0].try_extract_array::<f32>().map_err(|e| e.to_string())?;
        let probs = probs.into_dimensionality::<ndarray::Ix3>().map_err(|e| e.to_string())?;
        Ok(probs.axis_iter(Axis(0)).map(|steps| decode(&self.dict, steps)).collect())
    }
}

fn decode(dict: &[String], steps: ArrayView2<f32>) -> String {
    let mut text = String::new();
    let mut last = 0;
    for step in steps.axis_iter(Axis(0)) {
        let best = step.iter().enumerate().max_by(|a, b| a.1.total_cmp(b.1)).map_or(0, |(i, _)| i);
        if best != 0
            && best != last
            && let Some(token) = dict.get(best - 1)
        {
            text.push_str(token);
        }
        last = best;
    }
    text
}
