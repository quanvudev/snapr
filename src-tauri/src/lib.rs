mod capture;
mod config;
mod draw;
mod gallery;
mod mouse;
mod ocr;
mod overlay;
mod recorder;
mod ring;
mod settings;
mod toast;
mod toolbar;

use config::Config;
use image::RgbaImage;
use overlay::{Outcome, Overlay, Pick, Shot};
use recorder::{Recorder, Target};
use ring::{Entry, Ring};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::sync::{Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Emitter, LogicalSize, Manager, WindowEvent};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};
use tauri_plugin_notification::NotificationExt;
use toast::Toast;
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::WindowId;
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows_sys::Win32::Graphics::Gdi::{MONITOR_DEFAULTTONEAREST, MonitorFromPoint};
use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetCursorPos};

#[derive(Clone)]
enum Action {
    Fullscreen,
    Pick(Pick),
    Text,
    Saved(RgbaImage, PathBuf),
    Record,
    Draw,
    Ring,
    Recorded(RgbaImage, PathBuf),
    RecordFailed(String),
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Shot,
    Record,
    Text,
}

enum Job {
    Save(Shot),
    Text(Shot),
}

pub struct State {
    path: PathBuf,
    cfg: RwLock<Config>,
    status: Mutex<String>,
    probe: Mutex<HashMap<String, bool>>,
    ocr: AtomicBool,
    proxy: Mutex<EventLoopProxy<Action>>,
    recording: Mutex<Option<Recorder>>,
    stop: MenuItem<tauri::Wry>,
}

fn tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(text));
    }
}

fn refresh(app: &AppHandle) {
    let state = app.state::<State>();
    let text = if state.recording.lock().unwrap().is_some() {
        "Snapr: recording…".to_string()
    } else {
        state.status.lock().unwrap().clone()
    };
    tooltip(app, &text);
}

fn ended(app: &AppHandle) {
    let state = app.state::<State>();
    let recorder = state.recording.lock().unwrap().take();
    drop(recorder);
    let _ = state.stop.set_enabled(false);
    refresh(app);
}

fn stop_recording(app: &AppHandle) -> bool {
    let recorder = app.state::<State>().recording.lock().unwrap().take();
    recorder.map(Recorder::stop).is_some()
}

fn trigger(app: &AppHandle, action: &Action) {
    if matches!(action, Action::Record) && stop_recording(app) {
        return;
    }
    let _ = app.state::<State>().proxy.lock().unwrap().send_event(action.clone());
}

fn record(app: &AppHandle, target: Target) {
    let state = app.state::<State>();
    let (dir, fps) = {
        let cfg = state.cfg.read().unwrap();
        (cfg.dir(), cfg.fps())
    };
    let proxy = state.proxy.lock().unwrap().clone();
    let started = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|_| Recorder::start(proxy, target, capture::unique_path(&dir, "mp4"), fps));
    match started {
        Ok(recorder) => {
            *state.recording.lock().unwrap() = Some(recorder);
            let _ = state.stop.set_enabled(true);
            refresh(app);
        }
        Err(e) => tooltip(app, &format!("Snapr: {e}")),
    }
}

fn is_desktop(hwnd: isize) -> bool {
    let mut buf = [0u16; 32];
    let len = unsafe { GetClassNameW(hwnd as _, buf.as_mut_ptr(), buf.len() as i32) };
    let name = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
    name == "Progman" || name == "WorkerW"
}

fn cursor_monitor_handle() -> isize {
    let mut p = unsafe { std::mem::zeroed() };
    unsafe {
        GetCursorPos(&mut p);
        MonitorFromPoint(p, MONITOR_DEFAULTTONEAREST) as isize
    }
}

