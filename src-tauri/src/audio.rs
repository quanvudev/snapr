use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{ErrorKind, FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use nnnoiseless::DenoiseState;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

pub const RATE: u32 = 48_000;
pub const FRAMES: usize = 480;
const TICK: Duration = Duration::from_millis(10);
const PRIME: usize = FRAMES * 4;
const LIMIT: usize = PRIME + FRAMES * 2 * 2;
const LEVEL_TICKS: u32 = 7;
const PCM_SCALE: f32 = i16::MAX as f32;
const _: () = assert!(FRAMES == DenoiseState::FRAME_SIZE);

pub struct Device {
    pub id: String,
    pub name: String,
    pub default: bool,
}

pub type Chunk = Vec<i16>;

fn name(device: &cpal::Device) -> Option<String> {
    device.description().ok().map(|d| d.name().to_string())
}

fn list(default: Option<cpal::Device>, devices: Option<impl Iterator<Item = cpal::Device>>) -> Vec<Device> {
    let default = default.and_then(|d| name(&d));
    let Some(devices) = devices else { return Vec::new() };
    devices
        .filter_map(|d| name(&d))
        .map(|name| Device { default: default.as_deref() == Some(name.as_str()), id: name.clone(), name })
        .collect()
}

pub fn inputs() -> Vec<Device> {
    let host = cpal::default_host();
    list(host.default_input_device(), host.input_devices().ok())
}

pub fn outputs() -> Vec<Device> {
    let host = cpal::default_host();
    list(host.default_output_device(), host.output_devices().ok())
}

fn input(host: &cpal::Host, id: &str) -> Option<cpal::Device> {
    if id != "default"
        && let Some(device) = host.input_devices().ok().and_then(|mut all| all.find(|d| name(d).as_deref() == Some(id)))
    {
        return Some(device);
    }
    host.default_input_device()
}

fn output(host: &cpal::Host, id: &str) -> Option<cpal::Device> {
    if id != "default"
        && let Some(device) = host.output_devices().ok().and_then(|mut all| all.find(|d| name(d).as_deref() == Some(id)))
    {
        return Some(device);
    }
    host.default_output_device()
}

#[derive(Default)]
struct Buffer {
    samples: VecDeque<f32>,
    primed: bool,
}

#[derive(Default)]
struct Source {
    buffer: Mutex<Buffer>,
    lost: AtomicBool,
}

impl Source {
    fn push(&self, samples: &[f32]) {
        let Ok(mut buffer) = self.buffer.lock() else { return };
        buffer.samples.extend(samples);
        let excess = buffer.samples.len().saturating_sub(LIMIT);
        buffer.samples.drain(..excess);
    }

    fn take(&self, out: &mut [f32]) {
        out.fill(0.0);
        let Ok(mut buffer) = self.buffer.lock() else { return };
        if !buffer.primed && buffer.samples.len() >= PRIME {
            buffer.primed = true;
        }
        if !buffer.primed {
            return;
        }
        let n = buffer.samples.len().min(out.len());
        for (o, s) in out.iter_mut().zip(buffer.samples.drain(..n)) {
            *o = s;
        }
    }
}

struct Resampler {
    step: f64,
    pos: f64,
    prev: [f32; 2],
}

impl Resampler {
    fn new(rate: u32) -> Self {
        Self { step: rate as f64 / RATE as f64, pos: 0.0, prev: [0.0; 2] }
    }

    fn push(&mut self, frame: [f32; 2], out: &mut Vec<f32>) {
        while self.pos <= 1.0 {
            let t = self.pos as f32;
            out.push(self.prev[0] + (frame[0] - self.prev[0]) * t);
            out.push(self.prev[1] + (frame[1] - self.prev[1]) * t);
            self.pos += self.step;
        }
        self.pos -= 1.0;
        self.prev = frame;
    }
}

fn stereo<T: SizedSample>(frame: &[T]) -> [f32; 2]
where
    f32: FromSample<T>,
{
    match frame {
        [] => [0.0; 2],
        [mono] => {
            let s = mono.to_sample::<f32>();
            [s, s]
        }
        [left, right, rest @ ..] => {
            let centre = rest.first().map_or(0.0, |c| c.to_sample::<f32>() / 2.0);
            [left.to_sample::<f32>() + centre, right.to_sample::<f32>() + centre]
        }
    }
}

fn fatal(kind: ErrorKind) -> bool {
    !matches!(kind, ErrorKind::Xrun | ErrorKind::DeviceChanged | ErrorKind::RealtimeDenied)
}

fn build<T: SizedSample>(device: &cpal::Device, config: StreamConfig, source: Arc<Source>) -> Result<Stream, cpal::Error>
where
    f32: FromSample<T>,
{
    let channels = config.channels.max(1) as usize;
    let mut resampler = Resampler::new(config.sample_rate);
    let mut out = Vec::new();
    let failed = source.clone();
    device.build_input_stream(
        config,
        move |data: &[T], _: &cpal::InputCallbackInfo| {
            out.clear();
            for frame in data.chunks(channels) {
                resampler.push(stereo(frame), &mut out);
            }
            source.push(&out);
        },
        move |e: cpal::Error| {
            if fatal(e.kind()) {
                failed.lost.store(true, Ordering::Relaxed);
            }
        },
        None,
    )
}

fn open(device: &cpal::Device, output: bool, source: Arc<Source>) -> Result<Stream, String> {
    let supported = if output { device.default_output_config() } else { device.default_input_config() }
        .map_err(|e| e.to_string())?;
    let config = supported.config();
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(device, config, source),
        SampleFormat::F64 => build::<f64>(device, config, source),
        SampleFormat::I16 => build::<i16>(device, config, source),
        SampleFormat::I32 => build::<i32>(device, config, source),
        SampleFormat::U16 => build::<u16>(device, config, source),
        SampleFormat::U8 => build::<u8>(device, config, source),
        format => return Err(format!("unsupported sample format {format}")),
    }
    .map_err(|e| e.to_string())?;
    stream.play().map_err(|e| e.to_string())?;
    Ok(stream)
}

