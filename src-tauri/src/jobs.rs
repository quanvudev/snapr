use crate::overlay::{self, Shot};
use crate::tray::{refresh, tooltip};
use crate::{Action, State, capture, ocr, paddle};
use std::sync::mpsc::Sender;
use tauri::{AppHandle, Manager};
use tauri_plugin_notification::NotificationExt;
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use winit::event_loop::EventLoopProxy;

pub(crate) fn spawn(worker: AppHandle, proxy: EventLoopProxy<Action>) -> Sender<Job> {
    let (jobs, queue) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
        let mut backgrounds = capture::BackgroundCache::default();
        for job in queue {
            match job {
                Job::Save(shot) => work(&worker, &proxy, &mut backgrounds, shot),
                Job::Text(shot) => read_text(&worker, shot),
            }
        }
    });
    jobs
}

pub(crate) enum Job {
    Save(Shot),
    Text(Shot),
}

fn work(app: &AppHandle, proxy: &EventLoopProxy<Action>, backgrounds: &mut capture::BackgroundCache, shot: Shot) {
    let Some(img) = (match shot {
        Shot::Ready(img) => Some(img),
        Shot::Window { hwnd, border, frame, rect } => capture::window(hwnd, border)
            .filter(|img| img.width() > 0 && img.height() > 0)
            .or_else(|| overlay::crop(&frame, rect)),
        Shot::Region { .. } => None,
    }) else {
        return;
    };
    let cfg = app.state::<State>().cfg.read().unwrap().clone();
    let mut bg_error = None;
    let mut img = img;
    let img = if cfg.padding.enabled && cfg.padding.size > 0 {
        if cfg.padding.round {
            capture::mask_corners(&mut img, cfg.padding.radius as f32);
        }
        let image = match cfg.padding.image.as_str() {
            "" if cfg.padding.wallpaper => capture::wallpaper(),
            image => image.to_string(),
        };
        let bg = backgrounds.get(&image).map_err(|e| bg_error = Some(e)).ok().flatten();
        capture::pad(&img, cfg.padding.size, bg)
    } else {
        img
    };
    match capture::write(&img, &cfg) {
        Ok(path) => {
            refresh(app);
            if let Some(e) = bg_error {
                tooltip(app, &format!("Snapr: {e}"));
            }
            let _ = proxy.send_event(Action::Saved(img.clone(), path));
            if cfg.copy_to_clipboard
                && let Err(e) = capture::copy(&img)
            {
                tooltip(app, &format!("Snapr: {e}"));
            }
        }
        Err(e) => tooltip(app, &format!("Snapr: {e}")),
    }
}

fn read_text(app: &AppHandle, shot: Shot) {
    let Shot::Ready(img) = shot else { return };
    let state = app.state::<State>();
    let (vi, paddle) = {
        let cfg = state.cfg.read().unwrap();
        (cfg.language == "vi", paddle::selected(app, &cfg))
    };
    let failed = |e: String| if vi { format!("Lỗi quét chữ: {e}") } else { format!("Text capture failed: {e}") };
    let codes = ocr::qr(&img);
    let msg = if !codes.is_empty() {
        let n = codes.len();
        match arboard::Clipboard::new().and_then(|mut c| c.set_text(codes.join("\n"))) {
            Ok(()) if n == 1 && vi => "Đã sao chép mã QR".to_string(),
            Ok(()) if n == 1 => "QR code copied".to_string(),
            Ok(()) if vi => format!("Đã sao chép {n} mã QR"),
            Ok(()) => format!("Copied {n} QR codes"),
            Err(e) => failed(e.to_string()),
        }
    } else {
        match ocr::extract(&img, paddle.as_ref()) {
            Ok(text) if text.trim().is_empty() => if vi { "Không tìm thấy chữ" } else { "No text found" }.to_string(),
            Ok(text) => {
                let text = text.trim();
                let n = text.lines().count();
                match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
                    Ok(()) if vi => format!("Đã sao chép {n} dòng"),
                    Ok(()) => format!("Copied {n} lines"),
                    Err(e) => failed(e.to_string()),
                }
            }
            Err(e) => failed(e),
        }
    };
    let _ = app.notification().builder().title("Snapr").body(msg).show();
}