fn apply(app: &AppHandle) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    let state = app.state::<State>();
    let ocr = ocr::available();
    state.ocr.store(ocr, Ordering::Relaxed);
    let (combos, active, ring_enabled) = {
        let cfg = state.cfg.read().unwrap();
        (cfg.hotkeys.combos(), cfg.text_capture && ocr, cfg.ring.enabled)
    };
    let actions = [
        Action::Fullscreen,
        Action::Pick(Pick::Window),
        Action::Pick(Pick::Area),
        Action::Text,
        Action::Record,
        Action::Draw,
        Action::Ring,
    ];
    mouse::bind(None);
    let mut failed = Vec::new();
    for (i, (text, action)) in combos.iter().cloned().zip(actions).enumerate() {
        if config::skipped(i, &combos, active, ring_enabled) {
            continue;
        }
        if i == config::RING_SLOT && config::is_mouse(&text) {
            mouse::bind(mouse::Binding::parse(&text));
            continue;
        }
        let ok = config::parse_hotkey(&text).is_ok_and(|key| {
            shortcuts
                .on_shortcut(key, move |app, _, e| {
                    if e.state == ShortcutState::Pressed {
                        trigger(app, &action);
                    }
                })
                .is_ok()
        });
        if !ok {
            failed.push(text);
        }
    }
    let status = if failed.is_empty() {
        "Snapr".to_string()
    } else {
        format!("Snapr: hotkey in use: {}", failed.join(", "))
    };
    *state.status.lock().unwrap() = status;
    refresh(app);
}

struct Snap {
    app: AppHandle,
    overlay: Option<Overlay>,
    ring: Option<Ring>,
    toast: Option<Toast>,
    jobs: Sender<Job>,
    mode: Mode,
    style: draw::Style,
}

fn work(app: &AppHandle, proxy: &EventLoopProxy<Action>, backgrounds: &mut capture::BackgroundCache, shot: Shot) {
    let Some(img) = (match shot {
        Shot::Ready(img) => Some(img),
        Shot::Window { hwnd, border, frame, rect } => capture::window(hwnd, border)
            .filter(|img| img.width() > 0 && img.height() > 0)
            .or_else(|| overlay::crop(&frame, rect)),
    }) else {
        return;
    };
    let cfg = app.state::<State>().cfg.read().unwrap().clone();
    let mut bg_error = None;
    let mut img = img;
    let img = if cfg.padding.enabled && cfg.padding.size > 0 {
        if cfg.padding.round {
            capture::mask_corners(&mut img, cfg.padding.radius as f32);
        }
        let bg = backgrounds.get(&cfg.padding.image).map_err(|e| bg_error = Some(e)).ok().flatten();
        capture::pad(&img, cfg.padding.size, bg)
    } else {
        img
    };
    match capture::write(&img, &cfg) {
        Ok(path) => {
            refresh(app);
            if let Some(e) = bg_error {
                tooltip(app, &format!("Snapr: {e}"));
            }
            let _ = proxy.send_event(Action::Saved(img.clone(), path));
            if cfg.copy_to_clipboard
                && let Err(e) = capture::copy(&img)
            {
                tooltip(app, &format!("Snapr: {e}"));
            }
        }
        Err(e) => tooltip(app, &format!("Snapr: {e}")),
    }
}

fn read_text(app: &AppHandle, shot: Shot) {
    let Shot::Ready(img) = shot else { return };
    let vi = app.state::<State>().cfg.read().unwrap().language == "vi";
    let failed = |e: String| if vi { format!("Lỗi nhận dạng chữ: {e}") } else { format!("Text capture failed: {e}") };
    let msg = match ocr::extract(&img) {
        Ok(text) if text.trim().is_empty() => if vi { "Không tìm thấy chữ" } else { "No text found" }.to_string(),
        Ok(text) => {
            let text = text.trim();
            let n = text.lines().count();
            match arboard::Clipboard::new().and_then(|mut c| c.set_text(text)) {
                Ok(()) if vi => format!("Đã sao chép {n} dòng"),
                Ok(()) => format!("Copied {n} lines"),
                Err(e) => failed(e.to_string()),
            }
        }
        Err(e) => failed(e),
    };
    let _ = app.notification().builder().title("Snapr").body(msg).show();
}

