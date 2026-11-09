pub mod engine;
pub mod languages;
pub mod prompt;
pub mod task;

use crate::State;
use crate::events;
use crate::fetch::{self, Fetch, Remote};
use serde::Serialize;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

pub const LICENSE_REQUIRED: &str = "LICENSE_REQUIRED";
const STORE_DIR: &str = "translate";
const MODEL_DIR: &str = "hymt-1.8b";
const DATA_FILE: &str = "model_q4.onnx_data";
const GENERATION_FILE: &str = "generation_config.json";
const THROTTLE: Duration = Duration::from_millis(100);
macro_rules! pinned {
    ($path:literal) => {
        concat!("https://huggingface.co/onnx-community/HY-MT1.5-1.8B-ONNX/resolve/2f11819b25de08cecd344735cdfa5136ade41a67", $path)
    };
}

const FILES: [(&str, Remote); 5] = [
    (
        engine::MODEL_FILE,
        Remote {
            url: pinned!("/onnx/model_q4.onnx"),
            sha256: "9ffed3b4d2321e42e4c79b970ee73ff80bd10fd4f89208a2b6ee6a5ef59b800c",
            bytes: 448_829,
        },
    ),
    (
        DATA_FILE,
        Remote {
            url: pinned!("/onnx/model_q4.onnx_data"),
            sha256: "ab975a558df7dbb1ed863d983e8d530e85eeb16de6f7028542b9c0af9a2222eb",
            bytes: 1_405_788_224,
        },
    ),
    (
        engine::TOKENIZER_FILE,
        Remote {
            url: pinned!("/tokenizer.json"),
            sha256: "1b119fa76913d752f1003222865691e56f69fbff53b257acc10f8fd183d4f70b",
            bytes: 8_672_322,
        },
    ),
    (
        engine::CONFIG_FILE,
        Remote {
            url: pinned!("/config.json"),
            sha256: "391307b3bab6318ea6987071ccb82e1502aa83bc19f52a93428f7efb9466ecaa",
            bytes: 1_639,
        },
    ),
    (
        GENERATION_FILE,
        Remote {
            url: pinned!("/generation_config.json"),
            sha256: "d350fa7971cdf384def814c0f21ee35b89ae9ca8608f7b7a1203e3a6111d4f5c",
            bytes: 255,
        },
    ),
];

static IN_USE: AtomicUsize = AtomicUsize::new(0);

pub struct UseGuard;

impl Drop for UseGuard {
    fn drop(&mut self) {
        IN_USE.fetch_sub(1, Ordering::Relaxed);
    }
}

pub fn acquire() -> UseGuard {
    IN_USE.fetch_add(1, Ordering::Relaxed);
    UseGuard
}

pub struct Download {
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicU64>,
    total: u64,
}

#[derive(Clone, Serialize)]
pub struct Progress {
    done: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct Done {
    error: Option<String>,
}

#[derive(Serialize)]
pub struct Status {
    ready: bool,
    bytes: u64,
    downloading: Option<Progress>,
}

pub fn model_dir(app: &AppHandle) -> PathBuf {
    app.path().app_local_data_dir().unwrap_or_default().join(STORE_DIR).join(MODEL_DIR)
}

pub fn installed(app: &AppHandle) -> bool {
    model_dir(app).exists()
}

pub fn ready(app: &AppHandle) -> bool {
    let dir = model_dir(app);
    FILES.iter().all(|(file, _)| dir.join(file).is_file()) && crate::onnx::runtime_path(app).is_file()
}

fn pinned_bytes() -> u64 {
    FILES.iter().map(|(_, remote)| remote.bytes).sum()
}

fn fetch_model(app: &AppHandle, fetch: &mut Fetch) -> Result<(), String> {
    let target = model_dir(app);
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    fetch::clean(&target);
    crate::onnx::ensure(app, fetch)?;
    for (file, remote) in &FILES {
        fetch.file(remote, &target.join(file))?;
    }
    Ok(())
}

fn run(app: &AppHandle, cancel: &AtomicBool, done: &AtomicU64, total: u64) -> Result<(), String> {
    let mut last = Instant::now() - THROTTLE;
    let mut fetch = Fetch {
        agent: fetch::agent(),
        cancel,
        done,
        report: Box::new(|done| {
            if last.elapsed() >= THROTTLE {
                last = Instant::now();
                let _ = app.emit_to("main", events::TRANSLATE_PROGRESS, Progress { done, total });
            }
        }),
    };
    let result = fetch_model(app, &mut fetch);
    if result.is_err() {
        let _ = fs::remove_dir_all(model_dir(app));
    }
    result
}

#[tauri::command]
pub fn translate_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let downloading = state
        .translate_download
        .lock()
        .unwrap()
        .as_ref()
        .map(|d| Progress { done: d.done.load(Ordering::Relaxed), total: d.total });
    Status { ready: ready(&app), bytes: pinned_bytes(), downloading }
}

#[tauri::command]
pub fn translate_download(app: AppHandle, state: tauri::State<State>) -> Result<(), String> {
    if !state.cfg.read().unwrap().hymt_license_accepted {
        return Err(LICENSE_REQUIRED.into());
    }
    let mut slot = state.translate_download.lock().unwrap();
    if slot.is_some() {
        return Err("a download is already running".into());
    }
    if ready(&app) {
        return Ok(());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = pinned_bytes() + crate::onnx::missing_bytes(&app);
    *slot = Some(Download { cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    std::thread::spawn(move || {
        let result = run(&app, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().translate_download.lock().unwrap().take();
        let _ = app.emit_to("main", events::TRANSLATE_DONE, Done { error });
    });
    Ok(())
}

#[tauri::command]
pub fn translate_cancel(state: tauri::State<State>) {
    if let Some(download) = state.translate_download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn translate_delete(app: AppHandle, state: tauri::State<State>) -> Result<(), String> {
    let others_busy = state.download.lock().unwrap().is_some()
        || state.whisper_download.lock().unwrap().is_some()
        || state.cutout_download.lock().unwrap().is_some()
        || state.upscale_download.lock().unwrap().is_some()
        || state.captions_download.lock().unwrap().is_some();
    let download = state.translate_download.lock().unwrap();
    if download.is_some() {
        return Err("the model is downloading".into());
    }
    if IN_USE.load(Ordering::Relaxed) > 0 {
        return Err("the model is in use".into());
    }
    let removed = match fs::remove_dir_all(model_dir(&app)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    };
    let _ = fs::remove_dir(model_dir(&app).parent().unwrap_or(&model_dir(&app)));
    crate::onnx::release(&app, others_busy);
    removed
}
