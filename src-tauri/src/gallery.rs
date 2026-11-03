use crate::State;
use crate::capture;
use image::{ImageFormat, RgbaImage};
use serde::Serialize;
use std::collections::HashSet;
use std::fs::File;
use std::io::{Cursor, Read, Seek, SeekFrom};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;
use tauri::http::{Request, Response, StatusCode, header};
use tauri::{AppHandle, Manager, UriSchemeContext, UriSchemeResponder, Wry};
use tauri_plugin_opener::OpenerExt;
use windows::Win32::Foundation::SIZE;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows::Win32::UI::Shell::{
    IShellItemImageFactory, SHCreateItemFromParsingName, SIIGBF, SIIGBF_BIGGERSIZEOK, SIIGBF_ICONONLY, SIIGBF_THUMBNAILONLY,
};
use windows::core::HSTRING;
use windows_sys::Win32::Foundation::{GlobalFree, POINT};
use windows_sys::Win32::Graphics::Gdi::{BITMAP, DeleteObject, GetDC, GetObjectW, HBITMAP, ReleaseDC};
use windows_sys::Win32::System::DataExchange::{CloseClipboard, EmptyClipboard, OpenClipboard, SetClipboardData};
use windows_sys::Win32::System::Memory::{GMEM_MOVEABLE, GMEM_ZEROINIT, GlobalAlloc, GlobalLock, GlobalUnlock};
use windows_sys::Win32::System::Ole::CF_HDROP;
use windows_sys::Win32::UI::Shell::{
    DROPFILES, FO_DELETE, FOF_ALLOWUNDO, FOF_NOCONFIRMATION, FOF_NOERRORUI, FOF_SILENT, SHFILEOPSTRUCTW, SHFileOperationW,
};

const THUMB_SIZE: i32 = 256;
const THUMB_WORKERS: usize = 3;
const THUMB_MAX_AGE: &str = "max-age=3600";
const MEDIA_CHUNK: u64 = 1024 * 1024;
const PNG_MIME: &str = "image/png";
const SIDECAR_EXT: &str = "srt";

#[derive(Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Photo,
    Video,
    Other,
}

const MEDIA: &[(&str, Kind, &str)] = &[
    ("png", Kind::Photo, PNG_MIME),
    ("jpg", Kind::Photo, "image/jpeg"),
    ("jpeg", Kind::Photo, "image/jpeg"),
    ("gif", Kind::Photo, "image/gif"),
    ("webp", Kind::Photo, "image/webp"),
    ("bmp", Kind::Photo, "image/bmp"),
    ("mp4", Kind::Video, "video/mp4"),
    ("webm", Kind::Video, "video/webm"),
];

fn media(path: &Path) -> Option<(Kind, &'static str)> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    MEDIA.iter().find(|(known, ..)| *known == ext).map(|&(_, kind, mime)| (kind, mime))
}

impl Kind {
    fn of(path: &Path) -> Kind {
        media(path).map_or(Kind::Other, |(kind, _)| kind)
    }
}

#[derive(Serialize)]
pub struct Item {
    path: String,
    date_ms: u64,
    size: u64,
    kind: Kind,
}

#[derive(Serialize)]
pub struct BulkFailure {
    path: String,
    error: String,
}

fn sidecar_stem(path: &Path) -> Option<String> {
    Some(path.file_stem()?.to_str()?.to_lowercase())
}

fn is_sidecar(path: &Path) -> bool {
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(SIDECAR_EXT))
}

fn list(dir: &Path) -> Vec<Item> {
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    let entries: Vec<_> = entries.flatten().collect();
    let video_stems: HashSet<String> = entries
        .iter()
        .map(|entry| entry.path())
        .filter(|path| Kind::of(path) == Kind::Video)
        .filter_map(|path| sidecar_stem(&path))
        .collect();
    let mut items: Vec<Item> = entries
        .iter()
        .filter_map(|entry| {
            let path = entry.path();
            if is_sidecar(&path) && sidecar_stem(&path).is_some_and(|stem| video_stems.contains(&stem)) {
                return None;
            }
            let kind = Kind::of(&path);
            let meta = entry.metadata().ok().filter(|m| m.is_file())?;
            let modified = meta.modified().ok()?;
            let date = meta.created().map_or(modified, |created| created.min(modified));
            let date_ms = date.duration_since(UNIX_EPOCH).ok()?.as_millis() as u64;
            Some(Item { path: path.to_str()?.to_string(), date_ms, size: meta.len(), kind })
        })
        .collect();
    items.sort_by_key(|item| std::cmp::Reverse(item.date_ms));
    items
}

