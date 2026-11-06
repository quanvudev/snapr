use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};
use tauri::{AppHandle, Manager};

const FILE: &str = "transcript-languages.json";
const PART: &str = "json.part";

static STORE: Mutex<Option<HashMap<String, String>>> = Mutex::new(None);

fn file(app: &AppHandle) -> Option<PathBuf> {
    app.path().app_local_data_dir().ok().map(|dir| dir.join(FILE))
}

pub fn key(video: &Path) -> String {
    video.to_string_lossy().to_lowercase()
}

fn load(app: &AppHandle) -> HashMap<String, String> {
    file(app)
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save(app: &AppHandle, map: &HashMap<String, String>) {
    let Some(path) = file(app) else { return };
    if let Some(dir) = path.parent() {
        let _ = fs::create_dir_all(dir);
    }
    let part = path.with_extension(PART);
    let Ok(text) = serde_json::to_string(map) else { return };
    if fs::write(&part, text).is_ok() && fs::rename(&part, &path).is_err() {
        let _ = fs::remove_file(&part);
    }
}

fn with<R>(app: &AppHandle, f: impl FnOnce(&mut HashMap<String, String>) -> R) -> R {
    let mut store = STORE.lock().unwrap_or_else(PoisonError::into_inner);
    f(store.get_or_insert_with(|| load(app)))
}

pub fn get(app: &AppHandle, video: &Path) -> Option<String> {
    with(app, |map| map.get(&key(video)).cloned())
}

pub fn all(app: &AppHandle) -> HashMap<String, String> {
    with(app, |map| map.clone())
}

pub fn set(app: &AppHandle, video: &Path, code: &str) {
    with(app, |map| {
        if map.insert(key(video), code.to_owned()).as_deref() != Some(code) {
            save(app, map);
        }
    });
}

pub fn rename(app: &AppHandle, old: &Path, new: &Path) {
    with(app, |map| {
        if let Some(code) = map.remove(&key(old)) {
            map.insert(key(new), code);
            save(app, map);
        }
    });
}

pub fn forget(app: &AppHandle, video: &Path) {
    with(app, |map| {
        if map.remove(&key(video)).is_some() {
            save(app, map);
        }
    });
}
