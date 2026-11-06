use crate::events;
use crate::State;
use crate::config::WhisperModel;
use crate::fetch::{self, Fetch, Remote};
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

const ENCODER_FILE: &str = "encoder.onnx";
const DECODER_FILE: &str = "decoder.onnx";
const GPU_ENCODER_FILE: &str = "encoder.gpu.onnx";
const GPU_DECODER_FILE: &str = "decoder.gpu.onnx";
pub const CPU_FILES: [&str; 2] = [ENCODER_FILE, DECODER_FILE];
pub const GPU_FILES: [&str; 2] = [GPU_ENCODER_FILE, GPU_DECODER_FILE];
const VOCAB_FILE: &str = "vocab.json";
const CONFIG_FILE: &str = "config.json";
const GENERATION_FILE: &str = "generation_config.json";
const PREPROCESSOR_FILE: &str = "preprocessor_config.json";
const FILE_COUNT: usize = 6;
const MODEL_COUNT: usize = 3;
const THROTTLE: Duration = Duration::from_millis(100);

struct Model {
    id: WhisperModel,
    files: [(&'static str, Remote); FILE_COUNT],
    gpu: [Remote; 2],
}

const MODELS: [Model; MODEL_COUNT] = [
    Model {
        id: WhisperModel::Base,
        files: [
            (
                ENCODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/onnx/encoder_model_int8.onnx",
                    sha256: "ca6177401f86a2c6b4dc5f7fc02fbca680678906bd0c22f6d89f0b80f124253f",
                    bytes: 23_201_297,
                },
            ),
            (
                DECODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/onnx/decoder_model_merged_int8.onnx",
                    sha256: "fa3ef9902734ce5ae6f9ef2bdb2ba9a6c4b5785b09f4f420ce036573dc9d090b",
                    bytes: 53_693_315,
                },
            ),
            (
                VOCAB_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/vocab.json",
                    sha256: "50d6a919f0a0601d56a04eb583c780d18553aa388254ba3158eb6a00f13e2c1a",
                    bytes: 1_036_584,
                },
            ),
            (
                CONFIG_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/config.json",
                    sha256: "f4d0608f7d918166da7edb3e188de5ef1bfe70d9802e785d271fd88111e9cf4b",
                    bytes: 2_243,
                },
            ),
            (
                GENERATION_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/generation_config.json",
                    sha256: "61070cf8de25b1e9256e8e102ded49d8d24a8369ed36ef84fdf21549e68125a0",
                    bytes: 3_832,
                },
            ),
            (
                PREPROCESSOR_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/preprocessor_config.json",
                    sha256: "a6a76d28c93edb273669eb9e0b0636a2bddbb1272c3261e47b7ca6dfdbac1b8d",
                    bytes: 339,
                },
            ),
        ],
        gpu: [
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/onnx/encoder_model_fp16.onnx",
                sha256: "ffba51b20185377c1b89362b1128148d0132de936a32b200af973da2a11a5369",
                bytes: 41_332_612,
            },
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-base/resolve/1846881b6b3a3024392c1eea3ad983695bc23925/onnx/decoder_model_merged_fp16.onnx",
                sha256: "2078fb08b753e3a74120e6a52446eef1b98f6152d1da43658731367a93fd71a4",
                bytes: 104_727_818,
            },
        ],
    },
    Model {
        id: WhisperModel::Small,
        files: [
            (
                ENCODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/onnx/encoder_model_int8.onnx",
                    sha256: "2601c9eb2d345c5916d4576d36f663a7c96589740fb2273828c48c3fc2c7db75",
                    bytes: 92_326_127,
                },
            ),
            (
                DECODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/onnx/decoder_model_merged_int8.onnx",
                    sha256: "ec07c3cbb64172c39791e26ee870a65ac22b458c36722bfe2776b3dbf741e0c9",
                    bytes: 156_750_845,
                },
            ),
            (
                VOCAB_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/vocab.json",
                    sha256: "50d6a919f0a0601d56a04eb583c780d18553aa388254ba3158eb6a00f13e2c1a",
                    bytes: 1_036_584,
                },
            ),
            (
                CONFIG_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/config.json",
                    sha256: "457854d452f17661e197d74aee12b8e74fb75ba30ebfaa7426d0d61ea1e08a18",
                    bytes: 2_227,
                },
            ),
            (
                GENERATION_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/generation_config.json",
                    sha256: "f538b28220c6a6d6f1af1458d4141cacb4ef4963df3de98a19490440c412ddf0",
                    bytes: 3_893,
                },
            ),
            (
                PREPROCESSOR_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/preprocessor_config.json",
                    sha256: "a6a76d28c93edb273669eb9e0b0636a2bddbb1272c3261e47b7ca6dfdbac1b8d",
                    bytes: 339,
                },
            ),
        ],
        gpu: [
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/onnx/encoder_model_fp16.onnx",
                sha256: "5549cd8666ff4b694ceb128bfa48b95bdcceec29075cf2c2212f90002cc058de",
                bytes: 176_607_756,
            },
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-small/resolve/36050c46d777d46dc4b5f43f6d90574fc38f8732/onnx/decoder_model_merged_fp16.onnx",
                sha256: "22aba6c7f5193701cbe1519051b6ef097eb530ad6887b7093065ec59b830f61d",
                bytes: 308_583_076,
            },
        ],
    },
    Model {
        id: WhisperModel::Turbo,
        files: [
            (
                ENCODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/onnx/encoder_model_q4.onnx",
                    sha256: "7e64b20dd7b556cb7a2c1c65d9fbf31af985323c006f993d2a9dd10083547d78",
                    bytes: 424_942_775,
                },
            ),
            (
                DECODER_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/onnx/decoder_model_merged_q4.onnx",
                    sha256: "8b933ac24074a24a1635084d06a4ed37e387ef22d980f184a5fc27418e9ceeb8",
                    bytes: 334_147_222,
                },
            ),
            (
                VOCAB_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/vocab.json",
                    sha256: "e2aa043ef015641d363d8288e7c241c85e36a5c761fb303598e0710233344387",
                    bytes: 1_036_558,
                },
            ),
            (
                CONFIG_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/config.json",
                    sha256: "35cd83669f75bc2867f3b3a4461850392d5e308cd6ea951c3700539883c28df1",
                    bytes: 1_332,
                },
            ),
            (
                GENERATION_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/generation_config.json",
                    sha256: "16f95291d2f47c944d3c2b19390bba7965666555c1ea2a0bdc850d1fab45612f",
                    bytes: 3_897,
                },
            ),
            (
                PREPROCESSOR_FILE,
                Remote {
                    url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/preprocessor_config.json",
                    sha256: "7ccc62c6f2765af1f3b46c00c9b5894426835a05021c8b9c01eecb6dfb542711",
                    bytes: 340,
                },
            ),
        ],
        gpu: [
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/onnx/encoder_model_q4f16.onnx",
                sha256: "aafd3383f1aa372db0825a885730ebbbae7e34871ec0460428663c69641a63d0",
                bytes: 369_974_078,
            },
            Remote {
                url: "https://huggingface.co/onnx-community/whisper-large-v3-turbo/resolve/360ebcde2559d60bb474678be3c1de9ef347d01a/onnx/decoder_model_merged_q4f16.onnx",
                sha256: "45981cdd958a4c8e1447839850d2e6e27e30974ccbe31b4a1e5ebe9ad8965a5f",
                bytes: 193_505_017,
            },
        ],
    },
];

