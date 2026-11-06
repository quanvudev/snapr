use crate::config::Acceleration;
use crate::fetch::{self, Fetch, Remote};
use crate::gpu::{self, Adapter};
use crate::{State, events, paddle};
use ort::environment::Environment;
use ort::ep;
use ort::memory::{AllocationDevice, AllocatorType, MemoryInfo, MemoryType};
use ort::session::{Session, SessionInputValue};
use ort::value::DynValue;
use std::cell::Cell;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};

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
const GPU_DIR: &str = "runtime";
const CPU_DIR: &str = "paddle";
const THROTTLE: Duration = Duration::from_millis(100);
pub const LOADED_RUNTIME: &str = "LOADED_RUNTIME";
pub const DOWNLOADING: &str = "DOWNLOADING";
pub const RECORDING: &str = "RECORDING";
pub const QUEUE_BUSY: &str = "QUEUE_BUSY";

struct Wheel {
    remote: Remote,
    entries: &'static [&'static str],
}

const DIRECTML: [Wheel; 1] = [Wheel {
    remote: Remote {
        url: "https://files.pythonhosted.org/packages/60/53/2bd2696fac19cf8ca55496a0bcfe431f3aff9579eabbb0e231dc238acf6f/onnxruntime_directml-1.24.4-cp313-cp313-win_amd64.whl",
        sha256: "2f1031cb2281e5b27cca9efe0b9399317c7286e4d226f7a79d4ab79bbd94d19e",
        bytes: 25_112_253,
    },
    entries: &[
        "onnxruntime/capi/onnxruntime.dll",
        "onnxruntime/capi/onnxruntime_providers_shared.dll",
        "onnxruntime/capi/DirectML.dll",
    ],
}];

const CUDA: [Wheel; 4] = [
    Wheel {
        remote: Remote {
            url: "https://files.pythonhosted.org/packages/23/3a/1a2305f400b97fa401e8249002c04040ce28ae21a96588667bcd1cb83d05/onnxruntime_gpu-1.31.1-cp313-cp313-win_amd64.whl",
            sha256: "1a667241591c89c516b43689a0ae38c4d99d0cb45a2220e95a9f822aeb6887c4",
            bytes: 168_097_596,
        },
        entries: &[
            "onnxruntime/capi/onnxruntime.dll",
            "onnxruntime/capi/onnxruntime_providers_shared.dll",
            "onnxruntime/capi/onnxruntime_providers_cuda.dll",
        ],
    },
    Wheel {
        remote: Remote {
            url: "https://files.pythonhosted.org/packages/12/39/e1b1b4adb8b171894d9a89035df0596d448cf73dc403802d24dbe30cc0be/nvidia_cublas-13.10.0.20-py3-none-win_amd64.whl",
            sha256: "c310b749eea855b88fafb906de08d6819b87216c6711640771764fea5dcd8546",
            bytes: 427_284_863,
        },
        entries: &["nvidia/cu13/bin/x86_64/cublas64_13.dll", "nvidia/cu13/bin/x86_64/cublasLt64_13.dll"],
    },
    Wheel {
        remote: Remote {
            url: "https://files.pythonhosted.org/packages/35/99/e07dd9565d2e0317c1a62ff39ae9fc0542c048f74bcfc777c8be97710347/nvidia_cuda_runtime-13.5.37-py3-none-win_amd64.whl",
            sha256: "f225d3fa5ea6bfbb58fc4979ef72375cbe661d9107f0d8a4ff22cc151525e4a9",
            bytes: 2_780_254,
        },
        entries: &["nvidia/cu13/bin/x86_64/cudart64_13.dll"],
    },
    Wheel {
        remote: Remote {
            url: "https://files.pythonhosted.org/packages/51/d6/dc4820f46f192e566f3e589e166fc7600acce5b0a09681cf25cdef20acfc/nvidia_cudnn_cu13-9.28.0.42-py3-none-win_amd64.whl",
            sha256: "fdddb1a8be28b5c95893bb1646822c53b3cf4a762e957c74ee1dd8f411993f05",
            bytes: 406_229_469,
        },
        entries: &[
            "nvidia/cudnn/bin/cudnn64_9.dll",
            "nvidia/cudnn/bin/cudnn_graph64_9.dll",
            "nvidia/cudnn/bin/cudnn_ops64_9.dll",
            "nvidia/cudnn/bin/cudnn_cnn64_9.dll",
            "nvidia/cudnn/bin/cudnn_heuristic64_9.dll",
            "nvidia/cudnn/bin/cudnn_engines_precompiled64_9.dll",
            "nvidia/cudnn/bin/cudnn_engines_runtime_compiled64_9.dll",
            "nvidia/cudnn/bin/cudnn_engines_tensor_ir64_9.dll",
        ],
    },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Runtime {
    Cpu,
    DirectMl,
    Cuda,
}

impl Runtime {
    pub const GPU: [Runtime; 2] = [Runtime::DirectMl, Runtime::Cuda];

    fn id(self) -> &'static str {
        match self {
            Runtime::Cpu => "cpu",
            Runtime::DirectMl => "directml",
            Runtime::Cuda => "cuda",
        }
    }

    fn wheels(self) -> &'static [Wheel] {
        match self {
            Runtime::Cpu => &[],
            Runtime::DirectMl => &DIRECTML,
            Runtime::Cuda => &CUDA,
        }
    }

    pub fn bytes(self) -> u64 {
        match self {
            Runtime::Cpu => RUNTIME.bytes,
            gpu => gpu.wheels().iter().map(|wheel| wheel.remote.bytes).sum(),
        }
    }

    fn files(self) -> impl Iterator<Item = &'static str> {
        self.wheels().iter().flat_map(|wheel| wheel.entries.iter().map(|entry| file_name(entry)))
    }
}