fn suppress(state: Option<&mut DenoiseState<'static>>, stereo: &mut [f32]) {
    let Some(state) = state else { return };
    let mut input = [0.0f32; FRAMES];
    let mut output = [0.0f32; FRAMES];
    for (sample, [left, right]) in input.iter_mut().zip(stereo.as_chunks::<2>().0) {
        *sample = (left + right) * 0.5 * PCM_SCALE;
    }
    state.process_frame(&mut output, &input);
    for (frame, &sample) in stereo.as_chunks_mut::<2>().0.iter_mut().zip(&output) {
        *frame = [sample / PCM_SCALE; 2];
    }
}

struct Mixer {
    mic: Option<Arc<Source>>,
    denoise: Option<Box<DenoiseState<'static>>>,
    system: Option<Arc<Source>>,
    chunks: Option<Sender<Chunk>>,
    stop: Arc<AtomicBool>,
}

impl Mixer {
    fn run(mut self, level: impl Fn(f32), lost: impl Fn()) {
        let mut mic = vec![0.0f32; FRAMES * 2];
        let mut system = vec![0.0f32; FRAMES * 2];
        let mut next = Instant::now() + TICK;
        let mut sum = 0.0f64;
        let mut count = 0usize;
        let mut ticks = 0u32;
        let mut reported = false;
        while !self.stop.load(Ordering::Relaxed) {
            let now = Instant::now();
            if now < next {
                std::thread::sleep(next - now);
                continue;
            }
            next += TICK;
            match &self.mic {
                Some(source) => source.take(&mut mic),
                None => mic.fill(0.0),
            }
            suppress(self.denoise.as_deref_mut(), &mut mic);
            match &self.system {
                Some(source) => source.take(&mut system),
                None => system.fill(0.0),
            }
            sum += mic.iter().map(|&s| (s as f64) * (s as f64)).sum::<f64>();
            count += mic.len();
            if let Some(tx) = &self.chunks {
                let chunk = mic
                    .iter()
                    .zip(&system)
                    .map(|(a, b)| ((a + b).clamp(-1.0, 1.0) * i16::MAX as f32) as i16)
                    .collect();
                let _ = tx.send(chunk);
            }
            ticks += 1;
            if ticks.is_multiple_of(LEVEL_TICKS) {
                level(((sum / count.max(1) as f64).sqrt() as f32).clamp(0.0, 1.0));
                sum = 0.0;
                count = 0;
            }
            if !reported && self.mic.as_ref().is_some_and(|s| s.lost.load(Ordering::Relaxed)) {
                reported = true;
                lost();
            }
        }
    }
}

