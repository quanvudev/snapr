use crate::events;
use crate::config::{CameraEffect, CameraLook, RecordMode};
use crate::overlay::Rect;
use crate::captions::{self, Language};
use crate::{audio, Action, State};
use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::webview::{PermissionKind, PermissionResponse};
use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Webview, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
    WindowEvent,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetWindowLongPtrW, HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SetWindowDisplayAffinity, SetWindowLongPtrW,
    SetWindowPos, WDA_EXCLUDEFROMCAPTURE, WS_EX_NOACTIVATE,
};

pub const PANEL: &str = "panel";
pub const CAMERA: &str = "camera";
pub const CAPTIONS: &str = "captions";
const CAPTIONS_SIZE: (f64, f64) = (640.0, 200.0);
const CAPTIONS_GAP: f64 = 12.0;
const SETUP: (f64, f64) = (360.0, 300.0);
const CAMERA_SIZE: f64 = 236.0;
const MARGIN: f64 = 24.0;
const GAP: f64 = -32.0;
const CAMERA_PAD: f64 = 28.0;
const PREVIEW_CENTRE: f64 = 120.0;
const MIC_LOST: &str = "mic_lost";
const BACKGROUND_EXTS: [&str; 5] = ["png", "jpg", "jpeg", "webp", "bmp"];

#[derive(Clone, Copy, Default, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Phase {
    #[default]
    Closed,
    Setup,
    Picking,
    Countdown,
    Recording,
    Captions,
}

#[derive(Clone, Default, Serialize)]
pub struct PanelState {
    pub phase: Phase,
    pub mic: String,
    pub camera: String,
    pub speaker: String,
    pub mode: RecordMode,
    pub denoise: bool,
    pub effect: CameraEffect,
    pub camera_image: String,
    pub look: CameraLook,
    pub started_ms: Option<u64>,
    pub paused_total_ms: u64,
    pub paused_at_ms: Option<u64>,
    pub camera_hidden: bool,
    pub warning: Option<&'static str>,
    pub captions_language: Option<Language>,
    pub captions_target: String,
    pub captions_ready: bool,
    pub translate_ready: bool,
}

#[derive(Default)]
pub struct Panel {
    state: PanelState,
    monitor: Option<audio::Capture>,
    anchor: Option<PhysicalPosition<i32>>,
}

#[derive(Serialize)]
pub struct Input {
    id: String,
    name: String,
    default: bool,
}

fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64)
}

pub fn phase(app: &AppHandle) -> Phase {
    app.state::<State>().panel.lock().unwrap().state.phase
}

#[cfg(debug_assertions)]
pub fn started_ms(app: &AppHandle) -> Option<u64> {
    app.state::<State>().panel.lock().unwrap().state.started_ms
}

pub fn choices(app: &AppHandle) -> (String, String, String) {
    let state = app.state::<State>();
    let panel = state.panel.lock().unwrap();
    (panel.state.mic.clone(), panel.state.camera.clone(), panel.state.speaker.clone())
}

fn denoise(app: &AppHandle) -> bool {
    app.state::<State>().panel.lock().unwrap().state.denoise
}

fn update(app: &AppHandle, f: impl FnOnce(&mut Panel)) {
    let shared = app.state::<State>();
    let state = {
        let mut panel = shared.panel.lock().unwrap();
        f(&mut panel);
        panel.state.clone()
    };
    let _ = app.emit(events::RECORD_PANEL, state);
}

fn work_area(app: &AppHandle) -> Option<(PhysicalPosition<i32>, PhysicalSize<u32>, f64)> {
    let cursor = app.cursor_position().ok()?;
    let monitor = app.monitor_from_point(cursor.x, cursor.y).ok().flatten().or_else(|| app.primary_monitor().ok().flatten())?;
    let area = monitor.work_area();
    Some((area.position, area.size, monitor.scale_factor()))
}

pub fn allow_camera(_: Webview, kind: PermissionKind) -> PermissionResponse {
    match kind {
        PermissionKind::Camera => PermissionResponse::Allow,
        _ => PermissionResponse::Default,
    }
}

fn build(app: &AppHandle, label: &str, query: &str, (w, h): (f64, f64), focus: bool) -> Option<WebviewWindow> {
    WebviewWindowBuilder::new(app, label, WebviewUrl::App(format!("index.html?{query}").into()))
        .title("Snapr")
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .resizable(false)
        .shadow(false)
        .focused(focus)
        .visible(false)
        .inner_size(w, h)
        .on_permission_request(allow_camera)
        .build()
        .ok()
}

