use crate::State;
use crate::library;
use crate::queue::{self, CANCELLED, Progress, Task};
use crate::video;
use ::gif::{DisposalMethod, Repeat};
use image::RgbaImage;
use image::imageops::{FilterType, crop_imm, resize};
use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{AppHandle, Manager};
use windows::Win32::Media::MediaFoundation::{
    IMF2DBuffer, IMFMediaType, IMFSample, IMFSourceReader, MF_MT_FRAME_RATE, MF_MT_FRAME_SIZE, MF_MT_MAJOR_TYPE,
    MF_MT_MINIMUM_DISPLAY_APERTURE, MF_MT_SUBTYPE, MF_PD_DURATION, MF_SOURCE_READER_ALL_STREAMS,
    MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING,
    MF_SOURCE_READER_FIRST_VIDEO_STREAM, MF_SOURCE_READER_MEDIASOURCE,
    MF_SOURCE_READERF_ENDOFSTREAM, MF_VERSION, MFCreateAttributes, MFCreateMediaType, MFCreateSourceReaderFromURL,
    MFMediaType_Video, MFSTARTUP_LITE, MFShutdown, MFStartup, MFVideoArea, MFVideoFormat_RGB32,
};
use windows::Win32::System::Com::StructuredStorage::PROPVARIANT;
use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
use windows::core::{GUID, HSTRING, Interface};

const KIND: &str = "gif";
const FPS_PRESETS: [u32; 3] = [10, 15, 20];
const WIDTH_PRESETS: [u32; 3] = [480, 640, 800];
const END_SLACK_MS: u64 = 100;
const HNS_PER_SECOND: u64 = 10_000_000;
pub(crate) const HNS_PER_MS: u64 = 10_000;
const CENTISECONDS_PER_SECOND: u64 = 100;
const ENCODER_SPEED: i32 = 10;
const BYTES_PER_PIXEL: usize = 4;
const OPAQUE: u8 = 255;
const READER_ATTRIBUTES: u32 = 2;

pub(crate) struct Media;

impl Media {
    pub(crate) fn start() -> Result<Media, String> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
            MFStartup(MF_VERSION, MFSTARTUP_LITE).map_err(|e| e.to_string())?;
        }
        Ok(Media)
    }
}

impl Drop for Media {
    fn drop(&mut self) {
        let _ = unsafe { MFShutdown() };
    }
}

pub(crate) struct Video {
    reader: IMFSourceReader,
    pub(crate) width: u32,
    pub(crate) height: u32,
    left: u32,
    top: u32,
    stride: usize,
    pub(crate) duration: u64,
    pub(crate) fps: Option<f64>,
}

fn stream() -> u32 {
    MF_SOURCE_READER_FIRST_VIDEO_STREAM.0 as u32
}

fn aperture(ty: &IMFMediaType, width: u32, height: u32) -> (u32, u32, u32, u32) {
    let mut area = MFVideoArea::default();
    let bytes = unsafe {
        std::slice::from_raw_parts_mut((&mut area as *mut MFVideoArea).cast::<u8>(), size_of::<MFVideoArea>())
    };
    let found = unsafe { ty.GetBlob(&MF_MT_MINIMUM_DISPLAY_APERTURE, bytes, None) }.is_ok();
    let left = (area.OffsetX.value.max(0) as u32).min(width);
    let top = (area.OffsetY.value.max(0) as u32).min(height);
    if !found || area.Area.cx <= 0 || area.Area.cy <= 0 || left == width || top == height {
        return (0, 0, width, height);
    }
    (left, top, (area.Area.cx as u32).min(width - left), (area.Area.cy as u32).min(height - top))
}