fn file_name(entry: &str) -> &str {
    entry.rsplit('/').next().unwrap_or(entry)
}

#[derive(Clone, Debug)]
pub struct Target {
    pub runtime: Runtime,
    pub dll: PathBuf,
    pub device: i32,
}

struct Loaded {
    environment: Environment,
    runtime: Runtime,
    device: i32,
}

pub struct Download {
    pub runtime: Runtime,
    pub cancel: Arc<AtomicBool>,
    pub done: Arc<AtomicU64>,
    pub total: u64,
}

#[derive(Clone, Serialize)]
pub struct Progress {
    pub runtime: Runtime,
    pub done: u64,
    pub total: u64,
}

#[derive(Clone, Serialize)]
struct Done {
    runtime: Runtime,
    error: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Workload {
    Ocr,
    Whisper,
    Translate,
}

static LOADED: OnceLock<Result<Loaded, String>> = OnceLock::new();
static GPU_ERROR: Mutex<Option<String>> = Mutex::new(None);

thread_local! {
    static CPU_ONLY: Cell<bool> = const { Cell::new(false) };
}

pub fn runtime_path(app: &AppHandle) -> PathBuf {
    paddle::dir(app).join(RUNTIME_FILE)
}

pub fn gpu_dir(app: &AppHandle, runtime: Runtime) -> PathBuf {
    app.path().app_local_data_dir().unwrap_or_default().join(GPU_DIR).join(runtime.id())
}

pub fn ready(app: &AppHandle, runtime: Runtime) -> bool {
    match runtime {
        Runtime::Cpu => runtime_path(app).is_file(),
        gpu => {
            let dir = gpu_dir(app, gpu);
            gpu.files().all(|file| dir.join(file).is_file())
        }
    }
}

pub fn disk_bytes(app: &AppHandle, runtime: Runtime) -> u64 {
    match runtime {
        Runtime::Cpu => fs::metadata(runtime_path(app)).map_or(0, |m| m.len()),
        gpu => fetch::disk_bytes(&gpu_dir(app, gpu)),
    }
}

pub fn missing_bytes(app: &AppHandle) -> u64 {
    if runtime_path(app).is_file() { 0 } else { RUNTIME.bytes }
}

pub fn resolve(acceleration: Acceleration, adapter: Option<&Adapter>, ready: impl Fn(Runtime) -> bool) -> Runtime {
    let nvidia = adapter.is_some_and(Adapter::nvidia);
    match acceleration {
        Acceleration::Cpu => Runtime::Cpu,
        Acceleration::DirectMl if adapter.is_some() && ready(Runtime::DirectMl) => Runtime::DirectMl,
        Acceleration::Cuda if nvidia && ready(Runtime::Cuda) => Runtime::Cuda,
        Acceleration::Auto if nvidia && ready(Runtime::Cuda) => Runtime::Cuda,
        Acceleration::Auto if adapter.is_some() && ready(Runtime::DirectMl) => Runtime::DirectMl,
        _ => Runtime::Cpu,
    }
}

pub fn selected(app: &AppHandle) -> Runtime {
    let acceleration = app.state::<State>().cfg.read().unwrap().acceleration();
    resolve(acceleration, gpu::cached().as_ref(), |runtime| ready(app, runtime))
}

pub fn target(app: &AppHandle) -> Target {
    let runtime = selected(app);
    let device = match runtime {
        Runtime::DirectMl => gpu::cached().map_or(0, |adapter| adapter.index as i32),
        _ => 0,
    };
    let dll = match runtime {
        Runtime::Cpu => runtime_path(app),
        gpu => gpu_dir(app, gpu).join(RUNTIME_FILE),
    };
    Target { runtime, dll, device }
}

fn load(dll: &Path, runtime: Runtime, device: i32) -> Result<Loaded, String> {
    let environment = ort::init_from(dll)
        .map_err(|e| e.to_string())?
        .with_name(ENVIRONMENT_NAME)
        .build()
        .map_err(|e| e.to_string())?;
    Ok(Loaded { environment, runtime, device })
}

fn cpu_beside(gpu_dll: &Path) -> PathBuf {
    let root = gpu_dll.parent().and_then(Path::parent).and_then(Path::parent).unwrap_or(Path::new(""));
    root.join(CPU_DIR).join(RUNTIME_FILE)
}

pub fn init(target: &Target) -> Result<&'static Environment, String> {
    LOADED
        .get_or_init(|| {
            if target.runtime == Runtime::Cpu {
                return load(&target.dll, Runtime::Cpu, 0);
            }
            if let Some(dir) = target.dll.parent() {
                search_dir(dir);
            }
            load(&target.dll, target.runtime, target.device).or_else(|error| {
                set_gpu_error(Some(error));
                load(&cpu_beside(&target.dll), Runtime::Cpu, 0)
            })
        })
        .as_ref()
        .map(|loaded| &loaded.environment)
        .map_err(Clone::clone)
}

