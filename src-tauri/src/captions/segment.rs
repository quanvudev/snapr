use std::collections::VecDeque;

const FACTOR: usize = 3;
const TAPS: usize = 48;
const CUTOFF: f32 = 7_000.0 / 48_000.0;
const PCM_SCALE: f32 = 32768.0;
const FRAME: usize = 160;
const PREROLL_FRAMES: usize = 30;
const PAUSE_FRAMES: usize = 120;
const MAX_FRAMES: usize = 1000;
const MIN_LEVEL: f32 = 0.008;
const FLOOR_RATIO: f32 = 2.0;
const FLOOR_ADAPT: f32 = 0.02;

pub struct Resampler {
    taps: Vec<f32>,
    history: Vec<f32>,
    phase: usize,
}

impl Default for Resampler {
    fn default() -> Self {
        let centre = (TAPS - 1) as f32 / 2.0;
        let raw: Vec<f32> = (0..TAPS)
            .map(|i| {
                let x = i as f32 - centre;
                let sinc = if x == 0.0 { 2.0 * CUTOFF } else { (2.0 * std::f32::consts::PI * CUTOFF * x).sin() / (std::f32::consts::PI * x) };
                let window = 0.5 - 0.5 * (2.0 * std::f32::consts::PI * i as f32 / (TAPS - 1) as f32).cos();
                sinc * window
            })
            .collect();
        let sum: f32 = raw.iter().sum();
        Resampler { taps: raw.iter().map(|t| t / sum).collect(), history: vec![0.0; TAPS], phase: 0 }
    }
}