pub(crate) fn open(path: &Path) -> windows::core::Result<Video> {
    unsafe {
        let mut attrs = None;
        MFCreateAttributes(&mut attrs, READER_ATTRIBUTES)?;
        let attrs = attrs.ok_or_else(windows::core::Error::empty)?;
        attrs.SetUINT32(&MF_READWRITE_ENABLE_HARDWARE_TRANSFORMS, 1)?;
        attrs.SetUINT32(&MF_SOURCE_READER_ENABLE_ADVANCED_VIDEO_PROCESSING, 1)?;
        let reader = MFCreateSourceReaderFromURL(&HSTRING::from(path.as_os_str()), &attrs)?;
        reader.SetStreamSelection(MF_SOURCE_READER_ALL_STREAMS.0 as u32, false)?;
        reader.SetStreamSelection(stream(), true)?;
        let ty = MFCreateMediaType()?;
        ty.SetGUID(&MF_MT_MAJOR_TYPE, &MFMediaType_Video)?;
        ty.SetGUID(&MF_MT_SUBTYPE, &MFVideoFormat_RGB32)?;
        reader.SetCurrentMediaType(stream(), None, &ty)?;
        let current = reader.GetCurrentMediaType(stream())?;
        let size = current.GetUINT64(&MF_MT_FRAME_SIZE)?;
        let frame_width = (size >> 32) as u32;
        let (left, top, width, height) = aperture(&current, frame_width, size as u32);
        let stride = frame_width as usize * BYTES_PER_PIXEL;
        let duration = reader.GetPresentationAttribute(MF_SOURCE_READER_MEDIASOURCE.0 as u32, &MF_PD_DURATION)?;
        let duration = u64::try_from(&duration)?;
        let fps = current.GetUINT64(&MF_MT_FRAME_RATE).ok().and_then(|rate| {
            let (num, den) = ((rate >> 32) as u32, rate as u32);
            (num > 0 && den > 0).then(|| f64::from(num) / f64::from(den))
        });
        Ok(Video { reader, width, height, left, top, stride, duration, fps })
    }
}

pub(crate) fn probe(path: &Path) -> Result<(u32, u32, u64), String> {
    let _media = Media::start()?;
    let video = open(path).map_err(|e| e.to_string())?;
    Ok((video.width, video.height, video.duration / HNS_PER_MS))
}

impl Video {
    fn seek(&self, at: u64) -> windows::core::Result<()> {
        let position = PROPVARIANT::from(at as i64);
        unsafe { self.reader.SetCurrentPosition(&GUID::zeroed(), &position) }
    }

    pub(crate) fn read(&self) -> windows::core::Result<Option<(IMFSample, u64)>> {
        loop {
            let mut flags = 0u32;
            let mut stamp = 0i64;
            let mut sample = None;
            unsafe {
                self.reader.ReadSample(stream(), 0, None, Some(&mut flags), Some(&mut stamp), Some(&mut sample))?;
            }
            if flags & MF_SOURCE_READERF_ENDOFSTREAM.0 as u32 != 0 {
                return Ok(None);
            }
            if let Some(sample) = sample {
                return Ok(Some((sample, stamp.max(0) as u64)));
            }
        }
    }

    fn rows(&self, sample: &IMFSample, height: u32, mut row: impl FnMut(usize, &[u8])) -> windows::core::Result<()> {
        let buffer = unsafe { sample.ConvertToContiguousBuffer()? };
        let mut copy = |scan0: *const u8, pitch: isize| {
            for y in 0..height as usize {
                let line = unsafe { scan0.offset((y as isize + self.top as isize) * pitch) };
                let line = unsafe { line.add(self.left as usize * BYTES_PER_PIXEL) };
                row(y, unsafe { std::slice::from_raw_parts(line, self.width as usize * BYTES_PER_PIXEL) });
            }
        };
        if let Ok(planar) = buffer.cast::<IMF2DBuffer>() {
            let mut scan0 = std::ptr::null_mut();
            let mut pitch = 0i32;
            unsafe { planar.Lock2D(&mut scan0, &mut pitch)? };
            copy(scan0 as *const u8, pitch as isize);
            unsafe { planar.Unlock2D()? };
        } else {
            let mut data = std::ptr::null_mut();
            unsafe { buffer.Lock(&mut data, None, None)? };
            copy(data as *const u8, self.stride as isize);
            unsafe { buffer.Unlock()? };
        }
        Ok(())
    }