pub struct Capture {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    chunks: Option<Receiver<Chunk>>,
    audio: bool,
}

impl Capture {
    pub fn start(
        mic: Option<String>,
        speaker: Option<String>,
        record: bool,
        denoise: bool,
        level: impl Fn(f32) + Send + 'static,
        lost: impl Fn() + Send + 'static,
    ) -> Result<Capture, String> {
        let stop = Arc::new(AtomicBool::new(false));
        let (tx, rx) = match record {
            true => {
                let (tx, rx) = mpsc::channel();
                (Some(tx), Some(rx))
            }
            false => (None, None),
        };
        let (ready, started) = mpsc::sync_channel::<Result<bool, String>>(1);
        let flag = stop.clone();
        let thread = std::thread::spawn(move || {
            let host = cpal::default_host();
            let mut streams = Vec::new();
            let mut open_source = |device: Option<cpal::Device>, output: bool, what: &str| -> Result<Option<Arc<Source>>, String> {
                let device = device.ok_or_else(|| format!("no {what} device"))?;
                let source = Arc::new(Source::default());
                streams.push(open(&device, output, source.clone()).map_err(|e| format!("{what}: {e}"))?);
                Ok(Some(source))
            };
            let mic = match mic.as_deref() {
                Some(id) => open_source(input(&host, id), false, "microphone"),
                None => Ok(None),
            };
            let mut loopback = None;
            let system = match (speaker.as_deref().filter(|_| record), &mic) {
                (Some("default"), Ok(_)) => {
                    let source = Arc::new(Source::default());
                    process_loopback(source.clone()).map(|l| {
                        loopback = Some(l);
                        Some(source)
                    })
                }
                (Some(id), Ok(_)) => open_source(output(&host, id), true, "speaker"),
                _ => Ok(None),
            };
            let (mic, system) = match (mic, system) {
                (Ok(mic), Ok(system)) => (mic, system),
                (Err(e), _) | (_, Err(e)) => {
                    let _ = ready.send(Err(e));
                    return;
                }
            };
            let _ = ready.send(Ok(mic.is_some() || system.is_some()));
            let denoise = (denoise && mic.is_some()).then(DenoiseState::new);
            Mixer { mic, denoise, system, chunks: tx, stop: flag }.run(level, lost);
            drop(loopback);
            drop(streams);
        });
        match started.recv() {
            Ok(Ok(audio)) => Ok(Capture { stop, thread: Some(thread), chunks: rx, audio }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err("audio thread failed".into())
            }
        }
    }

    pub fn chunks(&mut self) -> Option<Receiver<Chunk>> {
        self.chunks.take()
    }

    pub fn has_audio(&self) -> bool {
        self.audio
    }
}