fn encode(value: &str) -> String {
    value
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => (b as char).to_string(),
            _ => format!("%{b:02X}"),
        })
        .collect()
}

fn listen(app: &AppHandle, mic: &str, speaker: &str, record: bool, denoise: bool) -> Option<audio::Capture> {
    let mic = (!mic.is_empty()).then(|| mic.to_string());
    let speaker = (!speaker.is_empty() && record).then(|| speaker.to_string());
    if mic.is_none() && speaker.is_none() {
        return None;
    }
    let (level, lost) = (app.clone(), app.clone());
    audio::Capture::start(
        mic,
        speaker,
        record,
        denoise,
        move |value| {
            let _ = level.emit_to(PANEL, events::RECORD_LEVEL, value);
        },
        move || update(&lost, |p| p.state.warning = Some(MIC_LOST)),
    )
    .map_err(|e| crate::tray::tooltip(app, &format!("Snapr: {e}")))
    .ok()
}

fn swap_monitor(app: &AppHandle, monitor: Option<audio::Capture>) {
    let state = app.state::<State>();
    let old = std::mem::replace(&mut state.panel.lock().unwrap().monitor, monitor);
    drop(old);
}

fn place_panel(app: &AppHandle, window: &WebviewWindow, (w, h): (f64, f64)) {
    let _ = window.set_size(tauri::LogicalSize::new(w, h));
    let Some((pos, size, scale)) = work_area(app) else { return };
    let x = pos.x + ((size.width as f64 - w * scale) / 2.0) as i32;
    let y = pos.y + (size.height as f64 - (h + MARGIN) * scale) as i32;
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn resize_panel(window: &WebviewWindow, (w, h): (f64, f64)) {
    let (Ok(pos), Ok(old), Ok(scale)) = (window.outer_position(), window.inner_size(), window.scale_factor()) else { return };
    let (nw, nh) = ((w * scale) as i32, (h * scale) as i32);
    let x = pos.x + (old.width as i32 - nw) / 2;
    let y = pos.y + old.height as i32 - nh;
    let _ = window.set_size(tauri::LogicalSize::new(w, h));
    let _ = window.set_position(PhysicalPosition::new(x, y));
}

fn attach(app: &AppHandle) {
    let (Some(panel), Some(camera)) = (app.get_webview_window(PANEL), app.get_webview_window(CAMERA)) else { return };
    if !matches!(phase(app), Phase::Countdown | Phase::Recording) || !camera.is_visible().unwrap_or(false) {
        return;
    }
    let (Ok(cp), Ok(cs), Ok(ps), Ok(scale)) = (camera.outer_position(), camera.outer_size(), panel.outer_size(), camera.scale_factor())
    else {
        return;
    };
    let (pw, ph) = (ps.width as i32, ps.height as i32);
    let gap = (GAP * scale) as i32;
    let mut x = cp.x + (cs.width as i32 - pw) / 2;
    let mut y = cp.y + cs.height as i32 + gap;
    if let Ok(Some(monitor)) = camera.current_monitor() {
        let area = monitor.work_area();
        let (left, top) = (area.position.x, area.position.y);
        x = x.clamp(left, (left + area.size.width as i32 - pw).max(left));
        y = y.min(top + area.size.height as i32 - ph).max(top);
    }
    let _ = panel.set_position(PhysicalPosition::new(x, y));
    if let Ok(hwnd) = panel.hwnd() {
        unsafe {
            SetWindowPos(hwnd.0 as _, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
    }
}

fn build_camera(app: &AppHandle, device: &str) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(CAMERA) {
        return Some(window);
    }
    let (effect, image, look) = {
        let state = app.state::<State>();
        let panel = state.panel.lock().unwrap();
        (panel.state.effect, panel.state.camera_image.clone(), panel.state.look)
    };
    let look = serde_json::to_string(&look).unwrap_or_default();
    let effect = match effect {
        CameraEffect::None => "none",
        CameraEffect::Blur => "blur",
        CameraEffect::Image => "image",
    };
    let query = format!(
        "view=camera&device={}&effect={effect}&image={}&look={}",
        encode(device),
        encode(&image),
        encode(&look)
    );
    let window = build(app, CAMERA, &query, (CAMERA_SIZE, CAMERA_SIZE), false)?;
    let handle = app.clone();
    window.on_window_event(move |event| {
        if let WindowEvent::Moved(_) = event {
            attach(&handle);
        }
    });
    Some(window)
}

fn close_camera(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(CAMERA) {
        let _ = window.destroy();
    }
}

fn exclude_from_capture(window: &WebviewWindow) {
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            SetWindowDisplayAffinity(hwnd.0 as _, WDA_EXCLUDEFROMCAPTURE);
        }
    }
}