pub fn environment() -> Result<&'static Environment, String> {
    match LOADED.get() {
        Some(result) => result.as_ref().map(|loaded| &loaded.environment).map_err(Clone::clone),
        None => Err(NOT_INITIALISED.into()),
    }
}

pub fn loaded() -> Option<(Runtime, i32)> {
    LOADED.get().and_then(|result| result.as_ref().ok()).map(|loaded| (loaded.runtime, loaded.device))
}

pub fn gpu_error() -> Option<String> {
    GPU_ERROR.lock().unwrap().clone()
}

pub fn set_gpu_error(error: Option<String>) {
    *GPU_ERROR.lock().unwrap() = error;
}

pub fn gpu_for(workload: Workload) -> Option<(Runtime, i32)> {
    if CPU_ONLY.get() {
        return None;
    }
    match (loaded()?, workload) {
        ((Runtime::Cuda, _), _) | ((Runtime::DirectMl, _), Workload::Ocr) => loaded(),
        _ => None,
    }
}

pub fn retry_on_cpu<T>(workload: Workload, mut run: impl FnMut() -> Result<T, String>) -> Result<T, String> {
    let on_gpu = gpu_for(workload).is_some();
    match run() {
        Err(error) if on_gpu && error != crate::queue::CANCELLED => {
            set_gpu_error(Some(error));
            CPU_ONLY.set(true);
            let result = run();
            CPU_ONLY.set(false);
            result
        }
        result => result,
    }
}