impl Drop for Capture {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

struct Loopback {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Drop for Loopback {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[windows_core::implement(windows::Win32::Media::Audio::IActivateAudioInterfaceCompletionHandler)]
struct Activated(mpsc::SyncSender<()>);

impl windows::Win32::Media::Audio::IActivateAudioInterfaceCompletionHandler_Impl for Activated_Impl {
    fn ActivateCompleted(
        &self,
        _: windows::core::Ref<windows::Win32::Media::Audio::IActivateAudioInterfaceAsyncOperation>,
    ) -> windows::core::Result<()> {
        let _ = self.0.try_send(());
        Ok(())
    }
}

fn process_client() -> Result<windows::Win32::Media::Audio::IAudioClient, String> {
    use std::mem::ManuallyDrop;
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::BLOB;
    use windows::Win32::System::Com::StructuredStorage::{PROPVARIANT, PROPVARIANT_0, PROPVARIANT_0_0, PROPVARIANT_0_0_0};
    use windows::core::Interface;

    let mut params = AUDIOCLIENT_ACTIVATION_PARAMS {
        ActivationType: AUDIOCLIENT_ACTIVATION_TYPE_PROCESS_LOOPBACK,
        Anonymous: AUDIOCLIENT_ACTIVATION_PARAMS_0 {
            ProcessLoopbackParams: AUDIOCLIENT_PROCESS_LOOPBACK_PARAMS {
                TargetProcessId: unsafe { windows::Win32::System::Threading::GetCurrentProcessId() },
                ProcessLoopbackMode: PROCESS_LOOPBACK_MODE_EXCLUDE_TARGET_PROCESS_TREE,
            },
        },
    };
    let prop = ManuallyDrop::new(PROPVARIANT {
        Anonymous: PROPVARIANT_0 {
            Anonymous: ManuallyDrop::new(PROPVARIANT_0_0 {
                vt: windows::Win32::System::Variant::VT_BLOB,
                wReserved1: 0,
                wReserved2: 0,
                wReserved3: 0,
                Anonymous: PROPVARIANT_0_0_0 {
                    blob: BLOB {
                        cbSize: size_of::<AUDIOCLIENT_ACTIVATION_PARAMS>() as u32,
                        pBlobData: &mut params as *mut _ as *mut u8,
                    },
                },
            }),
        },
    });
    let (tx, rx) = mpsc::sync_channel(1);
    let handler: IActivateAudioInterfaceCompletionHandler = Activated(tx).into();
    let operation = unsafe {
        ActivateAudioInterfaceAsync(VIRTUAL_AUDIO_DEVICE_PROCESS_LOOPBACK, &IAudioClient::IID, Some(&*prop), &handler)
    }
    .map_err(|e| e.to_string())?;
    rx.recv_timeout(Duration::from_secs(5)).map_err(|_| "system audio did not start".to_string())?;
    let mut result = windows::core::HRESULT(0);
    let mut unknown = None;
    unsafe { operation.GetActivateResult(&mut result, &mut unknown) }.map_err(|e| e.to_string())?;
    result.ok().map_err(|e| e.to_string())?;
    unknown.ok_or("system audio unavailable")?.cast().map_err(|e| e.to_string())
}

fn process_loopback(source: Arc<Source>) -> Result<Loopback, String> {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Media::Audio::*;
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx, CoUninitialize};
    use windows::Win32::System::Threading::{CreateEventW, WaitForSingleObject};

    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    let (ready, started) = mpsc::sync_channel::<Result<(), String>>(1);
    let thread = std::thread::spawn(move || {
        let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        let open = || -> Result<(IAudioClient, IAudioCaptureClient, HANDLE), String> {
            let client = process_client()?;
            let format = WAVEFORMATEX {
                wFormatTag: 1,
                nChannels: 2,
                nSamplesPerSec: RATE,
                nAvgBytesPerSec: RATE * 4,
                nBlockAlign: 4,
                wBitsPerSample: 16,
                cbSize: 0,
            };
            let flags = AUDCLNT_STREAMFLAGS_LOOPBACK | AUDCLNT_STREAMFLAGS_EVENTCALLBACK | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM;
            unsafe { client.Initialize(AUDCLNT_SHAREMODE_SHARED, flags, 1_000_000, 0, &format, None) }.map_err(|e| e.to_string())?;
            let event = unsafe { CreateEventW(None, false, false, None) }.map_err(|e| e.to_string())?;
            unsafe { client.SetEventHandle(event) }.map_err(|e| e.to_string())?;
            let capture: IAudioCaptureClient = unsafe { client.GetService() }.map_err(|e| e.to_string())?;
            unsafe { client.Start() }.map_err(|e| e.to_string())?;
            Ok((client, capture, event))
        };
        let (client, capture, event) = match open() {
            Ok(opened) => {
                let _ = ready.send(Ok(()));
                opened
            }
            Err(e) => {
                let _ = ready.send(Err(e));
                unsafe { CoUninitialize() };
                return;
            }
        };
        let mut out = Vec::new();
        while !flag.load(Ordering::Relaxed) {
            unsafe { WaitForSingleObject(event, 100) };
            while let Ok(frames) = unsafe { capture.GetNextPacketSize() }
                && frames > 0
            {
                let (mut data, mut count, mut flags) = (std::ptr::null_mut(), 0u32, 0u32);
                if unsafe { capture.GetBuffer(&mut data, &mut count, &mut flags, None, None) }.is_err() {
                    break;
                }
                out.clear();
                if flags & AUDCLNT_BUFFERFLAGS_SILENT.0 as u32 != 0 || data.is_null() {
                    out.resize(count as usize * 2, 0.0);
                } else {
                    let samples = unsafe { std::slice::from_raw_parts(data as *const i16, count as usize * 2) };
                    out.extend(samples.iter().map(|&s| s as f32 / 32768.0));
                }
                let _ = unsafe { capture.ReleaseBuffer(count) };
                source.push(&out);
            }
        }
        let _ = unsafe { client.Stop() };
        let _ = unsafe { CloseHandle(event) };
        drop(capture);
        drop(client);
        unsafe { CoUninitialize() };
    });
    match started.recv() {
        Ok(Ok(())) => Ok(Loopback { stop, thread: Some(thread) }),
        Ok(Err(e)) => {
            let _ = thread.join();
            Err(e)
        }
        Err(_) => {
            let _ = thread.join();
            Err("system audio failed".into())
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn noise(state: &mut (u64, f32)) -> Vec<f32> {
        (0..FRAMES)
            .flat_map(|_| {
                state.0 = state.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let white = (state.0 >> 33) as f32 / (1u64 << 31) as f32 - 0.5;
                state.1 = 0.9 * state.1 + 0.1 * white;
                [state.1, state.1]
            })
            .collect()
    }

    fn rms(buffer: &[f32]) -> f32 {
        (buffer.iter().map(|s| s * s).sum::<f32>() / buffer.len() as f32).sqrt()
    }

    #[test]
    fn suppresses_fan_noise() {
        let mut seed = (1, 0.0);
        let mut state = DenoiseState::new();
        for _ in 0..100 {
            suppress(Some(&mut state), &mut noise(&mut seed));
        }
        let mut before = 0.0;
        let mut after = 0.0;
        let frames = 200;
        let start = Instant::now();
        for _ in 0..frames {
            let mut buffer = noise(&mut seed);
            before += rms(&buffer);
            suppress(Some(&mut state), &mut buffer);
            assert!(buffer.as_chunks::<2>().0.iter().all(|[l, r]| l == r));
            after += rms(&buffer);
        }
        let per_frame = start.elapsed() / frames;
        println!("rms {:.4} -> {:.4}, {per_frame:?} per 10 ms frame", before / frames as f32, after / frames as f32);
        assert!(after < before * 0.1);
    }

    #[test]
    fn off_leaves_buffer_unchanged() {
        let mut buffer = noise(&mut (7, 0.0));
        let copy = buffer.clone();
        suppress(None, &mut buffer);
        assert_eq!(buffer, copy);
    }
}
