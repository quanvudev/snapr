use crate::State;
use crate::config::{Config, OcrEngine, PaddleSize};
use crate::ocr::PaddleModel;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use yaml_rust2::{Yaml, YamlLoader};

struct Remote {
    url: &'static str,
    sha256: &'static str,
    bytes: u64,
}

struct Model {
    size: PaddleSize,
    det: Remote,
    rec: Remote,
    yml: Remote,
    params: (f32, f32, f32),
}

const RUNTIME: Remote = Remote {
    url: "https://files.pythonhosted.org/packages/59/bf/a1d282b4c753ba3e9b31f4c11abd2bc2390513dc5e3792ca3b78e42401c3/onnxruntime-1.31.1-cp313-cp313-win_amd64.whl",
    sha256: "0558f29d5f6ac09957390c9c5495931485ee8ae7f0ce781ccee86a0ae5c2348b",
    bytes: 14_888_304,
};
const RUNTIME_ENTRY: &str = "onnxruntime/capi/onnxruntime.dll";
const RUNTIME_FILE: &str = "onnxruntime.dll";
const WHEEL_PART: &str = "onnxruntime.whl.part";

const MODELS: [Model; 3] = [
    Model {
        size: PaddleSize::Tiny,
        det: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_tiny_det_onnx/resolve/2ba1506c0380b8f0b03dd142459aac66d4421f6c/inference.onnx",
            sha256: "193bab7a04fca699a6c82e6abb5b81bdb28177f0abd4062552b04908dafb19f8",
            bytes: 1_780_590,
        },
        rec: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_tiny_rec_onnx/resolve/2612ab37152ae0a677521bae4e1e3d4fb4cf7c30/inference.onnx",
            sha256: "9ef676d6ed3c88256a2d92c640c44f25b0c40947e111b14b8be8f594091563e6",
            bytes: 4_462_639,
        },
        yml: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_tiny_rec_onnx/resolve/2612ab37152ae0a677521bae4e1e3d4fb4cf7c30/inference.yml",
            sha256: "66170210bad538e83fff3c4a3867e547d6bf20b50d64b20347c4b913f3034ea1",
            bytes: 55_571,
        },
        params: (0.2, 0.4, 1.4),
    },
    Model {
        size: PaddleSize::Small,
        det: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_det_onnx/resolve/28fe5895c24fd108c19eb3e8479f4ab385fbfc62/inference.onnx",
            sha256: "d73e0058b7a8086bbd57f3d10b8bcd4ff95363f67e06e2762b5e814fe9c9410e",
            bytes: 9_880_512,
        },
        rec: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/b8f84f0b80c529de40b4fbb3544b84fa7233a513/inference.onnx",
            sha256: "5435fd747c9e0efe15a96d0b378d5bd157e9492ed8fd80edf08f30d02fa24634",
            bytes: 21_159_378,
        },
        yml: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_small_rec_onnx/resolve/b8f84f0b80c529de40b4fbb3544b84fa7233a513/inference.yml",
            sha256: "ab078671bb49f06228eadccd34f1bb501e157f7a047095ffb943ba81512c77d1",
            bytes: 150_579,
        },
        params: (0.2, 0.45, 1.4),
    },
    Model {
        size: PaddleSize::Medium,
        det: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_medium_det_onnx/resolve/61323801669c338b7891481ec7bac61ce31b576a/inference.onnx",
            sha256: "eb13b44b25bb36f89528b68720af8a61d9cf381176107f465db1757b65d086e1",
            bytes: 62_032_837,
        },
        rec: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_medium_rec_onnx/resolve/50c7eacafc52fa7bcf4194e8cd08e46f8558504b/inference.onnx",
            sha256: "9c09abf0957f7968c7586464b7397b84ad2387a0497a351af40e9acc71b673ba",
            bytes: 76_554_979,
        },
        yml: Remote {
            url: "https://huggingface.co/PaddlePaddle/PP-OCRv6_medium_rec_onnx/resolve/50c7eacafc52fa7bcf4194e8cd08e46f8558504b/inference.yml",
            sha256: "991b700facf5b50a7de193468207d5f4255b538dde0d312ae3b7c7a9b6873129",
            bytes: 150_580,
        },
        params: (0.2, 0.45, 1.4),
    },
];

const SIZES: [PaddleSize; 3] = [PaddleSize::Tiny, PaddleSize::Small, PaddleSize::Medium];
const THROTTLE: Duration = Duration::from_millis(100);

pub struct Download {
    size: PaddleSize,
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicU64>,
    total: u64,
}

#[derive(Clone, Serialize)]
pub struct Progress {
    size: PaddleSize,
    done: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct Done {
    size: PaddleSize,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct Status {
    ready: Vec<PaddleSize>,
    bytes: HashMap<PaddleSize, u64>,
    downloading: Option<Progress>,
    runtime: bool,
}

fn model(size: PaddleSize) -> &'static Model {
    MODELS.iter().find(|m| m.size == size).unwrap()
}

fn name(size: PaddleSize) -> &'static str {
    match size {
        PaddleSize::Tiny => "tiny",
        PaddleSize::Small => "small",
        PaddleSize::Medium => "medium",
    }
}