impl Resampler {
    pub fn push(&mut self, stereo: &[i16], out: &mut Vec<f32>) {
        for [left, right] in stereo.as_chunks::<2>().0 {
            let mono = (*left as f32 + *right as f32) * 0.5 / PCM_SCALE;
            self.history.rotate_left(1);
            self.history[TAPS - 1] = mono;
            self.phase += 1;
            if self.phase == FACTOR {
                self.phase = 0;
                out.push(self.history.iter().zip(&self.taps).map(|(s, t)| s * t).sum());
            }
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Segment {
    Speech(Vec<f32>),
    End,
}

pub struct Segmenter {
    floor: f32,
    speaking: bool,
    silence: usize,
    length: usize,
    preroll: VecDeque<Vec<f32>>,
    frame: Vec<f32>,
}

impl Default for Segmenter {
    fn default() -> Self {
        Segmenter { floor: MIN_LEVEL / FLOOR_RATIO, speaking: false, silence: 0, length: 0, preroll: VecDeque::new(), frame: Vec::with_capacity(FRAME) }
    }
}

fn rms(frame: &[f32]) -> f32 {
    (frame.iter().map(|s| s * s).sum::<f32>() / frame.len().max(1) as f32).sqrt()
}

fn speech(out: &mut Vec<Segment>, samples: &[f32]) {
    match out.last_mut() {
        Some(Segment::Speech(audio)) => audio.extend_from_slice(samples),
        _ => out.push(Segment::Speech(samples.to_vec())),
    }
}

impl Segmenter {
    fn threshold(&self) -> f32 {
        (self.floor * FLOOR_RATIO).max(MIN_LEVEL)
    }

    pub fn push(&mut self, samples: &[f32]) -> Vec<Segment> {
        let mut out = Vec::new();
        for &sample in samples {
            self.frame.push(sample);
            if self.frame.len() == FRAME {
                let frame = std::mem::replace(&mut self.frame, Vec::with_capacity(FRAME));
                self.step(frame, &mut out);
            }
        }
        out
    }

    fn step(&mut self, frame: Vec<f32>, out: &mut Vec<Segment>) {
        let level = rms(&frame);
        let loud = level > self.threshold();
        if !self.speaking {
            if !loud {
                self.floor += (level - self.floor) * FLOOR_ADAPT;
                self.preroll.push_back(frame);
                if self.preroll.len() > PREROLL_FRAMES {
                    self.preroll.pop_front();
                }
                return;
            }
            self.speaking = true;
            self.silence = 0;
            self.length = 0;
            for earlier in self.preroll.drain(..) {
                speech(out, &earlier);
            }
        }
        speech(out, &frame);
        self.length += 1;
        self.silence = if loud { 0 } else { self.silence + 1 };
        if self.silence >= PAUSE_FRAMES || self.length >= MAX_FRAMES {
            self.speaking = false;
            out.push(Segment::End);
        }
    }
}

impl Segmenter {
    pub fn cut(&mut self) -> Option<Segment> {
        std::mem::take(&mut self.speaking).then_some(Segment::End)
    }
}

#[cfg(test)]
impl Segmenter {
    fn speaking(&self) -> bool {
        self.speaking
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(seconds: f32, amplitude: f32) -> Vec<f32> {
        (0..(seconds * 16_000.0) as usize).map(|i| amplitude * (i as f32 * 2.0 * std::f32::consts::PI * 440.0 / 16_000.0).sin()).collect()
    }

    fn speech_samples(segments: &[Segment]) -> usize {
        segments.iter().map(|s| if let Segment::Speech(a) = s { a.len() } else { 0 }).sum()
    }

    #[test]
    fn resamples_a_tone_level() {
        let stereo: Vec<i16> =
            (0..48_000).flat_map(|i| [((i as f32 * 2.0 * std::f32::consts::PI * 1000.0 / 48_000.0).sin() * 16384.0) as i16; 2]).collect();
        let mut out = Vec::new();
        Resampler::default().push(&stereo, &mut out);
        assert_eq!(out.len(), 16_000);
        let level = rms(&out[1000..]);
        assert!((level - 0.5 / 2f32.sqrt()).abs() < 0.02, "{level}");
    }

    #[test]
    fn removes_content_above_eight_khz() {
        let stereo: Vec<i16> =
            (0..48_000).flat_map(|i| [((i as f32 * 2.0 * std::f32::consts::PI * 15_000.0 / 48_000.0).sin() * 16384.0) as i16; 2]).collect();
        let mut out = Vec::new();
        Resampler::default().push(&stereo, &mut out);
        assert!(rms(&out[1000..]) < 0.02);
    }

    #[test]
    fn skips_silence() {
        let mut segmenter = Segmenter::default();
        assert!(segmenter.push(&vec![0.0; 32_000]).is_empty());
        assert!(!segmenter.speaking());
    }

    #[test]
    fn starts_with_preroll_and_ends_after_a_pause() {
        let mut segmenter = Segmenter::default();
        let mut audio = vec![0.0; 16_000];
        audio.extend(tone(1.0, 0.3));
        audio.extend(vec![0.0; 32_000]);
        let out = segmenter.push(&audio);
        assert_eq!(out.last(), Some(&Segment::End));
        assert_eq!(out.iter().filter(|s| **s == Segment::End).count(), 1);
        let samples = speech_samples(&out);
        assert_eq!(samples, (PREROLL_FRAMES + 100 + PAUSE_FRAMES) * FRAME);
        assert!(!segmenter.speaking());
    }

    #[test]
    fn caps_a_long_line() {
        let mut segmenter = Segmenter::default();
        let out = segmenter.push(&tone(25.0, 0.3));
        assert_eq!(out.iter().filter(|s| **s == Segment::End).count(), 2);
        assert!(segmenter.speaking());
        assert_eq!(segmenter.cut(), Some(Segment::End));
    }

    #[test]
    fn floor_follows_steady_noise() {
        let mut segmenter = Segmenter::default();
        let noise: Vec<f32> = (0..64_000).map(|i| if i % 2 == 0 { 0.006 } else { -0.006 }).collect();
        assert!(segmenter.push(&noise).is_empty());
        assert!(segmenter.push(&tone(0.5, 0.3)).iter().any(|s| matches!(s, Segment::Speech(_))));
    }
}