impl ApplicationHandler<Action> for Snap {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.toast.as_ref().is_some_and(|t| t.expired()) {
            self.toast = None;
        }
        if let Some(ring) = self.ring.as_mut() {
            ring.tick();
        }
        if let Some(entry) = self.ring.take_if(|ring| ring.finished()).and_then(Ring::into_picked) {
            entry.run(&self.app);
        }
        let deadline = [self.toast.as_ref().and_then(|t| t.deadline()), self.ring.as_ref().map(|r| r.deadline())]
            .into_iter()
            .flatten()
            .min();
        let flow = match deadline {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        };
        event_loop.set_control_flow(flow);
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, id: WindowId, event: winit::event::WindowEvent) {
        if let Some(ring) = self.ring.as_mut().filter(|r| r.id() == id) {
            match ring.event(event) {
                ring::Outcome::Pending => {}
                ring::Outcome::Cancel => ring.begin_close(None),
                ring::Outcome::Pick(entry) => ring.begin_close(Some(entry)),
            }
            return;
        }
        if let Some(toast) = self.toast.as_mut().filter(|t| t.id() == id) {
            if toast.event(event) {
                self.toast = None;
            }
            return;
        }
        let Some(overlay) = self.overlay.as_mut().filter(|o| o.id() == id) else { return };
        let click = matches!(
            event,
            winit::event::WindowEvent::MouseInput {
                state: winit::event::ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            }
        );
        if self.mode == Mode::Record && click && overlay.hovered().is_none_or(is_desktop) {
            self.overlay = None;
            self.mode = Mode::Shot;
            record(&self.app, Target::Monitor(cursor_monitor_handle()));
            return;
        }
        let outcome = overlay.event(event);
        if !matches!(outcome, Outcome::Pending)
            && let Some(style) = overlay.style()
        {
            self.style = style;
        }
        match outcome {
            Outcome::Pending => {}
            Outcome::Cancel => {
                self.overlay = None;
                self.mode = Mode::Shot;
            }
            Outcome::Save(shot) => {
                self.overlay = None;
                match (std::mem::replace(&mut self.mode, Mode::Shot), shot) {
                    (Mode::Record, Shot::Window { hwnd, .. }) => record(&self.app, Target::Window(hwnd)),
                    (Mode::Text, shot) => {
                        let _ = self.jobs.send(Job::Text(shot));
                    }
                    (_, shot) => {
                        let _ = self.jobs.send(Job::Save(shot));
                    }
                }
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, action: Action) {
        match action {
            Action::Fullscreen => match capture::fullscreen() {
                Ok(img) => {
                    let _ = self.jobs.send(Job::Save(Shot::Ready(img)));
                }
                Err(e) => tooltip(&self.app, &format!("Snapr: {e}")),
            },
            Action::Pick(pick) => self.pick(event_loop, pick, Mode::Shot, false),
            Action::Text => self.pick(event_loop, Pick::Area, Mode::Text, false),
            Action::Record => self.pick(event_loop, Pick::Window, Mode::Record, false),
            Action::Draw => self.pick(event_loop, Pick::Area, Mode::Shot, true),
            Action::Ring => self.open_ring(event_loop),
            Action::Saved(img, path) => {
                let _ = self.app.emit_to("main", "gallery-changed", ());
                self.preview(event_loop, &img, path);
            }
            Action::Recorded(img, path) => {
                ended(&self.app);
                let _ = self.app.emit_to("main", "gallery-changed", ());
                self.preview(event_loop, &img, path);
            }
            Action::RecordFailed(e) => {
                ended(&self.app);
                tooltip(&self.app, &format!("Snapr: {e}"));
            }
        }
    }
}

impl Snap {
    fn pick(&mut self, event_loop: &ActiveEventLoop, pick: Pick, mode: Mode, draw: bool) {
        if self.overlay.is_some() {
            return;
        }
        self.toast = None;
        let vi = self.app.state::<State>().cfg.read().unwrap().language == "vi";
        self.overlay = Overlay::open(event_loop, pick, draw.then_some(self.style), vi);
        self.mode = if self.overlay.is_some() { mode } else { Mode::Shot };
        if self.overlay.is_none() {
            tooltip(&self.app, "Snapr: capture failed");
        }
    }

    fn open_ring(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ring) = self.ring.as_mut() {
            ring.begin_close(None);
            return;
        }
        if self.overlay.is_some() {
            return;
        }
        self.toast = None;
        let state = self.app.state::<State>();
        let (text, vi, items, enabled, keys) = {
            let cfg = state.cfg.read().unwrap();
            (cfg.text_capture && state.ocr.load(Ordering::Relaxed), cfg.language == "vi", cfg.ring_items(), cfg.ring.enabled, cfg.ring.keys)
        };
        if !enabled {
            return;
        }
        let recording = state.recording.lock().unwrap().is_some();
        let entries: Vec<Entry> = items.iter().map(Entry::from).filter(|entry| entry.available(text)).collect();
        if entries.is_empty() {
            return;
        }
        self.ring = Ring::open(event_loop, entries, recording, vi, keys);
        if self.ring.is_none() {
            tooltip(&self.app, "Snapr: ring failed");
        }
    }

    fn preview(&mut self, event_loop: &ActiveEventLoop, img: &RgbaImage, path: PathBuf) {
        self.toast = None;
        let secs = self.app.state::<State>().cfg.read().unwrap().preview_seconds;
        if secs > 0 && self.overlay.is_none() {
            self.toast = Toast::open(event_loop, img, path, secs);
        }
    }
}