fn plain(path: &Path) -> Option<PathBuf> {
    let s = path.to_str()?;
    Some(match s.strip_prefix(r"\\?\UNC\") {
        Some(rest) => PathBuf::from(format!(r"\\{rest}")),
        None => PathBuf::from(s.strip_prefix(r"\\?\").unwrap_or(s)),
    })
}

pub(crate) fn guard(app: &AppHandle, path: &str) -> Option<PathBuf> {
    let dir = app.state::<State>().cfg.read().unwrap().dir().canonicalize().ok()?;
    let file = Path::new(path).canonicalize().ok()?;
    if file.parent() != Some(dir.as_path()) || !file.is_file() {
        return None;
    }
    plain(&file)
}

fn guarded(app: &AppHandle, path: &str) -> Result<PathBuf, String> {
    guard(app, path).ok_or_else(|| "path outside the output folder".to_string())
}

#[tauri::command]
pub fn gallery_list(state: tauri::State<State>) -> Vec<Item> {
    let dir = state.cfg.read().unwrap().dir();
    list(&dir)
}

#[tauri::command]
pub fn gallery_open(app: AppHandle, path: String) -> Result<(), String> {
    let path = guarded(&app, &path)?;
    app.opener().open_path(path.display().to_string(), None::<&str>).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn gallery_reveal(app: AppHandle, path: String) -> Result<(), String> {
    let path = guarded(&app, &path)?;
    std::process::Command::new("explorer")
        .arg(format!("/select,{}", path.display()))
        .spawn()
        .map(drop)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn gallery_copy(app: AppHandle, path: String) -> Result<(), String> {
    let path = guarded(&app, &path)?;
    if Kind::of(&path) != Kind::Photo {
        return Err("not an image".into());
    }
    let img = image::open(&path).map_err(|e| e.to_string())?.to_rgba8();
    capture::copy(&img).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn gallery_copy_files(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    let mut files = Vec::with_capacity(paths.len());
    for path in &paths {
        let file = guarded(&app, path)?;
        if Kind::of(&file) != Kind::Photo {
            return Err("not an image".into());
        }
        files.push(file);
    }
    if files.is_empty() {
        return Err("no images".into());
    }
    clipboard_files(&files)
}

fn clipboard_files(paths: &[PathBuf]) -> Result<(), String> {
    let mut data = Vec::new();
    for path in paths {
        data.extend(path.as_os_str().encode_wide());
        data.push(0);
    }
    data.push(0);
    let header = size_of::<DROPFILES>();
    let bytes = header + data.len() * 2;
    let drop = DROPFILES { pFiles: header as u32, pt: POINT { x: 0, y: 0 }, fNC: 0, fWide: 1 };
    unsafe {
        let handle = GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes);
        if handle.is_null() {
            return Err("clipboard allocation failed".into());
        }
        let target = GlobalLock(handle);
        if target.is_null() {
            GlobalFree(handle);
            return Err("clipboard lock failed".into());
        }
        std::ptr::write_unaligned(target.cast::<DROPFILES>(), drop);
        std::ptr::copy_nonoverlapping(data.as_ptr().cast::<u8>(), target.cast::<u8>().add(header), data.len() * 2);
        let _ = GlobalUnlock(handle);
        if OpenClipboard(null_mut()) == 0 {
            GlobalFree(handle);
            return Err("clipboard busy".into());
        }
        if EmptyClipboard() == 0 || SetClipboardData(CF_HDROP as u32, handle as _).is_null() {
            GlobalFree(handle);
            CloseClipboard();
            return Err("clipboard write failed".into());
        }
        CloseClipboard();
    }
    Ok(())
}

#[tauri::command]
pub fn gallery_pin(app: AppHandle, path: String) -> Result<(), String> {
    let path = guarded(&app, &path)?;
    if Kind::of(&path) != Kind::Photo {
        return Err("not an image".into());
    }
    app.state::<State>()
        .proxy
        .lock()
        .unwrap()
        .send_event(crate::Action::Pin(path))
        .map_err(|_| "pin failed".to_string())
}

#[derive(Serialize)]
pub struct Recognized {
    qr: Vec<String>,
    text: String,
}

#[tauri::command]
pub async fn gallery_text(app: AppHandle, path: String) -> Result<Recognized, String> {
    let path = guarded(&app, &path)?;
    if Kind::of(&path) != Kind::Photo {
        return Err("not an image".into());
    }
    let paddle = crate::paddle::selected(&app, &app.state::<State>().cfg.read().unwrap());
    tauri::async_runtime::spawn_blocking(move || {
        let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
        let img = image::open(&path).map_err(|e| e.to_string())?.to_rgba8();
        let qr = crate::ocr::qr(&img);
        let text = match crate::ocr::extract(&img, paddle.as_ref()) {
            Ok(text) => text.trim().to_string(),
            Err(_) if !qr.is_empty() => String::new(),
            Err(e) => return Err(e),
        };
        Ok(Recognized { qr, text })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn gallery_pin_many(app: AppHandle, paths: Vec<String>) -> Result<(), String> {
    let mut files = Vec::with_capacity(paths.len());
    for path in &paths {
        let file = guarded(&app, path)?;
        if Kind::of(&file) != Kind::Photo {
            return Err("not an image".into());
        }
        files.push(file);
    }
    if files.is_empty() {
        return Err("no images".into());
    }
    app.state::<State>()
        .proxy
        .lock()
        .unwrap()
        .send_event(crate::Action::PinMany(files))
        .map_err(|_| "pin failed".to_string())
}

pub(crate) fn delete_file(path: &Path) -> Result<(), String> {
    let from: Vec<u16> = path.as_os_str().encode_wide().chain([0, 0]).collect();
    let mut op = SHFILEOPSTRUCTW {
        wFunc: FO_DELETE,
        pFrom: from.as_ptr(),
        fFlags: (FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI) as u16,
        ..Default::default()
    };
    let status = unsafe { SHFileOperationW(&mut op) };
    if status != 0 || op.fAnyOperationsAborted != 0 {
        return Err(format!("delete failed ({status})"));
    }
    Ok(())
}

fn delete_with_sidecar(path: &Path) -> Result<(), String> {
    delete_file(path)?;
    if Kind::of(path) == Kind::Video {
        let sidecar = path.with_extension(SIDECAR_EXT);
        if sidecar.is_file() {
            let _ = delete_file(&sidecar);
        }
    }
    Ok(())
}

#[tauri::command]
pub fn gallery_delete(app: AppHandle, path: String) -> Result<(), String> {
    let path = guarded(&app, &path)?;
    delete_with_sidecar(&path)
}

#[tauri::command]
pub fn gallery_delete_many(app: AppHandle, paths: Vec<String>) -> Result<Vec<BulkFailure>, String> {
    let validated = paths
        .into_iter()
        .map(|path| guarded(&app, &path).map(|file| (path, file)))
        .collect::<Result<Vec<_>, _>>()?;
    let mut failures = Vec::new();
    for (path, file) in validated {
        if let Err(error) = delete_with_sidecar(&file) {
            failures.push(BulkFailure { path, error });
        }
    }
    Ok(failures)
}

fn decode(s: &str) -> Option<String> {
    let mut out = Vec::with_capacity(s.len());
    let mut bytes = s.bytes();
    while let Some(b) = bytes.next() {
        out.push(if b == b'%' {
            let hex = [bytes.next()?, bytes.next()?];
            u8::from_str_radix(std::str::from_utf8(&hex).ok()?, 16).ok()?
        } else {
            b
        });
    }
    String::from_utf8(out).ok()
}

fn requested(app: &AppHandle, request: &Request<Vec<u8>>) -> Option<PathBuf> {
    let raw = request.uri().path().strip_prefix('/')?;
    guard(app, &decode(raw)?)
}

fn not_found() -> Response<Vec<u8>> {
    let mut response = Response::new(Vec::new());
    *response.status_mut() = StatusCode::NOT_FOUND;
    response
}

unsafe fn rgba(bitmap: HBITMAP) -> Option<RgbaImage> {
    unsafe {
        let mut info: BITMAP = std::mem::zeroed();
        if GetObjectW(bitmap, size_of::<BITMAP>() as i32, (&mut info as *mut BITMAP).cast()) == 0 {
            return None;
        }
        let (w, h) = (info.bmWidth, info.bmHeight.abs());
        if w <= 0 || h <= 0 {
            return None;
        }
        let dc = GetDC(null_mut());
        let raw = capture::dib(dc, bitmap, w, h);
        ReleaseDC(null_mut(), dc);
        let mut raw = raw?;
        let opaque = raw.as_chunks::<4>().0.iter().all(|p| p[3] == 0);
        for p in raw.as_chunks_mut::<4>().0 {
            let a = if opaque { u8::MAX } else { p[3] };
            let straight = |c: u8| match a {
                0 | u8::MAX => c,
                _ => (c as u32 * u8::MAX as u32 / a as u32).min(u8::MAX as u32) as u8,
            };
            *p = [straight(p[2]), straight(p[1]), straight(p[0]), a];
        }
        RgbaImage::from_raw(w as u32, h as u32, raw)
    }
}

fn shell_rgba(factory: &IShellItemImageFactory, flags: SIIGBF, side: i32) -> Option<RgbaImage> {
    let size = SIZE { cx: side, cy: side };
    let bitmap = unsafe { factory.GetImage(size, flags | SIIGBF_BIGGERSIZEOK) }.ok()?;
    let img = unsafe { rgba(bitmap.0) };
    unsafe { DeleteObject(bitmap.0) };
    img
}

fn png(img: &RgbaImage) -> Option<Vec<u8>> {
    let mut png = Vec::new();
    img.write_to(&mut Cursor::new(&mut png), ImageFormat::Png).ok()?;
    Some(png)
}

fn shell_image(factory: &IShellItemImageFactory, flags: SIIGBF) -> Option<Vec<u8>> {
    png(&shell_rgba(factory, flags, THUMB_SIZE)?)
}

fn shell_factory(path: &Path) -> Option<IShellItemImageFactory> {
    unsafe { SHCreateItemFromParsingName(&HSTRING::from(path), None) }.ok()
}

pub fn icon_rgba(path: &Path, side: i32) -> Option<RgbaImage> {
    let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    shell_rgba(&shell_factory(path)?, SIIGBF_ICONONLY, side)
}

pub fn icon_png(path: &Path, side: i32) -> Option<Vec<u8>> {
    png(&icon_rgba(path, side)?)
}

fn thumbnail(path: &Path) -> Option<Vec<u8>> {
    let factory = shell_factory(path)?;
    if Kind::of(path) != Kind::Other
        && let Some(png) = shell_image(&factory, SIIGBF_THUMBNAILONLY)
    {
        return Some(png);
    }
    shell_image(&factory, SIIGBF_ICONONLY)
}

fn thumb_response(path: &Path) -> Response<Vec<u8>> {
    thumbnail(path)
        .and_then(|png| {
            Response::builder()
                .header(header::CONTENT_TYPE, PNG_MIME)
                .header(header::CACHE_CONTROL, THUMB_MAX_AGE)
                .body(png)
                .ok()
        })
        .unwrap_or_else(not_found)
}

pub fn thumb_scheme() -> impl Fn(UriSchemeContext<'_, Wry>, Request<Vec<u8>>, UriSchemeResponder) + Send + Sync + 'static {
    let (tx, rx): (Sender<(PathBuf, UriSchemeResponder)>, _) = std::sync::mpsc::channel();
    let rx = Arc::new(Mutex::new(rx));
    for _ in 0..THUMB_WORKERS {
        let rx = rx.clone();
        std::thread::spawn(move || {
            let _ = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
            loop {
                let Ok((path, responder)) = rx.lock().unwrap().recv() else { break };
                responder.respond(thumb_response(&path));
            }
        });
    }
    move |ctx, request, responder| match requested(ctx.app_handle(), &request) {
        Some(path) => {
            let _ = tx.send((path, responder));
        }
        None => responder.respond(not_found()),
    }
}

fn parse_range(value: &str, len: u64) -> Option<(u64, u64)> {
    let (start, end) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start: u64 = start.trim().parse().ok()?;
    if start >= len {
        return None;
    }
    let end = match end.trim() {
        "" => len - 1,
        end => end.parse::<u64>().ok()?.min(len - 1),
    };
    (start <= end).then_some((start, end.min(start + MEDIA_CHUNK - 1)))
}

fn serve(path: &Path, range: Option<&str>) -> Option<Response<Vec<u8>>> {
    let (_, mime) = media(path)?;
    let mut file = File::open(path).ok()?;
    let len = file.metadata().ok()?.len();
    let builder = Response::builder().header(header::CONTENT_TYPE, mime).header(header::ACCEPT_RANGES, "bytes");
    let Some((start, end)) = range.and_then(|r| parse_range(r, len)) else {
        let mut body = Vec::with_capacity(len as usize);
        file.read_to_end(&mut body).ok()?;
        return builder.body(body).ok();
    };
    let mut body = vec![0; (end - start + 1) as usize];
    file.seek(SeekFrom::Start(start)).ok()?;
    file.read_exact(&mut body).ok()?;
    builder
        .status(StatusCode::PARTIAL_CONTENT)
        .header(header::CONTENT_RANGE, format!("bytes {start}-{end}/{len}"))
        .body(body)
        .ok()
}

pub fn media_scheme(ctx: UriSchemeContext<'_, Wry>, request: Request<Vec<u8>>, responder: UriSchemeResponder) {
    let Some(path) = requested(ctx.app_handle(), &request) else {
        return responder.respond(not_found());
    };
    let range = request.headers().get(header::RANGE).and_then(|v| v.to_str().ok()).map(str::to_owned);
    tauri::async_runtime::spawn_blocking(move || {
        responder.respond(serve(&path, range.as_deref()).unwrap_or_else(not_found));
    });
}