const ALL: [WhisperModel; MODEL_COUNT] = [WhisperModel::Base, WhisperModel::Small, WhisperModel::Turbo];

static IN_USE: Mutex<Vec<WhisperModel>> = Mutex::new(Vec::new());

pub struct Download {
    model: WhisperModel,
    gpu: bool,
    cancel: Arc<AtomicBool>,
    done: Arc<AtomicU64>,
    total: u64,
}

#[derive(Clone, Serialize)]
pub struct Progress {
    model: WhisperModel,
    gpu: bool,
    done: u64,
    total: u64,
}

#[derive(Clone, Serialize)]
struct Done {
    model: WhisperModel,
    gpu: bool,
    error: Option<String>,
}

#[derive(Serialize)]
pub struct Status {
    ready: Vec<WhisperModel>,
    bytes: HashMap<WhisperModel, u64>,
    gpu_ready: Vec<WhisperModel>,
    gpu_bytes: HashMap<WhisperModel, u64>,
    downloading: Option<Progress>,
}

pub struct UseGuard(WhisperModel);

impl Drop for UseGuard {
    fn drop(&mut self) {
        let mut in_use = IN_USE.lock().unwrap();
        if let Some(index) = in_use.iter().position(|&m| m == self.0) {
            in_use.remove(index);
        }
    }
}