    pub(crate) fn image(&self, sample: &IMFSample) -> windows::core::Result<RgbaImage> {
        let mut img = RgbaImage::new(self.width, self.height);
        let row_len = self.width as usize * BYTES_PER_PIXEL;
        self.rows(sample, self.height, |y, src| {
            let dst = &mut img.as_mut()[y * row_len..(y + 1) * row_len];
            for (px, bgra) in dst.as_chunks_mut::<BYTES_PER_PIXEL>().0.iter_mut().zip(src.as_chunks::<BYTES_PER_PIXEL>().0) {
                *px = [bgra[2], bgra[1], bgra[0], OPAQUE];
            }
        })?;
        Ok(img)
    }

    pub(crate) fn bottom_up_bgra(&self, sample: &IMFSample, width: u32, height: u32, out: &mut Vec<u8>) -> windows::core::Result<()> {
        let row_len = width as usize * BYTES_PER_PIXEL;
        out.resize(row_len * height as usize, 0);
        self.rows(sample, height, |y, src| {
            let start = (height as usize - 1 - y) * row_len;
            out[start..start + row_len].copy_from_slice(&src[..row_len]);
        })
    }
}

pub struct GifTask {
    path: PathBuf,
    out_dir: PathBuf,
    start_ms: u64,
    end_ms: u64,
    width: Option<u32>,
    fps: u32,
}

struct Pending {
    image: RgbaImage,
    left: u32,
    top: u32,
    until: u64,
}

struct Encoder<'a> {
    gif: ::gif::Encoder<&'a mut BufWriter<File>>,
    width: u32,
    height: u32,
    fps: u64,
    previous: Option<RgbaImage>,
    pending: Option<Pending>,
    written_cs: u64,
}

fn changed(previous: &RgbaImage, next: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let row_len = next.width() as usize * BYTES_PER_PIXEL;
    let rows = previous.as_raw().chunks_exact(row_len).zip(next.as_raw().chunks_exact(row_len));
    let mut bounds: Option<(usize, usize, usize, usize)> = None;
    for (y, (a, b)) in rows.enumerate() {
        if a == b {
            continue;
        }
        let pixels = || a.as_chunks::<BYTES_PER_PIXEL>().0.iter().zip(b.as_chunks::<BYTES_PER_PIXEL>().0);
        let first = pixels().position(|(p, q)| p != q).unwrap_or(0);
        let last = pixels().rposition(|(p, q)| p != q).unwrap_or(first);
        bounds = Some(bounds.map_or((first, y, last, y), |(x0, y0, x1, _)| (x0.min(first), y0, x1.max(last), y)));
    }
    bounds.map(|(x0, y0, x1, y1)| (x0 as u32, y0 as u32, (x1 - x0 + 1) as u32, (y1 - y0 + 1) as u32))
}

