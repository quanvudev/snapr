use crate::events;
use crate::State;
use crate::capture;
use crate::gif::GifTask;
use crate::subtitle::{Position, Size, Style, SubtitleTask};
use crate::transcript::{TranscribeTask, Track};
use crate::video::CompressTask;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;

pub const CANCELLED: &str = "cancelled";
const THROTTLE: Duration = Duration::from_millis(100);
const SHUTDOWN_WAIT: Duration = Duration::from_secs(3);
const PART_SUFFIX: &str = ".part";
const PART_EXTS: [&str; 3] = ["gif", "mp4", "srt"];
const HISTORY_LIMIT: usize = 50;

pub trait Task: Send {
    fn kind(&self) -> &'static str;
    fn name(&self) -> String;
    fn source(&self) -> Option<&Path> {
        None
    }
    fn detail(&self) -> Option<String> {
        None
    }
    fn run(&self, progress: &Progress, cancel: &AtomicBool) -> Result<Option<PathBuf>, String>;
}

pub struct Progress {
    app: AppHandle,
    done: AtomicU64,
    total: AtomicU64,
    last: Mutex<Instant>,
}

impl Progress {
    fn new(app: &AppHandle) -> Progress {
        Progress {
            app: app.clone(),
            done: AtomicU64::new(0),
            total: AtomicU64::new(0),
            last: Mutex::new(Instant::now() - THROTTLE),
        }
    }

    pub fn set(&self, done: u64, total: u64) {
        self.done.store(done, Ordering::Relaxed);
        self.total.store(total, Ordering::Relaxed);
        let mut last = self.last.lock().unwrap();
        if last.elapsed() >= THROTTLE {
            *last = Instant::now();
            drop(last);
            emit(&self.app);
        }
    }
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum ItemState {
    Waiting,
    Running,
}

#[derive(Serialize, Clone)]
pub struct QueueItem {
    id: u64,
    kind: &'static str,
    name: String,
    source: Option<String>,
    detail: Option<String>,
    state: ItemState,
    done: u64,
    total: u64,
}

#[derive(Serialize, Clone)]
struct QueueEnded {
    id: u64,
    kind: &'static str,
    name: String,
    source: Option<String>,
    detail: Option<String>,
    output: Option<String>,
    error: Option<String>,
}

#[derive(Serialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryState {
    Done,
    Failed,
    Cancelled,
}

#[derive(Serialize, Clone)]
pub struct HistoryItem {
    id: u64,
    kind: &'static str,
    name: String,
    source: Option<String>,
    detail: Option<String>,
    output: Option<String>,
    state: HistoryState,
    error: Option<String>,
    retryable: bool,
    #[serde(skip)]
    retry: Option<TaskSpec>,
}

#[derive(Clone)]
struct Entry {
    id: u64,
    kind: &'static str,
    name: String,
    source: Option<String>,
    detail: Option<String>,
    retry: Option<TaskSpec>,
    progress: Arc<Progress>,
    cancel: Arc<AtomicBool>,
}

impl Entry {
    fn ended(&self, result: &Result<Option<PathBuf>, String>) -> HistoryItem {
        let (state, output, error) = match result {
            Ok(output) => (HistoryState::Done, output.as_ref().map(|path| path.display().to_string()), None),
            Err(e) if e == CANCELLED => (HistoryState::Cancelled, None, None),
            Err(e) => (HistoryState::Failed, None, Some(e.clone())),
        };
        HistoryItem {
            id: self.id,
            kind: self.kind,
            name: self.name.clone(),
            source: self.source.clone(),
            detail: self.detail.clone(),
            output,
            state,
            error,
            retryable: self.retry.is_some(),
            retry: self.retry.clone(),
        }
    }

    fn view(&self, state: ItemState) -> QueueItem {
        QueueItem {
            id: self.id,
            kind: self.kind,
            name: self.name.clone(),
            source: self.source.clone(),
            detail: self.detail.clone(),
            state,
            done: self.progress.done.load(Ordering::Relaxed),
            total: self.progress.total.load(Ordering::Relaxed),
        }
    }
}

struct Item {
    entry: Entry,
    task: Box<dyn Task>,
}

#[derive(Default)]
struct Items {
    waiting: VecDeque<Item>,
    running: Option<Entry>,
    history: VecDeque<HistoryItem>,
    next_id: u64,
    closed: bool,
}

impl Items {
    fn record(&mut self, item: HistoryItem) {
        self.history.push_front(item);
        self.history.truncate(HISTORY_LIMIT);
    }
}

#[derive(Default)]
pub struct Queue {
    items: Mutex<Items>,
    wake: Condvar,
}

