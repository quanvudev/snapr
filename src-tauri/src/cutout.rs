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

const STORE_DIR: &str = "cutout";
const MODEL_DIR: &str = "ben2-base";
pub const MODEL_FILE: &str = "model.onnx";
const THROTTLE: Duration = Duration::from_millis(100);
const MODEL: Remote = Remote {
    url: "https://huggingface.co/PramaLLC/BEN2/resolve/e48a20765fb421d19dcdb0bf3cc61e802ca5ec8f/BEN2_Base.onnx",
    sha256: "22cea62108ff53b7ccc20f7a008bf30494228d84b1687f29ecbe76936a998101",
    bytes: 222_932_053,
};

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
    model_dir(app).join(MODEL_FILE).is_file() && crate::onnx::runtime_path(app).is_file()
}

fn fetch_model(app: &AppHandle, fetch: &mut Fetch) -> Result<(), String> {
    let target = model_dir(app);
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    fetch::clean(&target);
    crate::onnx::ensure(app, fetch)?;
    fetch.file(&MODEL, &target.join(MODEL_FILE))
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
                let _ = app.emit_to("main", events::CUTOUT_PROGRESS, Progress { done, total });
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
pub fn cutout_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let downloading = state
        .cutout_download
        .lock()
        .unwrap()
        .as_ref()
        .map(|d| Progress { done: d.done.load(Ordering::Relaxed), total: d.total });
    Status { ready: ready(&app), bytes: MODEL.bytes, downloading }
}

#[tauri::command]
pub fn cutout_download(app: AppHandle, state: tauri::State<State>) -> Result<(), String> {
    let mut slot = state.cutout_download.lock().unwrap();
    if slot.is_some() {
        return Err("a download is already running".into());
    }
    if ready(&app) {
        return Ok(());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = MODEL.bytes + crate::onnx::missing_bytes(&app);
    *slot = Some(Download { cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    std::thread::spawn(move || {
        let result = run(&app, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().cutout_download.lock().unwrap().take();
        let _ = app.emit_to("main", events::CUTOUT_DONE, Done { error });
    });
    Ok(())
}

#[tauri::command]
pub fn cutout_cancel(state: tauri::State<State>) {
    if let Some(download) = state.cutout_download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn cutout_delete(app: AppHandle, state: tauri::State<State>) -> Result<(), String> {
    let others_busy = state.download.lock().unwrap().is_some()
        || state.whisper_download.lock().unwrap().is_some()
        || state.translate_download.lock().unwrap().is_some()
        || state.upscale_download.lock().unwrap().is_some();
    let download = state.cutout_download.lock().unwrap();
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