fn model_flags(app: &AppHandle, language: Language) -> (bool, bool) {
    (captions::ready(app, language), crate::translate::ready(app))
}

pub fn refresh_models(app: &AppHandle) {
    if phase(app) == Phase::Closed {
        return;
    }
    let language = app.state::<State>().panel.lock().unwrap().state.captions_language.unwrap_or(Language::En);
    let (captions_ready, translate_ready) = model_flags(app, language);
    update(app, |p| {
        p.state.captions_ready = captions_ready;
        p.state.translate_ready = translate_ready;
    });
}

pub fn open(app: &AppHandle) {
    open_with(app, None);
}

pub fn open_with(app: &AppHandle, mode_override: Option<RecordMode>) {
    if phase(app) != Phase::Closed {
        if phase(app) == Phase::Setup
            && let Some(mode) = mode_override
        {
            update(app, |p| p.state.mode = mode);
        }
        return;
    }
    let state = app.state::<State>();
    let (captions_language, captions_target) = {
        let cfg = state.cfg.read().unwrap();
        (cfg.captions_language(), cfg.captions_target().unwrap_or(crate::config::CAPTIONS_OFF).to_string())
    };
    let (captions_ready, translate_ready) = model_flags(app, captions_language);
    let (mic, camera, speaker, mode, denoise, effect, camera_image, look) = {
        let state = app.state::<State>();
        let cfg = state.cfg.read().unwrap();
        (
            cfg.record_mic.clone(),
            cfg.record_camera.clone(),
            cfg.record_speaker.clone(),
            cfg.record_mode,
            cfg.record_denoise,
            cfg.camera_effect(),
            cfg.record_camera_image.clone(),
            cfg.camera_look(),
        )
    };
    let inputs = audio::inputs();
    let mic = if mic.is_empty() || mic == "default" || inputs.iter().any(|d| d.id == mic) { mic } else { "default".into() };
    let speaker =
        if speaker.is_empty() || speaker == "default" || audio::outputs().iter().any(|d| d.id == speaker) { speaker } else { "default".into() };
    let mode = mode_override.unwrap_or(mode);
    let Some(window) = build(app, PANEL, "view=panel", SETUP, true) else { return };
    exclude_from_capture(&window);
    place_panel(app, &window, SETUP);
    swap_monitor(app, listen(app, &mic, "", false, denoise));
    update(app, |p| {
        p.state = PanelState {
            phase: Phase::Setup,
            mic,
            camera,
            speaker,
            mode,
            denoise,
            effect,
            camera_image,
            look,
            captions_language: Some(captions_language),
            captions_target,
            captions_ready,
            translate_ready,
            ..PanelState::default()
        };
    });
    let _ = window.show();
    let _ = window.set_focus();
}

pub fn close(app: &AppHandle) {
    for label in [PANEL, CAPTIONS] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.destroy();
        }
    }
    close_camera(app);
    swap_monitor(app, None);
    update(app, |p| p.state = PanelState::default());
}

pub fn toggle(app: &AppHandle) -> bool {
    match phase(app) {
        Phase::Closed => {
            open(app);
            true
        }
        Phase::Setup => {
            close(app);
            true
        }
        Phase::Picking => true,
        Phase::Captions => {
            captions::session::stop_session(app);
            true
        }
        Phase::Countdown | Phase::Recording => false,
    }
}

fn open_captions_window(app: &AppHandle) {
    let Some(window) = build(app, CAPTIONS, "view=captions", CAPTIONS_SIZE, false) else { return };
    exclude_from_capture(&window);
    if let Ok(hwnd) = window.hwnd() {
        unsafe {
            let style = GetWindowLongPtrW(hwnd.0 as _, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd.0 as _, GWL_EXSTYLE, style | WS_EX_NOACTIVATE as isize);
        }
    }
    if let (Some(panel), Ok(scale)) = (app.get_webview_window(PANEL), window.scale_factor())
        && let (Ok(pos), Ok(size)) = (panel.outer_position(), panel.outer_size())
    {
        let (w, h) = ((CAPTIONS_SIZE.0 * scale) as i32, (CAPTIONS_SIZE.1 * scale) as i32);
        let x = pos.x + (size.width as i32 - w) / 2;
        let y = pos.y - h - (CAPTIONS_GAP * scale) as i32;
        let _ = window.set_position(PhysicalPosition::new(x, y.max(0)));
    }
    let _ = window.show();
}