pub fn dir(app: &AppHandle) -> PathBuf {
    app.path().app_local_data_dir().unwrap_or_default().join("paddle")
}

fn size_dir(app: &AppHandle, size: PaddleSize) -> PathBuf {
    dir(app).join(name(size))
}

pub fn files(app: &AppHandle, size: PaddleSize) -> (PathBuf, PathBuf, PathBuf) {
    let root = size_dir(app, size);
    (root.join("det.onnx"), root.join("rec.onnx"), root.join("dict.txt"))
}

pub fn runtime(app: &AppHandle) -> PathBuf {
    dir(app).join(RUNTIME_FILE)
}

pub fn ready(app: &AppHandle, size: PaddleSize) -> bool {
    let (det, rec, dict) = files(app, size);
    det.is_file() && rec.is_file() && dict.is_file() && runtime(app).is_file()
}

pub fn det_params(size: PaddleSize) -> (f32, f32, f32) {
    model(size).params
}

pub fn selected(app: &AppHandle, cfg: &Config) -> Option<PaddleModel> {
    let size = cfg.paddle_model;
    if cfg.ocr_engine != OcrEngine::Paddle || !ready(app, size) {
        return None;
    }
    let (det, rec, dict) = files(app, size);
    Some(PaddleModel { size, det, rec, dict, runtime: runtime(app), params: det_params(size) })
}

pub fn unload_others(cfg: &Config) {
    let keep = (cfg.ocr_engine == OcrEngine::Paddle).then_some(cfg.paddle_model);
    for size in SIZES {
        if Some(size) != keep {
            crate::ocr::paddle_unload(Some(size));
        }
    }
}

fn disk_bytes(path: &Path) -> u64 {
    fs::read_dir(path)
        .map(|entries| entries.flatten().filter_map(|e| e.metadata().ok()).filter(|m| m.is_file()).map(|m| m.len()).sum())
        .unwrap_or(0)
}

fn clean(path: &Path) {
    let Ok(entries) = fs::read_dir(path) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "part") {
            let _ = fs::remove_file(path);
        }
    }
}

fn part(path: &Path) -> PathBuf {
    let mut name = path.file_name().unwrap_or_default().to_os_string();
    name.push(".part");
    path.with_file_name(name)
}

struct Fetch<'a> {
    agent: ureq::Agent,
    cancel: &'a AtomicBool,
    done: &'a AtomicU64,
    report: Box<dyn FnMut(u64) + 'a>,
}

impl Fetch<'_> {
    fn get(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let result = self.stream(remote, to);
        if result.is_err() {
            let _ = fs::remove_file(to);
        }
        result
    }

    fn stream(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let response = self.agent.get(remote.url).call().map_err(|e| e.to_string())?;
        let mut reader = response.into_body().into_with_config().limit(remote.bytes + 1).reader();
        let mut file = File::create(to).map_err(|e| e.to_string())?;
        let mut hasher = Sha256::new();
        let mut buf = vec![0u8; 64 * 1024];
        let mut received = 0u64;
        loop {
            if self.cancel.load(Ordering::Relaxed) {
                return Err("cancelled".into());
            }
            let n = reader.read(&mut buf).map_err(|e| e.to_string())?;
            if n == 0 {
                break;
            }
            hasher.update(&buf[..n]);
            file.write_all(&buf[..n]).map_err(|e| e.to_string())?;
            received += n as u64;
            let done = self.done.fetch_add(n as u64, Ordering::Relaxed) + n as u64;
            (self.report)(done);
        }
        file.flush().map_err(|e| e.to_string())?;
        drop(file);
        let digest: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
        if received != remote.bytes || digest != remote.sha256 {
            return Err(format!("checksum mismatch: {}", remote.url.rsplit('/').next().unwrap_or_default()));
        }
        Ok(())
    }

    fn file(&mut self, remote: &Remote, to: &Path) -> Result<(), String> {
        let tmp = part(to);
        self.get(remote, &tmp)?;
        fs::rename(&tmp, to).map_err(|e| {
            let _ = fs::remove_file(&tmp);
            e.to_string()
        })
    }
}