fn cpu_session(env: &Environment, path: &Path, workload: Workload) -> Result<Session, String> {
    let builder = Session::builder(env).map_err(|e| e.to_string())?;
    let mut builder = if workload == Workload::Ocr {
        let threads = std::thread::available_parallelism().map_or(1, |n| (n.get() / 2).max(1));
        builder
            .with_intra_threads(threads)
            .map_err(|e| e.to_string())?
            .with_intra_op_spinning(false)
            .map_err(|e| e.to_string())?
            .with_inter_op_spinning(false)
            .map_err(|e| e.to_string())?
    } else {
        builder
    };
    builder.commit_from_file(path).map_err(|e| e.to_string())
}

fn gpu_session(env: &Environment, path: &Path, runtime: Runtime, device: i32) -> Result<Session, String> {
    let builder = Session::builder(env).map_err(|e| e.to_string())?;
    let mut builder = match runtime {
        Runtime::DirectMl => builder
            .with_memory_pattern(false)
            .map_err(|e| e.to_string())?
            .with_parallel_execution(false)
            .map_err(|e| e.to_string())?
            .with_execution_providers([ep::DirectML::default().with_device_id(device).build().error_on_failure()])
            .map_err(|e| e.to_string())?,
        _ => builder
            .with_execution_providers([ep::CUDA::default().with_device_id(device).build().error_on_failure()])
            .map_err(|e| e.to_string())?,
    };
    builder.commit_from_file(path).map_err(|e| e.to_string())
}

pub fn on_cpu<T>(run: impl FnOnce() -> T) -> T {
    let before = CPU_ONLY.replace(true);
    let result = run();
    CPU_ONLY.set(before);
    result
}

pub fn session(env: &Environment, path: &Path, workload: Workload) -> Result<Session, String> {
    Ok(placed(env, path, workload)?.0)
}

fn placed(env: &Environment, path: &Path, workload: Workload) -> Result<(Session, Option<(Runtime, i32)>), String> {
    if let Some((runtime, device)) = gpu_for(workload) {
        match gpu_session(env, path, runtime, device) {
            Ok(session) => {
                set_gpu_error(None);
                return Ok((session, Some((runtime, device))));
            }
            Err(error) => set_gpu_error(Some(error)),
        }
    }
    Ok((cpu_session(env, path, workload)?, None))
}

pub enum Feed<'a> {
    Owned(DynValue),
    Ref(&'a DynValue),
}

impl Feed<'_> {
    fn value(&self) -> &DynValue {
        match self {
            Feed::Owned(value) => value,
            Feed::Ref(value) => value,
        }
    }
}

pub struct Runner {
    session: Session,
    device: Option<MemoryInfo<'static>>,
    host: MemoryInfo<'static>,
    outputs: Vec<String>,
}

pub fn runner(env: &Environment, path: &Path, workload: Workload) -> Result<Runner, String> {
    let (session, placement) = placed(env, path, workload)?;
    let device = match placement.filter(|_| workload == Workload::Whisper) {
        Some((Runtime::Cuda, id)) => Some(memory(AllocationDevice::CUDA, id)?),
        Some((Runtime::DirectMl, id)) => Some(memory(AllocationDevice::DIRECTML, id)?),
        _ => None,
    };
    let outputs = session.outputs().iter().map(|o| o.name().to_owned()).collect();
    Ok(Runner { session, device, host: memory(AllocationDevice::CPU, 0)?, outputs })
}

