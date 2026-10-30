use crate::State;
use crate::settings::wide;
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use minisign_verify::{PublicKey, Signature};
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use windows_sys::Win32::Foundation::CloseHandle;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
    RegSetValueExW,
};
use windows_sys::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject};

const FIRST: Duration = Duration::from_secs(10);
const EVERY: Duration = Duration::from_secs(6 * 60 * 60);
const THROTTLE: Duration = Duration::from_millis(100);
const MENU_SLOT: usize = 5;
const RELEASES: &str = "https://github.com/quanvudev/snapr-releases/releases";
const MAX_EXE: u64 = 256 * 1024 * 1024;
const AFTER: &str = "--after";
const WAIT_MS: u32 = 10_000;
const UNINSTALL_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Snapr";

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
    found: Option<String>,
    announced: Option<String>,
    listed: bool,
}

#[derive(Deserialize)]
struct Feed {
    version: String,
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

fn found(app: &AppHandle, version: String) {
    let state = app.state::<State>();
    let vi = state.cfg.read().unwrap().language == "vi";
    let (fresh, listed) = {
        let mut u = state.update.lock().unwrap();
        let fresh = u.announced.as_deref() != Some(version.as_str());
        u.found = Some(version.clone());
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

fn newer(current: &semver::Version) -> Result<Option<String>, String> {
    let text = crate::paddle::agent()
        .get(format!("{RELEASES}/latest/download/latest.json"))
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let feed: Feed = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let version = feed.version.trim_start_matches('v').to_string();
    let parsed = semver::Version::parse(&version).map_err(|e| e.to_string())?;
    Ok((parsed > *current).then_some(version))
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
    let current = app.package_info().version.clone();
    let result = tauri::async_runtime::spawn_blocking(move || newer(&current))
        .await
        .unwrap_or_else(|e| Err(e.to_string()));
    match result {
        Ok(Some(version)) => found(app, version),
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

fn sibling(exe: &Path, suffix: &str) -> PathBuf {
    let mut name = exe.file_name().unwrap_or_default().to_os_string();
    name.push(suffix);
    exe.with_file_name(name)
}

fn signature(url: &str) -> Result<Signature, String> {
    let text = crate::paddle::agent()
        .get(url)
        .call()
        .map_err(|e| e.to_string())?
        .body_mut()
        .read_to_string()
        .map_err(|e| e.to_string())?;
    let decoded = STANDARD.decode(text.trim()).map_err(|e| e.to_string())?;
    Signature::decode(&String::from_utf8_lossy(&decoded)).map_err(|e| e.to_string())
}

fn verify(app: &AppHandle, bytes: &[u8], sig: &Signature) -> Result<(), String> {
    let pubkey = app.config().plugins.0.get("updater").and_then(|u| u["pubkey"].as_str()).ok_or("no update key")?;
    let decoded = STANDARD.decode(pubkey).map_err(|e| e.to_string())?;
    let key = PublicKey::decode(&String::from_utf8_lossy(&decoded)).map_err(|e| e.to_string())?;
    key.verify(bytes, sig, false).map_err(|_| "signature does not match".to_string())
}

fn download(app: &AppHandle, version: &str, to: &Path) -> Result<(), String> {
    let url = format!("{RELEASES}/download/v{version}/Snapr_{version}_x64.exe");
    let sig = signature(&format!("{url}.sig"))?;
    let mut response = crate::paddle::agent().get(&url).call().map_err(|e| e.to_string())?;
    let total = response.body().content_length().unwrap_or(0);
    let mut reader = response.body_mut().with_config().limit(MAX_EXE).reader();
    let mut bytes = Vec::with_capacity(total as usize);
    let mut buf = vec![0u8; 64 * 1024];
    let mut last = Instant::now();
    loop {
        let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        bytes.extend_from_slice(&buf[..n]);
        if last.elapsed() >= THROTTLE {
            last = Instant::now();
            let done = bytes.len() as u64;
            set(app, |s| {
                s.done = done;
                s.total = total;
            });
        }
    }
    verify(app, &bytes, &sig)?;
    let mut file = File::create(to).map_err(|e| e.to_string())?;
    file.write_all(&bytes).map_err(|e| e.to_string())?;
    file.flush().map_err(|e| e.to_string())
}

fn swap(exe: &Path, new: &Path) -> Result<(), String> {
    let old = sibling(exe, ".old");
    let _ = fs::remove_file(&old);
    fs::rename(exe, &old).map_err(|e| e.to_string())?;
    if let Err(e) = fs::rename(new, exe) {
        let _ = fs::rename(&old, exe);
        return Err(e.to_string());
    }
    if let Err(e) = std::process::Command::new(exe).arg(AFTER).arg(std::process::id().to_string()).spawn() {
        let _ = fs::rename(exe, new);
        let _ = fs::rename(&old, exe);
        return Err(e.to_string());
    }
    Ok(())
}

fn registry_string(key: HKEY, name: &str) -> Option<String> {
    let name = wide(name);
    let mut size = 0u32;
    if unsafe { RegQueryValueExW(key, name.as_ptr(), null(), null_mut(), null_mut(), &mut size) } != 0 {
        return None;
    }
    let mut buf = vec![0u16; size as usize / 2 + 1];
    if unsafe { RegQueryValueExW(key, name.as_ptr(), null(), null_mut(), buf.as_mut_ptr().cast(), &mut size) } != 0 {
        return None;
    }
    let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    Some(String::from_utf16_lossy(&buf[..len]))
}

fn display_version(exe: &Path, version: &str) {
    let Some(dir) = exe.parent() else { return };
    let mut key: HKEY = null_mut();
    let access = KEY_QUERY_VALUE | KEY_SET_VALUE;
    if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide(UNINSTALL_KEY).as_ptr(), 0, access, &mut key) } != 0 {
        return;
    }
    let installed = registry_string(key, "InstallLocation").map(|s| PathBuf::from(s.trim_matches('"')));
    if installed.is_some_and(|p| p.as_os_str().eq_ignore_ascii_case(dir.as_os_str())) {
        let data = wide(version);
        unsafe {
            RegSetValueExW(key, wide("DisplayVersion").as_ptr(), 0, REG_SZ, data.as_ptr().cast(), (data.len() * 2) as u32)
        };
    }
    unsafe { RegCloseKey(key) };
}

pub async fn install(app: &AppHandle) {
    let state = app.state::<State>();
    let version = {
        let u = state.update.lock().unwrap();
        if busy(u.status.state) {
            return;
        }
        u.found.clone()
    };
    let Some(version) = version else { return };
    if recording(app) {
        set(app, |s| s.state = Phase::Blocked);
        return;
    }
    set(app, |s| {
        *s = Status { state: Phase::Downloading, version: s.version.take(), ..Status::default() };
    });
    let handle = app.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let exe = std::env::current_exe().map_err(|e| e.to_string())?;
        let new = sibling(&exe, ".new");
        if let Err(e) = download(&handle, &version, &new) {
            let _ = fs::remove_file(&new);
            return Err(e);
        }
        if recording(&handle) {
            let _ = fs::remove_file(&new);
            return Ok(false);
        }
        set(&handle, |s| s.state = Phase::Installing);
        if let Err(e) = swap(&exe, &new) {
            let _ = fs::remove_file(&new);
            return Err(e);
        }
        display_version(&exe, &version);
        Ok(true)
    })
    .await
    .unwrap_or_else(|e| Err(e.to_string()));
    match result {
        Ok(true) => {
            app.cleanup_before_exit();
            std::process::exit(0);
        }
        Ok(false) => set(app, |s| s.state = Phase::Blocked),
        Err(e) => set(app, |s| {
            s.state = Phase::Error;
            s.error = Some(e);
        }),
    }
}

pub fn handoff() {
    let mut args = std::env::args().skip_while(|a| a != AFTER).skip(1);
    if let Some(pid) = args.next().and_then(|p| p.parse::<u32>().ok()) {
        let process = unsafe { OpenProcess(PROCESS_SYNCHRONIZE, 0, pid) };
        if !process.is_null() {
            unsafe {
                WaitForSingleObject(process, WAIT_MS);
                CloseHandle(process);
            }
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        let _ = fs::remove_file(sibling(&exe, ".old"));
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