fn start_captions(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<State>();
    let (language, target) = {
        let cfg = state.cfg.read().unwrap();
        (cfg.captions_language(), cfg.captions_target())
    };
    captions::session::start(app, language, target)?;
    update(app, |p| {
        p.state.phase = Phase::Captions;
        p.state.started_ms = Some(now_ms());
        p.state.warning = None;
    });
    let handle = app.clone();
    std::thread::spawn(move || open_captions_window(&handle));
    Ok(())
}

pub fn cancelled(app: &AppHandle) {
    if phase(app) != Phase::Picking {
        return;
    }
    let (mic, _, _) = choices(app);
    swap_monitor(app, listen(app, &mic, "", false, denoise(app)));
    update(app, |p| p.state.phase = Phase::Setup);
    if let Some(window) = app.get_webview_window(PANEL) {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

pub fn picked(app: &AppHandle, (bx, by, bw, bh): Rect) {
    update(app, |p| {
        if p.state.phase == Phase::Picking {
            p.state.phase = Phase::Countdown;
        }
    });
    if let Some(window) = app.get_webview_window(PANEL) {
        let _ = window.show();
    }
    let (_, camera, _) = choices(app);
    if camera.is_empty() {
        return;
    }
    let Some(window) = build_camera(app, &camera) else { return };
    let (anchor, mode) = {
        let state = app.state::<State>();
        let panel = state.panel.lock().unwrap();
        (panel.anchor, panel.state.mode)
    };
    let scale = window.scale_factor().unwrap_or(1.0);
    let (side, pad, margin) = ((CAMERA_SIZE * scale) as i32, (CAMERA_PAD * scale) as i32, (MARGIN * scale) as i32);
    let circle = side - 2 * pad;
    let inside = |at: &PhysicalPosition<i32>| {
        let (x, y) = (at.x + pad, at.y + pad);
        x >= bx && y >= by && x + circle <= bx + bw && y + circle <= by + bh
    };
    let at = anchor.filter(|at| mode != RecordMode::Area && inside(at)).unwrap_or_else(|| {
        PhysicalPosition::new(bx + margin - pad, (by + bh - circle - margin - pad).max(by - pad))
    });
    let _ = window.set_position(at);
    let _ = window.show();
    attach(app);
}

pub fn started(app: &AppHandle) {
    update(app, |p| {
        p.state.phase = Phase::Recording;
        p.state.started_ms = Some(now_ms());
    });
}

pub fn paused(app: &AppHandle, paused: bool) {
    update(app, |p| {
        let now = now_ms();
        match (paused, p.state.paused_at_ms) {
            (true, None) => p.state.paused_at_ms = Some(now),
            (false, Some(at)) => {
                p.state.paused_total_ms += now.saturating_sub(at);
                p.state.paused_at_ms = None;
            }
            _ => {}
        }
    });
}

pub fn audio(app: &AppHandle) -> Option<audio::Capture> {
    let (mic, _, speaker) = choices(app);
    listen(app, &mic, &speaker, true, denoise(app))
}

#[tauri::command]
pub fn record_panel_state(app: AppHandle) -> PanelState {
    app.state::<State>().panel.lock().unwrap().state.clone()
}

#[tauri::command]
pub fn record_panel_fit(app: AppHandle, width: f64, height: f64) {
    if let Some(window) = app.get_webview_window(PANEL)
        && width > 0.0
        && height > 0.0
    {
        resize_panel(&window, (width, height));
        attach(&app);
    }
}

#[tauri::command]
pub fn record_inputs() -> Vec<Input> {
    audio::inputs().into_iter().map(|d| Input { id: d.id, name: d.name, default: d.default }).collect()
}

#[tauri::command]
pub async fn camera_background(path: String) -> Result<tauri::ipc::Response, String> {
    let path = std::path::PathBuf::from(path);
    let image = path
        .extension()
        .and_then(|e| e.to_str())
        .is_some_and(|e| BACKGROUND_EXTS.contains(&e.to_ascii_lowercase().as_str()));
    if !image {
        return Err("unsupported".into());
    }
    tauri::async_runtime::spawn_blocking(move || std::fs::read(&path))
        .await
        .map_err(|e| e.to_string())?
        .map(tauri::ipc::Response::new)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn record_outputs() -> Vec<Input> {
    audio::outputs().into_iter().map(|d| Input { id: d.id, name: d.name, default: d.default }).collect()
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
pub async fn record_select(
    app: AppHandle,
    mic: String,
    camera: String,
    speaker: String,
    mode: RecordMode,
    denoise: bool,
    effect: CameraEffect,
    captions_language: Language,
    captions_target: String,
) -> Result<(), String> {
    if phase(&app) != Phase::Setup {
        return Err("recording already started".into());
    }
    let (old_mic, _, _) = choices(&app);
    if old_mic != mic || self::denoise(&app) != denoise {
        swap_monitor(&app, listen(&app, &mic, "", false, denoise));
    }
    update(&app, |p| {
        p.state.mic = mic;
        p.state.camera = camera;
        p.state.speaker = speaker;
        p.state.mode = mode;
        p.state.denoise = denoise;
        p.state.effect = effect;
        p.state.captions_language = Some(captions_language);
        p.state.captions_target = captions_target;
    });
    refresh_models(&app);
    Ok(())
}

#[tauri::command]
pub fn record_start(app: AppHandle) -> Result<(), String> {
    if phase(&app) != Phase::Setup {
        return Err("not ready".into());
    }
    let (mic, camera, speaker) = choices(&app);
    let state = app.state::<State>();
    let (mode, denoise, effect, captions_language, captions_target) = {
        let panel = state.panel.lock().unwrap();
        (panel.state.mode, panel.state.denoise, panel.state.effect, panel.state.captions_language, panel.state.captions_target.clone())
    };
    {
        let mut cfg = state.cfg.write().unwrap();
        cfg.record_denoise = denoise;
        cfg.record_camera_effect = effect;
        cfg.record_mic = mic;
        cfg.record_camera = camera;
        cfg.record_speaker = speaker;
        cfg.record_mode = mode;
        if let Some(language) = captions_language {
            cfg.captions_language = language.code().into();
            cfg.captions_target = captions_target;
        }
        cfg.save(&state.path).map_err(|e| e.to_string())?;
    }
    swap_monitor(&app, None);
    if mode == RecordMode::Captions {
        return start_captions(&app).inspect_err(|_| {
            let (mic, _, _) = choices(&app);
            swap_monitor(&app, listen(&app, &mic, "", false, denoise));
        });
    }
    let anchor = app.get_webview_window(PANEL).and_then(|window| {
        let (pos, size, scale) = (window.outer_position().ok()?, window.inner_size().ok()?, window.scale_factor().ok()?);
        let half = CAMERA_SIZE * scale / 2.0;
        Some(PhysicalPosition::new(
            pos.x + (size.width as f64 / 2.0 - half) as i32,
            pos.y + (PREVIEW_CENTRE * scale - half) as i32,
        ))
    });
    update(&app, |p| {
        p.anchor = anchor;
        p.state.phase = Phase::Picking;
        p.state.warning = None;
    });
    for label in [PANEL, CAMERA] {
        if let Some(window) = app.get_webview_window(label) {
            let _ = window.hide();
        }
    }
    let _ = state.proxy.lock().unwrap().send_event(Action::RecordPick(mode));
    Ok(())
}

#[tauri::command]
pub fn record_close(app: AppHandle) {
    if phase(&app) == Phase::Setup {
        close(&app);
    }
}

#[tauri::command]
pub fn record_pause(app: AppHandle) {
    crate::recording::pause_recording(&app, true);
}

#[tauri::command]
pub fn record_resume(app: AppHandle) {
    crate::recording::pause_recording(&app, false);
}

#[tauri::command]
pub fn open_models(app: AppHandle) {
    crate::settings::show_settings(&app, "models");
}

#[tauri::command]
pub fn record_stop(app: AppHandle) {
    if captions::session::running(&app) {
        captions::session::stop_session(&app);
        return;
    }
    if !crate::recording::stop_recording(&app) {
        crate::hotkeys::trigger(&app, &Action::CancelCountdown);
    }
}

#[tauri::command]
pub fn record_discard(app: AppHandle) {
    let state = app.state::<State>();
    let running = state.recording.lock().unwrap().is_some();
    if running {
        state.discard.store(true, std::sync::atomic::Ordering::Relaxed);
        if !crate::recording::stop_recording(&app) {
            state.discard.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    } else {
        crate::hotkeys::trigger(&app, &Action::CancelCountdown);
    }
}

#[tauri::command]
pub fn record_camera_toggle(app: AppHandle) {
    if !matches!(phase(&app), Phase::Countdown | Phase::Recording) {
        return;
    }
    let Some(window) = app.get_webview_window(CAMERA) else { return };
    let mut hidden = false;
    update(&app, |p| {
        p.state.camera_hidden = !p.state.camera_hidden;
        hidden = p.state.camera_hidden;
    });
    if hidden {
        let _ = window.hide();
    } else {
        let _ = window.show();
        attach(&app);
    }
}

#[tauri::command]
pub fn open_camera_settings() {
    let _ = std::process::Command::new("explorer").arg("ms-settings:privacy-webcam").spawn();
}