impl Queue {
    pub fn add(&self, app: &AppHandle, task: Box<dyn Task>, retry: Option<TaskSpec>) -> u64 {
        let id = {
            let mut items = self.items.lock().unwrap();
            items.next_id += 1;
            let id = items.next_id;
            let entry = Entry {
                id,
                kind: task.kind(),
                name: task.name(),
                source: task.source().map(|path| path.display().to_string()),
                detail: task.detail(),
                retry,
                progress: Arc::new(Progress::new(app)),
                cancel: Arc::new(AtomicBool::new(false)),
            };
            items.waiting.push_back(Item { entry, task });
            id
        };
        self.wake.notify_all();
        emit(app);
        id
    }

    pub fn list(&self) -> Vec<QueueItem> {
        let items = self.items.lock().unwrap();
        let running = items.running.iter().map(|entry| entry.view(ItemState::Running));
        running.chain(items.waiting.iter().map(|item| item.entry.view(ItemState::Waiting))).collect()
    }

    pub fn has(&self, kind: &str, source: &Path, detail: Option<&str>) -> bool {
        let source = source.display().to_string();
        self.list().iter().any(|item| {
            item.kind == kind
                && item.source.as_deref() == Some(source.as_str())
                && (detail.is_none() || item.detail.as_deref() == detail)
        })
    }

    pub fn history(&self) -> Vec<HistoryItem> {
        self.items.lock().unwrap().history.iter().cloned().collect()
    }

    fn history_clear(&self) {
        self.items.lock().unwrap().history.clear();
    }

    fn retry_spec(&self, id: u64) -> Option<TaskSpec> {
        let items = self.items.lock().unwrap();
        items.history.iter().find(|item| item.id == id).and_then(|item| item.retry.clone())
    }

    fn history_remove(&self, id: u64) {
        self.items.lock().unwrap().history.retain(|item| item.id != id);
    }

    fn record(&self, item: HistoryItem) {
        self.items.lock().unwrap().record(item);
    }

    fn cancel(&self, id: Option<u64>) {
        let mut items = self.items.lock().unwrap();
        for item in std::mem::take(&mut items.waiting) {
            if id.is_none_or(|id| item.entry.id == id) {
                let ended = item.entry.ended(&Err(CANCELLED.into()));
                items.record(ended);
            } else {
                items.waiting.push_back(item);
            }
        }
        if let Some(running) = items.running.as_ref().filter(|entry| id.is_none_or(|id| entry.id == id)) {
            running.cancel.store(true, Ordering::Relaxed);
        }
    }

    fn next(&self) -> Option<Item> {
        let mut items = self.items.lock().unwrap();
        loop {
            if items.closed {
                return None;
            }
            if let Some(item) = items.waiting.pop_front() {
                items.running = Some(item.entry.clone());
                return Some(item);
            }
            items = self.wake.wait(items).unwrap();
        }
    }

    fn finish(&self) {
        self.items.lock().unwrap().running = None;
        self.wake.notify_all();
    }
}

fn emit(app: &AppHandle) {
    let _ = app.emit_to("main", events::QUEUE, app.state::<State>().queue.list());
}

fn emit_history(app: &AppHandle) {
    let _ = app.emit_to("main", events::QUEUE_HISTORY, app.state::<State>().queue.history());
}

fn report(app: &AppHandle, ended: HistoryItem) {
    if ended.state == HistoryState::Cancelled {
        return;
    }
    let HistoryItem { id, kind, name, source, detail, output, error, .. } = ended;
    if output.is_some() {
        let _ = app.emit_to("main", events::GALLERY_CHANGED, ());
    }
    let visible = app.get_webview_window("main").is_some_and(|w| w.is_visible().unwrap_or(false));
    if visible {
        let _ = app.emit_to("main", events::QUEUE_ENDED, QueueEnded { id, kind, name, source, detail, output, error });
        return;
    }
    let vi = app.state::<State>().cfg.read().unwrap().language == "vi";
    let msg = match (error, vi) {
        (None, true) => format!("Đã xong: {name}"),
        (None, false) => format!("Finished: {name}"),
        (Some(e), true) => format!("Lỗi {name}: {e}"),
        (Some(e), false) => format!("Failed {name}: {e}"),
    };
    let _ = app.notification().builder().title("Snapr").body(msg).show();
}