pub fn acquire(model: WhisperModel) -> UseGuard {
    IN_USE.lock().unwrap().push(model);
    UseGuard(model)
}

fn entry(model: WhisperModel) -> &'static Model {
    MODELS.iter().find(|m| m.id == model).unwrap()
}

fn name(model: WhisperModel) -> &'static str {
    match model {
        WhisperModel::Base => "base",
        WhisperModel::Small => "small",
        WhisperModel::Turbo => "turbo",
    }
}

fn pinned_bytes(model: WhisperModel) -> u64 {
    entry(model).files.iter().map(|(_, remote)| remote.bytes).sum()
}

fn gpu_bytes(model: WhisperModel) -> u64 {
    entry(model).gpu.iter().map(|remote| remote.bytes).sum()
}

pub fn gpu_ready(app: &AppHandle, model: WhisperModel) -> bool {
    let root = model_dir(app, model);
    GPU_FILES.iter().all(|file| root.join(file).is_file())
}

fn remove_gpu(app: &AppHandle, model: WhisperModel) -> io::Result<()> {
    let root = model_dir(app, model);
    for file in GPU_FILES {
        match fs::remove_file(root.join(file)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => return Err(e),
            _ => {}
        }
    }
    Ok(())
}

fn dir(app: &AppHandle) -> PathBuf {
    app.path().app_local_data_dir().unwrap_or_default().join("whisper")
}

pub fn model_dir(app: &AppHandle, model: WhisperModel) -> PathBuf {
    dir(app).join(name(model))
}

pub fn installed(app: &AppHandle) -> bool {
    ALL.into_iter().any(|model| model_dir(app, model).exists())
}

pub fn ready(app: &AppHandle, model: WhisperModel) -> bool {
    let root = model_dir(app, model);
    entry(model).files.iter().all(|(file, _)| root.join(file).is_file()) && crate::onnx::runtime_path(app).is_file()
}

fn total(app: &AppHandle, model: WhisperModel) -> u64 {
    pinned_bytes(model) + crate::onnx::missing_bytes(app)
}

fn fetch_model(app: &AppHandle, model: WhisperModel, fetch: &mut Fetch) -> Result<(), String> {
    let target = model_dir(app, model);
    fs::create_dir_all(&target).map_err(|e| e.to_string())?;
    fetch::clean(&target);
    crate::onnx::ensure(app, fetch)?;
    for (file, remote) in &entry(model).files {
        fetch.file(remote, &target.join(file))?;
    }
    Ok(())
}

fn fetch_gpu(app: &AppHandle, model: WhisperModel, fetch: &mut Fetch) -> Result<(), String> {
    let target = model_dir(app, model);
    fetch::clean(&target);
    for (file, remote) in GPU_FILES.iter().zip(&entry(model).gpu) {
        fetch.file(remote, &target.join(file))?;
    }
    Ok(())
}