fn memory(device: AllocationDevice<'static>, id: i32) -> Result<MemoryInfo<'static>, String> {
    MemoryInfo::new(device, id, AllocatorType::Device, MemoryType::Default).map_err(|e| e.to_string())
}

impl Runner {
    pub fn input_names(&self) -> Vec<String> {
        self.session.inputs().iter().map(|i| i.name().to_owned()).collect()
    }

    pub fn run(&mut self, feed: Vec<(String, Feed)>, host: &[&str]) -> Result<Vec<(String, DynValue)>, String> {
        let Some(device) = &self.device else {
            let inputs: Vec<(String, SessionInputValue)> = feed
                .into_iter()
                .map(|(name, value)| match value {
                    Feed::Owned(value) => (name, value.into()),
                    Feed::Ref(value) => (name, value.into()),
                })
                .collect();
            let outputs = self.session.run(inputs).map_err(|e| e.to_string())?;
            return Ok(outputs.into_iter().map(|(name, value)| (name.to_owned(), value)).collect());
        };
        let mut binding = self.session.create_binding().map_err(|e| e.to_string())?;
        for (name, value) in &feed {
            binding.bind_input(name.as_str(), value.value()).map_err(|e| e.to_string())?;
        }
        for name in &self.outputs {
            let place = if host.contains(&name.as_str()) { &self.host } else { device };
            binding.bind_output_to_device(name.as_str(), place).map_err(|e| e.to_string())?;
        }
        let outputs = self.session.run_binding(&binding).map_err(|e| e.to_string())?;
        Ok(outputs.into_iter().map(|(name, value)| (name.to_owned(), value)).collect())
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
    let extracted = extract(&wheel, &[RUNTIME_ENTRY], root);
    let _ = fs::remove_file(&wheel);
    extracted
}

pub fn release(app: &AppHandle, busy: bool) {
    if !busy && !paddle::installed(app) && !crate::whisper::installed(app) && !crate::translate::installed(app) {
        let _ = fs::remove_file(runtime_path(app));
    }
}

fn fetch_runtime(dir: &Path, wheels: &[Wheel], fetch: &mut Fetch) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    fetch::clean(dir);
    for wheel in wheels {
        let part = dir.join(WHEEL_PART);
        fetch.get(&wheel.remote, &part)?;
        let extracted = extract(&part, wheel.entries, dir);
        let _ = fs::remove_file(&part);
        extracted?;
    }
    Ok(())
}

fn fetch_into(dir: &Path, wheels: &[Wheel], fetch: &mut Fetch) -> Result<(), String> {
    let result = fetch_runtime(dir, wheels, fetch);
    if result.is_err() {
        let _ = fs::remove_dir_all(dir);
    }
    result
}

fn run_download(app: &AppHandle, runtime: Runtime, cancel: &AtomicBool, done: &AtomicU64, total: u64) -> Result<(), String> {
    let mut last = Instant::now() - THROTTLE;
    let mut fetch = Fetch {
        agent: fetch::agent(),
        cancel,
        done,
        report: Box::new(|done| {
            if last.elapsed() >= THROTTLE {
                last = Instant::now();
                let _ = app.emit_to("main", events::ACCELERATION_PROGRESS, Progress { runtime, done, total });
            }
        }),
    };
    fetch_into(&gpu_dir(app, runtime), runtime.wheels(), &mut fetch)
}

pub fn download(app: &AppHandle, runtime: Runtime) -> Result<(), String> {
    if runtime == Runtime::Cpu || ready(app, runtime) {
        return Ok(());
    }
    let state = app.state::<State>();
    let mut slot = state.runtime_download.lock().unwrap();
    if slot.is_some() {
        return Err(DOWNLOADING.into());
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let done = Arc::new(AtomicU64::new(0));
    let total = runtime.bytes();
    *slot = Some(Download { runtime, cancel: cancel.clone(), done: done.clone(), total });
    drop(slot);
    let app = app.clone();
    std::thread::spawn(move || {
        let result = run_download(&app, runtime, &cancel, &done, total);
        let error = match result {
            Err(_) if cancel.load(Ordering::Relaxed) => None,
            Err(e) => Some(e),
            Ok(()) => None,
        };
        app.state::<State>().runtime_download.lock().unwrap().take();
        let _ = app.emit_to("main", events::ACCELERATION_DONE, Done { runtime, error });
    });
    Ok(())
}

pub fn cancel(app: &AppHandle) {
    if let Some(download) = app.state::<State>().runtime_download.lock().unwrap().as_ref() {
        download.cancel.store(true, Ordering::Relaxed);
    }
}

pub fn delete(app: &AppHandle, runtime: Runtime) -> Result<(), String> {
    if runtime == Runtime::Cpu {
        return Ok(());
    }
    if loaded().is_some_and(|(current, _)| current == runtime) {
        return Err(LOADED_RUNTIME.into());
    }
    let downloading = app.state::<State>().runtime_download.lock().unwrap().as_ref().is_some_and(|d| d.runtime == runtime);
    if downloading {
        return Err(DOWNLOADING.into());
    }
    let dir = gpu_dir(app, runtime);
    let removed = match fs::remove_dir_all(&dir) {
        Err(e) if e.kind() != io::ErrorKind::NotFound => Err(e.to_string()),
        _ => Ok(()),
    };
    let _ = fs::remove_dir(dir.parent().unwrap_or(&dir));
    removed
}

#[derive(Serialize)]
pub struct RuntimeStatus {
    id: Runtime,
    ready: bool,
    bytes: u64,
    disk: u64,
}

#[derive(Serialize)]
pub struct Status {
    adapter: Option<Adapter>,
    recommended: Runtime,
    resolved: Runtime,
    loaded: Option<Runtime>,
    restart_needed: bool,
    runtimes: Vec<RuntimeStatus>,
    downloading: Option<Progress>,
    error: Option<String>,
}

pub fn recommended(adapter: Option<&Adapter>) -> Runtime {
    match adapter {
        Some(adapter) if adapter.nvidia() => Runtime::Cuda,
        Some(_) => Runtime::DirectMl,
        None => Runtime::Cpu,
    }
}

#[tauri::command]
pub fn acceleration_status(app: AppHandle, state: tauri::State<State>) -> Status {
    let adapter = gpu::refresh();
    let resolved = selected(&app);
    let loaded = loaded().map(|(runtime, _)| runtime);
    let runtimes = Runtime::GPU
        .into_iter()
        .map(|id| RuntimeStatus { id, ready: ready(&app, id), bytes: id.bytes(), disk: disk_bytes(&app, id) })
        .collect();
    let downloading = state.runtime_download.lock().unwrap().as_ref().map(|d| Progress {
        runtime: d.runtime,
        done: d.done.load(Ordering::Relaxed),
        total: d.total,
    });
    Status {
        recommended: recommended(adapter.as_ref()),
        adapter,
        resolved,
        loaded,
        restart_needed: loaded.is_some_and(|runtime| runtime != resolved),
        runtimes,
        downloading,
        error: gpu_error(),
    }
}

#[tauri::command]
pub fn acceleration_download(app: AppHandle, runtime: Runtime) -> Result<(), String> {
    download(&app, runtime)
}

#[tauri::command]
pub fn acceleration_cancel(app: AppHandle) {
    cancel(&app);
}

#[tauri::command]
pub fn acceleration_delete(app: AppHandle, runtime: Runtime) -> Result<(), String> {
    delete(&app, runtime)
}

#[tauri::command]
pub fn acceleration_restart(app: AppHandle, state: tauri::State<State>) -> Result<(), String> {
    if crate::update::recording(&app) {
        return Err(RECORDING.into());
    }
    if !state.queue.list().is_empty() {
        return Err(QUEUE_BUSY.into());
    }
    crate::update::relaunch()?;
    app.cleanup_before_exit();
    std::process::exit(0);
}

fn extract(wheel: &Path, entries: &[&str], dir: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(File::open(wheel).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    for name in entries {
        let to = dir.join(file_name(name));
        let tmp = fetch::part(&to);
        let result = (|| {
            let mut entry = archive.by_name(name).map_err(|e| format!("{name}: {e}"))?;
            let mut out = File::create(&tmp).map_err(|e| e.to_string())?;
            io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            out.flush().map_err(|e| e.to_string())?;
            drop(out);
            fs::rename(&tmp, &to).map_err(|e| e.to_string())
        })();
        if result.is_err() {
            let _ = fs::remove_file(&tmp);
            return result;
        }
    }
    Ok(())
}

pub fn search_dir(dir: &Path) {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = dir.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    unsafe { windows_sys::Win32::System::LibraryLoader::SetDllDirectoryW(wide.as_ptr()) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use ort::ep;
    use ort::session::Session;
    use ort::value::Tensor;

    const DIR_ENV: &str = "SNAPR_RUNTIME_TEST_DIR";

    fn fetcher<'a>(cancel: &'a AtomicBool, done: &'a AtomicU64, stop_at: u64) -> Fetch<'a> {
        Fetch {
            agent: fetch::agent(),
            cancel,
            done,
            report: Box::new(move |received| {
                if received >= stop_at {
                    cancel.store(true, Ordering::Relaxed);
                }
            }),
        }
    }

    #[test]
    fn resolves_runtimes() {
        let nvidia = Adapter { index: 0, name: "RTX".into(), vendor: 0x10DE, memory: 16 << 30 };
        let amd = Adapter { index: 1, name: "Radeon".into(), vendor: 0x1002, memory: 8 << 30 };
        let all = |_: Runtime| true;
        let dml_only = |runtime: Runtime| runtime == Runtime::DirectMl;
        assert_eq!(resolve(Acceleration::Auto, None, all), Runtime::Cpu);
        assert_eq!(resolve(Acceleration::Auto, Some(&nvidia), all), Runtime::Cuda);
        assert_eq!(resolve(Acceleration::Auto, Some(&nvidia), dml_only), Runtime::DirectMl);
        assert_eq!(resolve(Acceleration::Auto, Some(&amd), all), Runtime::DirectMl);
        assert_eq!(resolve(Acceleration::Cuda, Some(&amd), all), Runtime::Cpu);
        assert_eq!(resolve(Acceleration::Cpu, Some(&nvidia), all), Runtime::Cpu);
        assert_eq!(resolve(Acceleration::DirectMl, Some(&nvidia), |_| false), Runtime::Cpu);
    }

    #[test]
    #[ignore]
    fn downloads_directml() {
        let Ok(root) = std::env::var(DIR_ENV) else { return };
        let dir = Path::new(&root).join("directml");
        let _ = fs::remove_dir_all(&dir);
        let (cancel, done) = (AtomicBool::new(false), AtomicU64::new(0));
        fetch_into(&dir, &DIRECTML, &mut fetcher(&cancel, &done, u64::MAX)).unwrap();
        for file in Runtime::DirectMl.files() {
            assert!(dir.join(file).is_file(), "{file} missing");
        }
        assert!(!dir.join(WHEEL_PART).exists());
        eprintln!("directml on disk: {} bytes", fetch::disk_bytes(&dir));

        let cancelled = Path::new(&root).join("cancelled");
        let (cancel, done) = (AtomicBool::new(false), AtomicU64::new(0));
        assert!(fetch_into(&cancelled, &DIRECTML, &mut fetcher(&cancel, &done, 5_000_000)).is_err());
        assert!(!cancelled.exists(), "cancel left files");

        let corrupt = [Wheel {
            remote: Remote { url: DIRECTML[0].remote.url, sha256: "00", bytes: DIRECTML[0].remote.bytes },
            entries: DIRECTML[0].entries,
        }];
        let bad = Path::new(&root).join("corrupt");
        let (cancel, done) = (AtomicBool::new(false), AtomicU64::new(0));
        let error = fetch_into(&bad, &corrupt, &mut fetcher(&cancel, &done, u64::MAX)).unwrap_err();
        assert!(error.contains("checksum"), "{error}");
        assert!(!bad.exists(), "corrupt download left files");
    }

    #[test]
    #[ignore]
    fn downloads_cuda() {
        let Ok(root) = std::env::var(DIR_ENV) else { return };
        let dir = Path::new(&root).join("cuda");
        let (cancel, done) = (AtomicBool::new(false), AtomicU64::new(0));
        fetch_into(&dir, &CUDA, &mut fetcher(&cancel, &done, u64::MAX)).unwrap();
        for file in Runtime::Cuda.files() {
            assert!(dir.join(file).is_file(), "{file} missing");
        }
        eprintln!("cuda on disk: {} bytes, downloaded {}", fetch::disk_bytes(&dir), done.load(Ordering::Relaxed));
    }

    #[test]
    #[ignore]
    fn falls_back_to_cpu() {
        let (Ok(dir), Ok(model)) = (std::env::var("SNAPR_CUDA_COPY"), std::env::var("SNAPR_ONNX_MODEL")) else { return };
        let dir = PathBuf::from(dir);
        let provider = dir.join("onnxruntime_providers_cuda.dll");
        let hidden = dir.join("onnxruntime_providers_cuda.dll.off");
        fs::rename(&provider, &hidden).unwrap();
        let env = init(&Target { runtime: Runtime::Cuda, dll: dir.join(RUNTIME_FILE), device: 0 }).unwrap();
        let run = || -> Result<usize, String> {
            let mut session = session(env, Path::new(&model), Workload::Whisper)?;
            let input = session.inputs()[0].name().to_owned();
            let tensor = ort::value::Tensor::from_array(([1usize, 80, 3000], vec![0.1f32; 80 * 3000])).map_err(|e| e.to_string())?;
            session.run(vec![(input, ort::session::SessionInputValue::from(tensor))]).map_err(|e| e.to_string())?;
            Ok(1)
        };
        let started = Instant::now();
        assert_eq!(retry_on_cpu(Workload::Whisper, run), Ok(1));
        let error = gpu_error();
        eprintln!("without the provider: {:?}, error {error:?}", started.elapsed());
        assert!(error.is_some());
        fs::rename(&hidden, &provider).unwrap();
        let started = Instant::now();
        assert_eq!(retry_on_cpu(Workload::Whisper, run), Ok(1));
        eprintln!("with the provider: {:?}, error {:?}", started.elapsed(), gpu_error());
        assert!(gpu_error().is_none());
    }

    #[test]
    #[ignore]
    fn gpu_session_runs() {
        let (Ok(dll), Ok(provider), Ok(model)) =
            (std::env::var("SNAPR_ORT_DLL"), std::env::var("SNAPR_ORT_EP"), std::env::var("SNAPR_ONNX_MODEL"))
        else {
            return;
        };
        let runtime = if provider == "directml" { Runtime::DirectMl } else { Runtime::Cuda };
        let env = init(&Target { runtime, dll: PathBuf::from(dll), device: 0 }).unwrap();
        assert_eq!(loaded().map(|(r, _)| r), Some(runtime));
        let builder = Session::builder(env).unwrap();
        let mut builder = match runtime {
            Runtime::DirectMl => builder
                .with_memory_pattern(false)
                .unwrap()
                .with_parallel_execution(false)
                .unwrap()
                .with_execution_providers([ep::DirectML::default().build().error_on_failure()])
                .unwrap(),
            _ => builder.with_execution_providers([ep::CUDA::default().build().error_on_failure()]).unwrap(),
        };
        let mut session = builder.commit_from_file(&model).unwrap();
        let input = session.inputs()[0].name().to_owned();
        let run = |session: &mut Session| {
            let started = std::time::Instant::now();
            let tensor = Tensor::from_array(([1usize, 80, 3000], vec![0.1f32; 80 * 3000])).unwrap();
            session.run(vec![(input.clone(), ort::session::SessionInputValue::from(tensor))]).unwrap();
            started.elapsed()
        };
        run(&mut session);
        eprintln!("{provider} encoder run in {:?}", run(&mut session));
    }
}
