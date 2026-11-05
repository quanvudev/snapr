use crate::fetch::{self, Fetch, Remote};
use crate::paddle;
use ort::environment::Environment;
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::AppHandle;

const RUNTIME: Remote = Remote {
    url: "https://files.pythonhosted.org/packages/59/bf/a1d282b4c753ba3e9b31f4c11abd2bc2390513dc5e3792ca3b78e42401c3/onnxruntime-1.31.1-cp313-cp313-win_amd64.whl",
    sha256: "0558f29d5f6ac09957390c9c5495931485ee8ae7f0ce781ccee86a0ae5c2348b",
    bytes: 14_888_304,
};
const RUNTIME_ENTRY: &str = "onnxruntime/capi/onnxruntime.dll";
const RUNTIME_FILE: &str = "onnxruntime.dll";
const WHEEL_PART: &str = "onnxruntime.whl.part";
const ENVIRONMENT_NAME: &str = "snapr";
const NOT_INITIALISED: &str = "ONNX Runtime is not initialised";

static ENVIRONMENT: OnceLock<Result<Environment, String>> = OnceLock::new();

pub fn runtime_path(app: &AppHandle) -> PathBuf {
    paddle::dir(app).join(RUNTIME_FILE)
}

pub fn missing_bytes(app: &AppHandle) -> u64 {
    if runtime_path(app).is_file() { 0 } else { RUNTIME.bytes }
}

pub fn init(path: &Path) -> Result<&'static Environment, String> {
    ENVIRONMENT
        .get_or_init(|| {
            ort::init_from(path)
                .map_err(|e| e.to_string())?
                .with_name(ENVIRONMENT_NAME)
                .build()
                .map_err(|e| e.to_string())
        })
        .as_ref()
        .map_err(Clone::clone)
}

pub fn environment() -> Result<&'static Environment, String> {
    match ENVIRONMENT.get() {
        Some(result) => result.as_ref().map_err(Clone::clone),
        None => Err(NOT_INITIALISED.into()),
    }
}

pub fn ensure(app: &AppHandle, fetch: &mut Fetch) -> Result<(), String> {
    let dll = runtime_path(app);
    if dll.is_file() {
        return Ok(());
    }
    let root = dll.parent().unwrap_or(Path::new(""));
    fs::create_dir_all(root).map_err(|e| e.to_string())?;
    let wheel = root.join(WHEEL_PART);
    fetch.get(&RUNTIME, &wheel)?;
    let extracted = extract(&wheel, &dll);
    let _ = fs::remove_file(&wheel);
    extracted
}

pub fn release(app: &AppHandle, busy: bool) {
    if !busy && !paddle::installed(app) && !crate::whisper::installed(app) && !crate::translate::installed(app) {
        let _ = fs::remove_file(runtime_path(app));
    }
}

fn extract(wheel: &Path, to: &Path) -> Result<(), String> {
    let tmp = fetch::part(to);
    let result = (|| {
        let mut archive = zip::ZipArchive::new(File::open(wheel).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
        let mut entry = archive.by_name(RUNTIME_ENTRY).map_err(|e| e.to_string())?;
        let mut out = File::create(&tmp).map_err(|e| e.to_string())?;
        io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        out.flush().map_err(|e| e.to_string())?;
        drop(out);
        fs::rename(&tmp, to).map_err(|e| e.to_string())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
