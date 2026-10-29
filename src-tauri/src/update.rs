use crate::State;
use serde::Serialize;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_updater::{Update, UpdaterExt};

const FIRST: Duration = Duration::from_secs(10);
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);
const THROTTLE: Duration = Duration::from_millis(100);
const MENU_SLOT: usize = 4;

#[derive(Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    #[default]
    Idle,
    Checking,
    Latest,
    Available,
    Blocked,
    Downloading,
    Installing,
    Error,
}

#[derive(Clone, Default, Serialize)]
pub struct Status {
    state: Phase,
    version: Option<String>,
    done: u64,
    total: u64,
    error: Option<String>,
}

#[derive(Default)]
pub struct Updater {
    status: Status,
    found: Option<Update>,
    announced: Option<String>,
    listed: bool,
}

fn busy(phase: Phase) -> bool {
    matches!(phase, Phase::Checking | Phase::Downloading | Phase::Installing)
}

fn set(app: &AppHandle, f: impl FnOnce(&mut Status)) {
    let state = app.state::<State>();
    let status = {
        let mut u = state.update.lock().unwrap();
        f(&mut u.status);
        u.status.clone()
    };
    let _ = app.emit_to("main", "update-status", status);
}

fn recording(app: &AppHandle) -> bool {
    let state = app.state::<State>();
    state.recording.lock().unwrap().is_some() || state.counting.load(Ordering::Relaxed)
}

fn unlist(app: &AppHandle) {
    let state = app.state::<State>();
    if std::mem::take(&mut state.update.lock().unwrap().listed) {
        let _ = state.menu.remove(&state.upgrade);
    }
}

fn found(app: &AppHandle, update: Update) {
    let state = app.state::<State>();
    let vi = state.cfg.read().unwrap().language == "vi";
    let version = update.version.clone();
    let (fresh, listed) = {
        let mut u = state.update.lock().unwrap();
        let fresh = u.announced.as_deref() != Some(version.as_str());
        u.found = Some(update);
        u.announced = Some(version.clone());
        u.status = Status { state: Phase::Available, version: Some(version.clone()), ..Status::default() };
        (fresh, std::mem::replace(&mut u.listed, true))
    };
    let label = if vi { format!("Cập nhật lên {version}") } else { format!("Update to {version}") };
    let _ = state.upgrade.set_text(label);
    if !listed {
        let _ = state.menu.insert(&state.upgrade, MENU_SLOT);
    }
    if fresh {
        let body = if vi { format!("Đã có Snapr {version}") } else { format!("Snapr {version} is available") };
        let _ = app.notification().builder().title("Snapr").body(body).show();
    }
    let status = state.update.lock().unwrap().status.clone();
    let _ = app.emit_to("main", "update-status", status);
}

pub async fn check(app: &AppHandle, manual: bool) {
    let state = app.state::<State>();
    let latest = {
        let u = state.update.lock().unwrap();
        if busy(u.status.state) {
            return;
        }
        u.status.state == Phase::Latest
    };
    if manual {
        set(app, |s| {
            s.state = Phase::Checking;
            s.error = None;
        });
    }
    let handle = app.clone();
    let result = match app.updater_builder().on_before_exit(move || handle.cleanup_before_exit()).build() {
        Ok(updater) => updater.check().await.map_err(|e| e.to_string()),
        Err(e) => Err(e.to_string()),
    };
    match result {
        Ok(Some(update)) => found(app, update),
        Ok(None) => {
            state.update.lock().unwrap().found = None;
            unlist(app);
            set(app, |s| *s = Status { state: if manual || latest { Phase::Latest } else { Phase::Idle }, ..Status::default() });
        }
        Err(e) if manual => set(app, |s| {
            s.state = Phase::Error;
            s.error = Some(e);
        }),
        Err(_) => {}
    }
}

pub async fn install(app: &AppHandle) {
    let state = app.state::<State>();
    let update = {
        let u = state.update.lock().unwrap();
        if busy(u.status.state) {
            return;
        }
        u.found.clone()
    };
    let Some(update) = update else { return };
    if recording(app) {
        set(app, |s| s.state = Phase::Blocked);
        return;
    }
    set(app, |s| {
        *s = Status { state: Phase::Downloading, version: s.version.take(), ..Status::default() };
    });
    let mut done = 0u64;
    let mut last = Instant::now();
    let bytes = update
        .download(
            |chunk, total| {
                done += chunk as u64;
                if last.elapsed() >= THROTTLE {
                    last = Instant::now();
                    set(app, |s| {
                        s.done = done;
                        s.total = total.unwrap_or(0);
                    });
                }
            },
            || {},
        )
        .await;
    let result = match bytes {
        Ok(_) if recording(app) => {
            set(app, |s| s.state = Phase::Blocked);
            return;
        }
        Ok(bytes) => {
            set(app, |s| s.state = Phase::Installing);
            update.install(bytes).map_err(|e| e.to_string())
        }
        Err(e) => Err(e.to_string()),
    };
    if let Err(e) = result {
        set(app, |s| {
            s.state = Phase::Error;
            s.error = Some(e);
        });
    }
}

pub fn start(app: &AppHandle) {
    if cfg!(debug_assertions) {
        return;
    }
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(FIRST);
        loop {
            if app.state::<State>().cfg.read().unwrap().update_check {
                tauri::async_runtime::block_on(check(&app, false));
            }
            std::thread::sleep(EVERY);
        }
    });
}

pub fn enabled(app: &AppHandle) {
    let app = app.clone();
    tauri::async_runtime::spawn(async move { check(&app, false).await });
}

pub fn from_tray(app: &AppHandle) {
    crate::show_settings(app, "about");
    let app = app.clone();
    tauri::async_runtime::spawn(async move { install(&app).await });
}

#[tauri::command]
pub fn update_status(app: AppHandle) -> Status {
    app.state::<State>().update.lock().unwrap().status.clone()
}

#[tauri::command]
pub async fn update_check(app: AppHandle) {
    check(&app, true).await;
}

#[tauri::command]
pub async fn update_install(app: AppHandle) {
    install(&app).await;
}