impl<'a> Encoder<'a> {
    fn new(writer: &'a mut BufWriter<File>, width: u32, height: u32, fps: u32) -> Result<Encoder<'a>, String> {
        let side = |v: u32| u16::try_from(v).map_err(|_| "video too large for a GIF".to_string());
        let mut gif = ::gif::Encoder::new(writer, side(width)?, side(height)?, &[]).map_err(|e| e.to_string())?;
        gif.set_repeat(Repeat::Infinite).map_err(|e| e.to_string())?;
        Ok(Encoder { gif, width, height, fps: u64::from(fps), previous: None, pending: None, written_cs: 0 })
    }

    fn push(&mut self, video: &Video, sample: &IMFSample, until: u64) -> Result<(), String> {
        let img = video.image(sample).map_err(|e| e.to_string())?;
        let img = if img.width() == self.width { img } else { resize(&img, self.width, self.height, FilterType::Triangle) };
        let rect = match &self.previous {
            Some(previous) => changed(previous, &img),
            None => Some((0, 0, self.width, self.height)),
        };
        let Some((left, top, width, height)) = rect else {
            if let Some(pending) = self.pending.as_mut() {
                pending.until = until;
            }
            return Ok(());
        };
        self.flush()?;
        let image = crop_imm(&img, left, top, width, height).to_image();
        self.pending = Some(Pending { image, left, top, until });
        self.previous = Some(img);
        Ok(())
    }

    fn flush(&mut self) -> Result<(), String> {
        let Some(pending) = self.pending.take() else { return Ok(()) };
        let end_cs = pending.until * CENTISECONDS_PER_SECOND / self.fps;
        let delay = end_cs.saturating_sub(self.written_cs);
        self.written_cs = end_cs;
        let (width, height) = pending.image.dimensions();
        let mut pixels = pending.image.into_raw();
        let mut frame = ::gif::Frame::from_rgba_speed(width as u16, height as u16, &mut pixels, ENCODER_SPEED);
        frame.left = pending.left as u16;
        frame.top = pending.top as u16;
        frame.delay = u16::try_from(delay).unwrap_or(u16::MAX);
        frame.dispose = DisposalMethod::Keep;
        self.gif.write_frame(&frame).map_err(|e| e.to_string())
    }

    fn finish(mut self) -> Result<(), String> {
        self.flush()?;
        self.gif.into_inner().map(drop).map_err(|e| e.to_string())
    }
}

impl GifTask {
    pub fn validated(
        app: &AppHandle,
        path: &str,
        start_ms: u64,
        end_ms: Option<u64>,
        width: Option<u32>,
        fps: u32,
    ) -> Result<GifTask, String> {
        if !FPS_PRESETS.contains(&fps) {
            return Err("unsupported frame rate".into());
        }
        if width.is_some_and(|w| !WIDTH_PRESETS.contains(&w)) {
            return Err("unsupported width".into());
        }
        let path = video::guarded_mp4(app, path)?;
        let (_, _, duration) = probe(&path)?;
        let end_ms = end_ms.unwrap_or(duration);
        if start_ms >= end_ms || end_ms > duration + END_SLACK_MS {
            return Err("invalid range".into());
        }
        let out_dir = library::images(&app.state::<State>().cfg.read().unwrap());
        Ok(GifTask { path, out_dir, start_ms, end_ms: end_ms.min(duration), width, fps })
    }

    fn stem(&self) -> String {
        self.path.file_stem().map(|s| s.to_string_lossy().into_owned()).unwrap_or_default()
    }

    fn encode(&self, file: File, progress: &Progress, cancel: &AtomicBool) -> Result<(), String> {
        if !self.path.is_file() {
            return Err("video not found".into());
        }
        let _media = Media::start()?;
        let video = open(&self.path).map_err(|e| e.to_string())?;
        let start = self.start_ms * HNS_PER_MS;
        let end = (self.end_ms * HNS_PER_MS).min(video.duration);
        let fps = u64::from(self.fps);
        let target = |k: u64| start + k * HNS_PER_SECOND / fps;
        let total = (end.saturating_sub(start) * fps).div_ceil(HNS_PER_SECOND);
        progress.set(0, total);
        let width = self.width.map_or(video.width, |w| w.min(video.width));
        let height = ((u64::from(video.height) * u64::from(width)) / u64::from(video.width)).max(1) as u32;
        let mut writer = BufWriter::new(file);
        let mut encoder = Encoder::new(&mut writer, width, height, self.fps)?;
        video.seek(start).map_err(|e| e.to_string())?;
        let mut done = 0u64;
        let mut pending: Option<IMFSample> = None;
        while done < total {
            if cancel.load(Ordering::Relaxed) {
                return Err(CANCELLED.into());
            }
            let Some((sample, stamp)) = video.read().map_err(|e| e.to_string())? else { break };
            if let Some(previous) = pending.take() {
                let mut reached = done;
                while reached < total && target(reached) < stamp {
                    reached += 1;
                }
                if reached > done {
                    encoder.push(&video, &previous, reached)?;
                    done = reached;
                    progress.set(done, total);
                }
            }
            if stamp >= end {
                break;
            }
            pending = Some(sample);
        }
        if let Some(last) = pending
            && done < total
        {
            encoder.push(&video, &last, total)?;
            done = total;
            progress.set(done, total);
        }
        if done == 0 {
            return Err("no frames decoded".into());
        }
        encoder.finish()?;
        writer.into_inner().map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl Task for GifTask {
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
        std::fs::create_dir_all(&self.out_dir).map_err(|e| e.to_string())?;
        queue::write_part(&self.out_dir, &self.stem(), KIND, |part| {
            let file = File::create(part).map_err(|e| e.to_string())?;
            self.encode(file, progress, cancel)
        })
    }
}
