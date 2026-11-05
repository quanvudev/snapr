use crate::hotkeys::{apply, ocr_available};
use crate::{State, events};
use crate::config::{Config, Hotkeys, RING_SLOT, SLOTS, is_mouse, parse_hotkey, skipped};
use serde::Serialize;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::ptr::{null, null_mut};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, LogicalSize, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::Com::{COINIT_APARTMENTTHREADED, CoInitializeEx, CoTaskMemFree, CoUninitialize, IBindCtx};
use windows::Win32::UI::Shell::{
    BHID_EnumItems, FOLDERID_AppsFolder, IEnumShellItems, IShellItem, KF_FLAG_DEFAULT, SHGetKnownFolderItem, SIGDN,
    SIGDN_NORMALDISPLAY, SIGDN_PARENTRELATIVEPARSING,
};
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "snapr";
const RING_ICON_SIZE: i32 = 32;
const APPS_FOLDER_PREFIX: &str = "shell:AppsFolder\\";

#[derive(Serialize)]
pub struct Loaded {
    config: Config,
    autostart: bool,
    dir: String,
    first_run: bool,
    ocr_available: bool,
    windows_ocr: bool,
}

#[derive(Serialize)]
pub struct Row {
    combo: String,
    error: Option<&'static str>,
    in_use: bool,
}

pub fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

fn run_key(access: u32) -> Option<HKEY> {
    let mut key: HKEY = null_mut();
    let status = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, wide(RUN_KEY).as_ptr(), 0, access, &mut key) };
    (status == 0).then_some(key)
}

fn autostart_get() -> bool {
    let Some(key) = run_key(KEY_READ) else { return false };
    let found = unsafe { RegQueryValueExW(key, wide(RUN_VALUE).as_ptr(), null(), null_mut(), null_mut(), null_mut()) } == 0;
    unsafe { RegCloseKey(key) };
    found
}

fn scoop_current(exe: &Path) -> Option<PathBuf> {
    let version = exe.parent()?;
    let app = version.parent()?;
    let current = app.join("current");
    let apps = app.parent()?.file_name()?.eq_ignore_ascii_case("apps");
    (apps && version.file_name()? != "current" && current.is_dir()).then(|| current.join(exe.file_name().unwrap_or_default()))
}

fn autostart_set(on: bool) {
    let Some(key) = run_key(KEY_SET_VALUE) else { return };
    let name = wide(RUN_VALUE);
    unsafe {
        if on {
            if let Ok(exe) = std::env::current_exe() {
                let data = wide(&format!("\"{}\"", scoop_current(&exe).unwrap_or(exe).display()));
                RegSetValueExW(key, name.as_ptr(), 0, REG_SZ, data.as_ptr().cast(), (data.len() * 2) as u32);
            }
        } else {
            RegDeleteValueW(key, name.as_ptr());
        }
        RegCloseKey(key);
    }
}

fn is_f_key(name: &str) -> bool {
    name.len() > 1 && name.starts_with('F') && name[1..].bytes().all(|b| b.is_ascii_digit())
}

fn in_use(app: &AppHandle, combo: &str) -> bool {
    let Ok(key) = parse_hotkey(combo) else { return false };
    let shortcuts = app.global_shortcut();
    if shortcuts.is_registered(key) {
        return false;
    }
    let state = app.state::<State>();
    let mut cache = state.probe.lock().unwrap();
    *cache.entry(combo.to_string()).or_insert_with(|| {
        let free = shortcuts.register(key).is_ok();
        if free {
            let _ = shortcuts.unregister(key);
        }
        !free
    })
}

fn exempt(i: usize, combos: &[String; SLOTS], text: bool, ring: bool) -> bool {
    skipped(i, combos, text, ring) || (i == RING_SLOT && is_mouse(&combos[i]))
}

fn rows(app: &AppHandle, hotkeys: &Hotkeys, text: bool, ring: bool) -> [Row; SLOTS] {
    let combos = hotkeys.combos();
    let mut errors = [None; SLOTS];
    let mut ids = [None; SLOTS];
    for i in 0..SLOTS {
        if exempt(i, &combos, text, ring) {
            continue;
        }
        match parse_hotkey(&combos[i]) {
            Err(_) => errors[i] = Some("invalid"),
            Ok(_) if !combos[i].contains('+') && !is_f_key(&combos[i]) => errors[i] = Some("needs_modifier"),
            Ok(key) => ids[i] = Some(key.id()),
        }
    }
    for i in 0..SLOTS {
        if ids[i].is_some() && (0..SLOTS).any(|j| j != i && ids[j] == ids[i]) {
            errors[i] = Some("duplicate");
        }
    }
    let skips: [bool; SLOTS] = std::array::from_fn(|i| exempt(i, &combos, text, ring));
    let mut i = 0;
    combos.map(|combo| {
        let error = errors[i];
        let in_use = error.is_none() && !skips[i] && in_use(app, &combo);
        i += 1;
        Row { combo, error, in_use }
    })
}

fn dir_of(output_dir: String) -> String {
    Config { output_dir, ..Config::default() }.dir().display().to_string()
}

#[tauri::command]
pub fn load(state: tauri::State<State>) -> Loaded {
    let config = state.cfg.read().unwrap().clone();
    let dir = config.dir().display().to_string();
    Loaded { config, autostart: autostart_get(), dir, first_run: !state.path.exists(), ocr_available: state.ocr.load(Ordering::Relaxed), windows_ocr: crate::ocr::windows_available() }
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    show_settings(&app, "settings");
}

#[tauri::command]
pub fn defaults() -> Config {
    Config::default()
}