fn run(app: &AppHandle, model: WhisperModel, gpu: bool, cancel: &AtomicBool, done: &AtomicU64, total: u64) -> Result<(), String> {
    let mut last = Instant::now() - THROTTLE;
    let mut fetch = Fetch {
        agent: fetch::agent(),
        cancel,
        done,
        report: Box::new(|done| {
            if last.elapsed() >= THROTTLE {
                last = Instant::now();
                let _ = app.emit_to("main", events::WHISPER_PROGRESS, Progress { model, gpu, done, total });
            }
        }),
    };
    if gpu {
        let result = fetch_gpu(app, model, &mut fetch);
        if result.is_err() {
            let _ = remove_gpu(app, model);
        }
        return result;
    }
    let result = fetch_model(app, model, &mut fetch);
    if result.is_err() {
        let _ = fs::remove_dir_all(model_dir(app, model));
    }
    result
}

#[tauri::command]
pub fn whisper_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let ready = ALL.into_iter().filter(|&m| self::ready(&app, m)).collect();
    let bytes = ALL.into_iter().map(|m| (m, pinned_bytes(m))).collect();
    let gpu_ready = ALL.into_iter().filter(|&m| self::gpu_ready(&app, m)).collect();
    let gpu_bytes = ALL.into_iter().map(|m| (m, self::gpu_bytes(m))).collect();
    let downloading = state.whisper_download.lock().unwrap().as_ref().map(|d| Progress {
        model: d.model,
        gpu: d.gpu,
        done: d.done.load(Ordering::Relaxed),
        total: d.total,
    });
    Status { ready, bytes, gpu_ready, gpu_bytes, downloading }
}

#[tauri::command]
pub fn whisper_download(
    app: AppHandle,
    state: tauri::State<State>,
    model: WhisperModel,
    gpu: Option<bool>,
) -> Result<(), String> {
    let gpu = gpu.unwrap_or(false);
    let mut slot = state.whisper_download.lock().unwrap();
    if slot.is_some() {
        return Err("a download is already running".into());
    }
    if gpu && !ready(&app, model) {
        return Err(crate::transcript::MODEL_MISSING.into());
    }
    if if gpu { gpu_ready(&app, model) } else { ready(&app, model) } {
        return Ok(());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = if gpu { gpu_bytes(model) } else { total(&app, model) };
    *slot = Some(Download { model, gpu, cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    std::thread::spawn(move || {
        let result = run(&app, model, gpu, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().whisper_download.lock().unwrap().take();
        let _ = app.emit_to("main", events::WHISPER_DONE, Done { model, gpu, error });
    });
    Ok(())
}

#[tauri::command]
pub fn whisper_cancel(state: tauri::State<State>) {
    if let Some(download) = state.whisper_download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

#[tauri::command]
pub fn whisper_delete(
    app: AppHandle,
    state: tauri::State<State>,
    model: WhisperModel,
    gpu: Option<bool>,
) -> Result<(), String> {
    let paddle_busy = state.download.lock().unwrap().is_some();
    let translate_busy =
        state.translate_download.lock().unwrap().is_some()
        || state.cutout_download.lock().unwrap().is_some()
        || state.upscale_download.lock().unwrap().is_some();
    let download = state.whisper_download.lock().unwrap();
    if download.as_ref().is_some_and(|d| d.model == model) {
        return Err("this model is downloading".into());
    }
    if IN_USE.lock().unwrap().contains(&model) {
        return Err("this model is in use".into());
    }
    if gpu.unwrap_or(false) {
        return remove_gpu(&app, model).map_err(|e| e.to_string());
    }
    let removed = match fs::remove_dir_all(model_dir(&app, model)) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    };
    crate::onnx::release(&app, paddle_busy || translate_busy || download.is_some());
    removed
}
