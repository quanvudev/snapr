use crate::State;
use crate::config::{Config, Hotkeys, RING_SLOT, SLOTS, is_mouse, parse_hotkey, skipped};
use serde::Serialize;
use std::ptr::{null, null_mut};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use windows_sys::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegQueryValueExW, RegSetValueExW,
};

const RUN_KEY: &str = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
const RUN_VALUE: &str = "snapr";

#[derive(Serialize)]
pub struct Loaded {
    config: Config,
    autostart: bool,
    dir: String,
    first_run: bool,
    ocr_available: bool,
}

#[derive(Serialize)]
pub struct Row {
    combo: String,
    error: Option<&'static str>,
    in_use: bool,
}

fn wide(s: &str) -> Vec<u16> {
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

fn autostart_set(on: bool) {
    let Some(key) = run_key(KEY_SET_VALUE) else { return };
    let name = wide(RUN_VALUE);
    unsafe {
        if on {
            if let Ok(exe) = std::env::current_exe() {
                let data = wide(&format!("\"{}\"", exe.display()));
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

fn exempt(i: usize, combos: &[String; SLOTS], text: bool) -> bool {
    skipped(i, combos, text) || (i == RING_SLOT && is_mouse(&combos[i]))
}

fn rows(app: &AppHandle, hotkeys: &Hotkeys, text: bool) -> [Row; SLOTS] {
    let combos = hotkeys.combos();
    let mut errors = [None; SLOTS];
    let mut ids = [None; SLOTS];
    for i in 0..SLOTS {
        if exempt(i, &combos, text) {
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
    let skips: [bool; SLOTS] = std::array::from_fn(|i| exempt(i, &combos, text));
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
    Loaded { config, autostart: autostart_get(), dir, first_run: !state.path.exists(), ocr_available: state.ocr.load(Ordering::Relaxed) }
}

#[tauri::command]
pub fn open_settings(app: AppHandle) {
    crate::show_settings(&app, "settings");
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
    rows(&app, &hotkeys, text)
}

#[tauri::command]
pub fn set_listening(app: AppHandle, on: bool) {
    if on {
        let _ = app.global_shortcut().unregister_all();
        crate::mouse::bind(None);
    } else {
        crate::apply(&app);
    }
}

#[tauri::command]
pub fn save(app: AppHandle, config: Config, autostart: bool) -> Result<(), String> {
    let state = app.state::<State>();
    let text = config.text_capture && state.ocr.load(Ordering::Relaxed);
    if rows(&app, &config.hotkeys, text).iter().any(|r| r.error.is_some()) {
        return Err("invalid hotkeys".into());
    }
    config.save(&state.path).map_err(|e| e.to_string())?;
    if autostart != autostart_get() {
        autostart_set(autostart);
    }
    let _ = std::fs::create_dir_all(config.dir());
    *state.cfg.write().unwrap() = config;
    state.probe.lock().unwrap().clear();
    crate::apply(&app);
    Ok(())
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
