use std::f64::consts::PI;

pub const SAMPLE_RATE: usize = 16_000;
pub const N_FFT: usize = 400;
pub const HOP_LENGTH: usize = 160;
pub const N_FRAMES: usize = 3000;
pub const N_SAMPLES: usize = N_FRAMES * HOP_LENGTH;

const N_BINS: usize = N_FFT / 2 + 1;
const PAD: usize = N_FFT / 2;
const LOG_FLOOR: f32 = 1e-10;
const DYNAMIC_RANGE: f32 = 8.0;
const LOG_OFFSET: f32 = 4.0;
const LOG_SCALE: f32 = 4.0;
const SLANEY_LINEAR_HZ: f64 = 200.0 / 3.0;
const SLANEY_LOG_HZ: f64 = 1000.0;
const SLANEY_LOG_STEP_BASE: f64 = 6.4;
const SLANEY_LOG_STEP_DIV: f64 = 27.0;

pub struct Mel {
    n_mels: usize,
    cos: Vec<f32>,
    sin: Vec<f32>,
    filters: Vec<f32>,
}

fn hz_to_mel(hz: f64) -> f64 {
    let log_mel = SLANEY_LOG_HZ / SLANEY_LINEAR_HZ;
    let step = SLANEY_LOG_STEP_BASE.ln() / SLANEY_LOG_STEP_DIV;
    if hz >= SLANEY_LOG_HZ { log_mel + (hz / SLANEY_LOG_HZ).ln() / step } else { hz / SLANEY_LINEAR_HZ }
}

fn mel_to_hz(mel: f64) -> f64 {
    let log_mel = SLANEY_LOG_HZ / SLANEY_LINEAR_HZ;
    let step = SLANEY_LOG_STEP_BASE.ln() / SLANEY_LOG_STEP_DIV;
    if mel >= log_mel { SLANEY_LOG_HZ * (step * (mel - log_mel)).exp() } else { mel * SLANEY_LINEAR_HZ }
}

fn filters(n_mels: usize) -> Vec<f32> {
    let top = hz_to_mel(SAMPLE_RATE as f64 / 2.0);
    let edges: Vec<f64> = (0..n_mels + 2).map(|i| mel_to_hz(top * i as f64 / (n_mels + 1) as f64)).collect();
    let mut weights = vec![0.0; n_mels * N_BINS];
    for m in 0..n_mels {
        let norm = 2.0 / (edges[m + 2] - edges[m]);
        for k in 0..N_BINS {
            let freq = k as f64 * SAMPLE_RATE as f64 / N_FFT as f64;
            let lower = (freq - edges[m]) / (edges[m + 1] - edges[m]);
            let upper = (edges[m + 2] - freq) / (edges[m + 2] - edges[m + 1]);
            weights[m * N_BINS + k] = (lower.min(upper).max(0.0) * norm) as f32;
        }
    }
    weights
}

impl Mel {
    pub fn new(n_mels: usize) -> Self {
        let mut cos = vec![0.0; N_BINS * N_FFT];
        let mut sin = vec![0.0; N_BINS * N_FFT];
        for i in 0..N_FFT {
            let window = 0.5 - 0.5 * (2.0 * PI * i as f64 / N_FFT as f64).cos();
            for k in 0..N_BINS {
                let angle = 2.0 * PI * ((k * i) % N_FFT) as f64 / N_FFT as f64;
                cos[k * N_FFT + i] = (window * angle.cos()) as f32;
                sin[k * N_FFT + i] = (window * angle.sin()) as f32;
            }
        }
        Self { n_mels, cos, sin, filters: filters(n_mels) }
    }

    pub fn features(&self, samples: &[f32]) -> Vec<f32> {
        let len = samples.len().min(N_SAMPLES);
        let mut padded = vec![0.0f32; N_SAMPLES + 2 * PAD];
        padded[PAD..PAD + len].copy_from_slice(&samples[..len]);
        for i in 0..PAD {
            padded[PAD - 1 - i] = padded[PAD + 1 + i];
            padded[PAD + N_SAMPLES + i] = padded[PAD + N_SAMPLES - 2 - i];
        }
        let mut out = vec![0.0f32; self.n_mels * N_FRAMES];
        let mut power = [0.0f32; N_BINS];
        for t in 0..N_FRAMES {
            let frame = &padded[t * HOP_LENGTH..t * HOP_LENGTH + N_FFT];
            for (k, p) in power.iter_mut().enumerate() {
                let row = k * N_FFT..(k + 1) * N_FFT;
                let (mut re, mut im) = (0.0f32, 0.0f32);
                for ((x, c), s) in frame.iter().zip(&self.cos[row.clone()]).zip(&self.sin[row]) {
                    re += x * c;
                    im += x * s;
                }
                *p = re * re + im * im;
            }
            for m in 0..self.n_mels {
                let weights = &self.filters[m * N_BINS..(m + 1) * N_BINS];
                let energy: f32 = weights.iter().zip(&power).map(|(w, p)| w * p).sum();
                out[m * N_FRAMES + t] = energy.max(LOG_FLOOR).log10();
            }
        }
        let floor = out.iter().copied().fold(f32::NEG_INFINITY, f32::max) - DYNAMIC_RANGE;
        for v in &mut out {
            *v = (v.max(floor) + LOG_OFFSET) / LOG_SCALE;
        }
        out
    }
}