pub fn write_part(
    dir: &Path,
    stem: &str,
    ext: &str,
    work: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<Option<PathBuf>, String> {
    write_part_to(dir, stem, ext, work, || capture::stem_path(dir, stem, ext))
}

pub fn write_part_to(
    dir: &Path,
    stem: &str,
    ext: &str,
    work: impl FnOnce(&Path) -> Result<(), String>,
    output: impl FnOnce() -> PathBuf,
) -> Result<Option<PathBuf>, String> {
    let part = dir.join(format!("{stem}.{ext}{PART_SUFFIX}"));
    let result = work(&part).and_then(|()| {
        let output = output();
        std::fs::rename(&part, &output).map(|()| output).map_err(|e| e.to_string())
    });
    if result.is_err() {
        let _ = std::fs::remove_file(&part);
    }
    result.map(Some)
}

fn clean_parts(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    let suffixes = PART_EXTS.map(|ext| format!(".{ext}{PART_SUFFIX}"));
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_ascii_lowercase();
        if suffixes.iter().any(|suffix| name.ends_with(suffix.as_str())) {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

pub fn spawn(app: AppHandle) {
    std::thread::spawn(move || {
        clean_parts(&app.state::<State>().cfg.read().unwrap().dir());
        while let Some(item) = app.state::<State>().queue.next() {
            emit(&app);
            let result = item.task.run(&item.entry.progress, &item.entry.cancel);
            let ended = item.entry.ended(&result);
            let queue = &app.state::<State>().queue;
            queue.record(ended.clone());
            queue.finish();
            emit(&app);
            emit_history(&app);
            report(&app, ended);
        }
    });
}

pub fn shutdown(app: &AppHandle) {
    let queue = &app.state::<State>().inner().queue;
    let mut items = queue.items.lock().unwrap();
    items.closed = true;
    items.waiting.clear();
    if let Some(running) = &items.running {
        running.cancel.store(true, Ordering::Relaxed);
    }
    queue.wake.notify_all();
    let _ = queue.wake.wait_timeout_while(items, SHUTDOWN_WAIT, |items| items.running.is_some());
}

#[derive(Deserialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskSpec {
    Gif { path: String, start_ms: u64, end_ms: Option<u64>, width: Option<u32>, fps: u32 },
    Compress { path: String, quality: String, height: Option<u32> },
    Transcribe { path: String },
    Translate {
        path: String,
        #[serde(default)]
        target: Option<String>,
    },
    Subtitle {
        path: String,
        position: Position,
        size: Size,
        #[serde(default)]
        style: Style,
        #[serde(default)]
        track: Track,
    },
}

impl TaskSpec {
    fn into_task(self, app: &AppHandle) -> Result<Box<dyn Task>, String> {
        Ok(match self {
            TaskSpec::Gif { path, start_ms, end_ms, width, fps } => {
                Box::new(GifTask::validated(app, &path, start_ms, end_ms, width, fps)?)
            }
            TaskSpec::Compress { path, quality, height } => Box::new(CompressTask::validated(app, &path, &quality, height)?),
            TaskSpec::Transcribe { path } => Box::new(TranscribeTask::validated(app, &path, Track::Original)?),
            TaskSpec::Translate { path, target } => crate::translate::task::task(app, &path, target.as_deref())?,
            TaskSpec::Subtitle { path, position, size, style, track } => {
                Box::new(SubtitleTask::validated(app, &path, position, size, &style, track)?)
            }
        })
    }
}

#[tauri::command]
pub async fn queue_add(app: AppHandle, spec: TaskSpec) -> Result<u64, String> {
    let retry = spec.clone();
    let handle = app.clone();
    let task = tauri::async_runtime::spawn_blocking(move || spec.into_task(&handle)).await.map_err(|e| e.to_string())??;
    Ok(app.state::<State>().queue.add(&app, task, Some(retry)))
}

#[tauri::command]
pub fn queue_history(state: tauri::State<State>) -> Vec<HistoryItem> {
    state.queue.history()
}

#[tauri::command]
pub fn queue_history_clear(app: AppHandle) {
    app.state::<State>().queue.history_clear();
    emit_history(&app);
}

#[tauri::command]
pub async fn queue_retry(app: AppHandle, id: u64) -> Result<u64, String> {
    let spec = app.state::<State>().queue.retry_spec(id).ok_or("nothing to retry")?;
    let retry = spec.clone();
    let handle = app.clone();
    let task = tauri::async_runtime::spawn_blocking(move || spec.into_task(&handle)).await.map_err(|e| e.to_string())??;
    let queue = &app.state::<State>().queue;
    queue.history_remove(id);
    let new_id = queue.add(&app, task, Some(retry));
    emit_history(&app);
    Ok(new_id)
}

#[tauri::command]
pub fn queue_list(state: tauri::State<State>) -> Vec<QueueItem> {
    state.queue.list()
}

#[tauri::command]
pub fn queue_cancel(app: AppHandle, id: u64) {
    app.state::<State>().queue.cancel(Some(id));
    emit(&app);
    emit_history(&app);
}

#[tauri::command]
pub fn queue_cancel_all(app: AppHandle) {
    app.state::<State>().queue.cancel(None);
    emit(&app);
    emit_history(&app);
}