fn spawn_capture(app: AppHandle) -> EventLoopProxy<Action> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let event_loop = EventLoop::<Action>::with_user_event()
            .with_any_thread(true)
            .build()
            .expect("capture event loop");
        let proxy = event_loop.create_proxy();
        let _ = tx.send(proxy.clone());
        let (jobs, queue) = std::sync::mpsc::channel();
        let worker = app.clone();
        std::thread::spawn(move || {
            let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
            let mut backgrounds = capture::BackgroundCache::default();
            for job in queue {
                match job {
                    Job::Save(shot) => work(&worker, &proxy, &mut backgrounds, shot),
                    Job::Text(shot) => read_text(&worker, shot),
                }
            }
        });
        let _ = event_loop.run_app(&mut Snap { app, overlay: None, ring: None, toast: None, jobs, mode: Mode::Shot, style: draw::Style::default() });
    });
    rx.recv().expect("capture event loop")
}

fn show_splash(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.set_size(LogicalSize::new(480.0, 300.0));
        let _ = w.center();
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub(crate) fn show_settings(app: &AppHandle, section: &str) {
    if ocr::available() != app.state::<State>().ocr.load(Ordering::Relaxed) {
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
    let _ = app.emit_to("main", "settings-open", section);
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let gallery = MenuItem::with_id(app, "gallery", "Gallery", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open folder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    let stop = &app.state::<State>().stop;
    let menu = Menu::with_items(app, &[&gallery, &settings, &open, stop, &quit])?;
    let mut tray = TrayIconBuilder::with_id("main").tooltip("Snapr").menu(&menu);
    if let Some(icon) = app.default_window_icon() {
        tray = tray.icon(icon.clone());
    }
    tray.on_menu_event(|app, e| match e.id().as_ref() {
        "gallery" => show_settings(app, "gallery"),
        "settings" => show_settings(app, "settings"),
        "open" => {
            let dir = app.state::<State>().cfg.read().unwrap().dir();
            let _ = std::process::Command::new("explorer").arg(dir).spawn();
        }
        "stop" => {
            stop_recording(app);
        }
        "quit" => {
            stop_recording(app);
            app.exit(0);
        }
        _ => {}
    })
    .build(app)?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .register_asynchronous_uri_scheme_protocol("thumb", gallery::thumb_scheme())
        .register_asynchronous_uri_scheme_protocol("media", gallery::media_scheme)
        .setup(|app| {
            let handle = app.handle().clone();
            let path = app.path().app_config_dir()?.join("config.toml");
            let first_run = !path.exists();
            let cfg = Config::load(&path);
            let proxy = spawn_capture(handle.clone());
            mouse::start(proxy.clone());
            let stop = MenuItem::with_id(app, "stop", "Stop recording", false, None::<&str>)?;
            app.manage(State {
                path,
                cfg: RwLock::new(cfg),
                status: Mutex::new("Snapr".into()),
                probe: Mutex::new(HashMap::new()),
                ocr: AtomicBool::new(false),
                proxy: Mutex::new(proxy),
                recording: Mutex::new(None),
                stop,
            });
            build_tray(&handle)?;
            apply(&handle);
            if first_run {
                show_splash(&handle);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                apply(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            settings::load,
            settings::defaults,
            settings::resolve_dir,
            settings::check,
            settings::save,
            settings::set_listening,
            settings::open_folder,
            settings::open_config_file,
            settings::open_settings,
            settings::open_language_settings,
            settings::ring_icon,
            settings::list_apps,
            gallery::gallery_list,
            gallery::gallery_open,
            gallery::gallery_reveal,
            gallery::gallery_copy,
            gallery::gallery_delete
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