#[tauri::command]
pub fn resolve_dir(output_dir: String) -> String {
    dir_of(output_dir)
}

#[tauri::command]
pub fn check(app: AppHandle, hotkeys: Hotkeys, text: bool) -> [Row; SLOTS] {
    let ring = app.state::<State>().cfg.read().unwrap().ring.enabled;
    rows(&app, &hotkeys, text, ring)
}

#[tauri::command]
pub fn set_listening(app: AppHandle, on: bool) {
    if on {
        let _ = app.global_shortcut().unregister_all();
        crate::mouse::bind(None);
    } else {
        apply(&app);
    }
}

#[tauri::command]
pub fn save(app: AppHandle, mut config: Config, autostart: bool) -> Result<(), String> {
    config.ring.sanitise();
    let state = app.state::<State>();
    let text = config.text_capture && state.ocr.load(Ordering::Relaxed);
    if rows(&app, &config.hotkeys, text, config.ring.enabled).iter().any(|r| r.error.is_some()) {
        return Err("invalid hotkeys".into());
    }
    config.save(&state.path).map_err(|e| e.to_string())?;
    if autostart != autostart_get() {
        autostart_set(autostart);
    }
    let _ = std::fs::create_dir_all(config.dir());
    let enabled = config.update_check && !state.cfg.read().unwrap().update_check;
    *state.cfg.write().unwrap() = config;
    state.probe.lock().unwrap().clear();
    apply(&app);
    if enabled {
        crate::update::enabled(&app);
    }
    Ok(())
}

#[tauri::command]
pub fn ring_icon(path: String) -> Result<tauri::ipc::Response, String> {
    let path = std::path::PathBuf::from(path);
    let is_exe = path.is_file() && path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("exe"));
    let is_store_app = path.to_str().is_some_and(|text| {
        text.get(..APPS_FOLDER_PREFIX.len()).is_some_and(|head| head.eq_ignore_ascii_case(APPS_FOLDER_PREFIX))
    });
    if !is_exe && !is_store_app {
        return Err("not an executable".into());
    }
    crate::gallery::icon_png(&path, RING_ICON_SIZE).map(tauri::ipc::Response::new).ok_or_else(|| "no icon".to_string())
}

#[derive(Serialize)]
pub struct InstalledApp {
    name: String,
    path: String,
}

fn display_name(item: &IShellItem, kind: SIGDN) -> Option<String> {
    let raw = unsafe { item.GetDisplayName(kind) }.ok()?;
    let text = unsafe { raw.to_string() }.ok();
    unsafe { CoTaskMemFree(Some(raw.0 as _)) };
    text
}

fn enumerate_apps() -> windows::core::Result<Vec<InstalledApp>> {
    let folder: IShellItem = unsafe { SHGetKnownFolderItem(&FOLDERID_AppsFolder, KF_FLAG_DEFAULT, None::<HANDLE>) }?;
    let items: IEnumShellItems = unsafe { folder.BindToHandler(None::<&IBindCtx>, &BHID_EnumItems) }?;
    let mut seen = HashSet::new();
    let mut apps = Vec::new();
    let mut batch = [None];
    let mut fetched = 0;
    while unsafe { items.Next(&mut batch, Some(&mut fetched)) }.is_ok() && fetched == 1 {
        let Some(item) = batch[0].take() else { break };
        let name = display_name(&item, SIGDN_NORMALDISPLAY).filter(|name| !name.is_empty());
        let id = display_name(&item, SIGDN_PARENTRELATIVEPARSING).filter(|id| !id.is_empty());
        if let (Some(name), Some(id)) = (name, id) {
            let path = format!("{APPS_FOLDER_PREFIX}{id}");
            if seen.insert(path.clone()) {
                apps.push(InstalledApp { name, path });
            }
        }
    }
    apps.sort_by_key(|app| app.name.to_lowercase());
    Ok(apps)
}

fn installed_apps() -> Result<Vec<InstalledApp>, String> {
    let init = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
    let apps = enumerate_apps().map_err(|e| e.to_string());
    if init.is_ok() {
        unsafe { CoUninitialize() };
    }
    apps
}

#[tauri::command]
pub async fn list_apps() -> Result<Vec<InstalledApp>, String> {
    tauri::async_runtime::spawn_blocking(installed_apps).await.map_err(|e| e.to_string())?
}

#[tauri::command]
pub fn open_folder(output_dir: String) {
    let _ = std::process::Command::new("explorer").arg(dir_of(output_dir)).spawn();
}

#[tauri::command]
pub fn open_config_file(state: tauri::State<State>) {
    let _ = std::process::Command::new("notepad").arg(&state.path).spawn();
}

#[tauri::command]
pub fn open_language_settings() {
    let _ = std::process::Command::new("explorer").arg("ms-settings:regionlanguage").spawn();
}

pub(crate) fn show_splash(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_size(LogicalSize::new(480.0, 300.0));
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub(crate) fn show_settings(app: &AppHandle, section: &str) {
    if ocr_available(app) != app.state::<State>().ocr.load(Ordering::Relaxed) {
        apply(app);
    }
    let Some(w) = app.get_webview_window("main") else { return };
    let (max_w, max_h) = match w.current_monitor() {
        Ok(Some(m)) => {
            let area = m.work_area().size.to_logical::<f64>(m.scale_factor());
            (area.width, area.height)
        }
        _ => (1366.0, 768.0),
    };
    let _ = w.set_size(LogicalSize::new(max_w.min(1366.0), max_h.min(768.0)));
    let _ = w.center();
    let _ = w.show();
    let _ = w.set_focus();
    let _ = app.emit_to("main", events::SETTINGS_OPEN, section);
}