fn extract_runtime(wheel: &Path, to: &Path) -> Result<(), String> {
    let tmp = part(to);
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

fn dictionary(yml: &Path) -> Result<String, String> {
    let text = fs::read_to_string(yml).map_err(|e| e.to_string())?;
    let docs = YamlLoader::load_from_str(&text).map_err(|e| e.to_string())?;
    let items = docs
        .first()
        .and_then(|doc| doc["PostProcess"]["character_dict"].as_vec())
        .ok_or("character_dict missing")?;
    let mut out = String::new();
    for item in items {
        let entry = match item {
            Yaml::String(s) | Yaml::Real(s) => s.clone(),
            Yaml::Integer(i) => i.to_string(),
            Yaml::Boolean(b) => b.to_string(),
            _ => return Err("character_dict has an invalid entry".into()),
        };
        if entry.contains(['\n', '\r']) {
            return Err("character_dict has an invalid entry".into());
        }
        out.push_str(&entry);
        out.push('\n');
    }
    Ok(out)
}

fn write_dict(yml: &Path, to: &Path) -> Result<(), String> {
    let tmp = part(to);
    let result = dictionary(yml).and_then(|text| {
        fs::write(&tmp, text).map_err(|e| e.to_string())?;
        fs::rename(&tmp, to).map_err(|e| e.to_string())
    });
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn total(root: &Path, size: PaddleSize) -> u64 {
    let m = model(size);
    let runtime = if root.join(RUNTIME_FILE).is_file() { 0 } else { RUNTIME.bytes };
    m.det.bytes + m.rec.bytes + m.yml.bytes + runtime
}

fn fetch_size(root: &Path, size: PaddleSize, fetch: &mut Fetch) -> Result<(), String> {
    let target = root.join(name(size));
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    clean(root);
    clean(&target);
    let dll = root.join(RUNTIME_FILE);
    if !dll.is_file() {
        let wheel = root.join(WHEEL_PART);
        fetch.get(&RUNTIME, &wheel)?;
        let extracted = extract_runtime(&wheel, &dll);
        let _ = fs::remove_file(&wheel);
        extracted?;
    }
    let m = model(size);
    fetch.file(&m.det, &target.join("det.onnx"))?;
    fetch.file(&m.rec, &target.join("rec.onnx"))?;
    let yml = target.join("rec.yml.part");
    fetch.get(&m.yml, &yml)?;
    let dict = write_dict(&yml, &target.join("dict.txt"));
    let _ = fs::remove_file(&yml);
    dict
}

pub fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_connect(Some(Duration::from_secs(30)))
        .timeout_per_read(Some(Duration::from_secs(30)))
        .build()
        .into()
}

fn run(app: &AppHandle, size: PaddleSize, cancel: &AtomicBool, done: &AtomicU64, total: u64) -> Result<(), String> {
    let root = dir(app);
    let mut last = Instant::now() - THROTTLE;
    let mut fetch = Fetch {
        agent: agent(),
        cancel,
        done,
        report: Box::new(|done| {
            if last.elapsed() >= THROTTLE {
                last = Instant::now();
                let _ = app.emit_to("main", "paddle-progress", Progress { size, done, total });
            }
        }),
    };
    let result = fetch_size(&root, size, &mut fetch);
    if result.is_err() {
        let _ = fs::remove_dir_all(root.join(name(size)));
    }
    result
}

#[tauri::command]
pub fn paddle_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let ready: Vec<PaddleSize> = SIZES.into_iter().filter(|&s| self::ready(&app, s)).collect();
    let bytes = ready.iter().map(|&s| (s, disk_bytes(&size_dir(&app, s)))).collect();
    let downloading = state.download.lock().unwrap().as_ref().map(|d| Progress {
        size: d.size,
        done: d.done.load(Ordering::Relaxed),
        total: d.total,
    });
    Status { ready, bytes, downloading, runtime: runtime(&app).is_file() }
}

#[tauri::command]
pub fn paddle_download(app: AppHandle, state: tauri::State<State>, size: PaddleSize) -> Result<(), String> {
    let mut slot = state.download.lock().unwrap();
    if slot.is_some() {
        return Err("a download is already running".into());
    }
    if ready(&app, size) {
        return Ok(());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = total(&dir(&app), size);
    *slot = Some(Download { size, cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    std::thread::spawn(move || {
        let result = run(&app, size, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().download.lock().unwrap().take();
        let handle = app.clone();
        let _ = app.run_on_main_thread(move || {
            crate::apply(&handle);
            let _ = handle.emit_to("main", "paddle-done", Done { size, error });
        });
    });
    Ok(())
}

#[tauri::command]
pub fn paddle_cancel(state: tauri::State<State>) {
    if let Some(download) = state.download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn paddle_delete(app: AppHandle, state: tauri::State<State>, size: PaddleSize) -> Result<(), String> {
    let download = state.download.lock().unwrap();
    if download.as_ref().is_some_and(|d| d.size == size) {
        return Err("this size is downloading".into());
    }
    crate::ocr::paddle_unload(Some(size));
    let target = size_dir(&app, size);
    let removed = match fs::remove_dir_all(&target) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    };
    if download.is_none() && SIZES.into_iter().all(|s| !size_dir(&app, s).exists()) {
        let _ = fs::remove_file(runtime(&app));
    }
    drop(download);
    crate::apply(&app);
    removed
}
