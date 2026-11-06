use crate::recorder::{Recorder, Target};
use crate::tray::{refresh, tooltip};
use crate::{State, audio, capture, panel};
use tauri::AppHandle;
use tauri::Manager;

pub(crate) fn ended(app: &AppHandle) {
    let state = app.state::<State>();
    let recorder = state.recording.lock().unwrap().take();
    drop(recorder);
    let capture = state.capture.lock().unwrap().take();
    drop(capture);
    let _ = state.stop.set_enabled(false);
    let _ = state.pause.set_enabled(false);
    let _ = state.pause.set_text("Pause recording");
    refresh(app);
    close_panel(app);
}

fn close_panel(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || panel::close(&handle));
}

pub(crate) fn pause_recording(app: &AppHandle, pause: bool) {
    let state = app.state::<State>();
    let done = state.recording.lock().unwrap().as_ref().map(|r| if pause { r.pause() } else { r.resume() }).is_some();
    if !done {
        return;
    }
    panel::paused(app, pause);
    let _ = state.pause.set_text(if pause { "Resume recording" } else { "Pause recording" });
    refresh(app);
}

#[cfg(debug_assertions)]
fn stop_after_limit(app: &AppHandle) {
    let Some(secs) = std::env::var("SNAPR_RECORD_SECS").ok().and_then(|v| v.parse::<u64>().ok()).filter(|&s| s > 0) else {
        return;
    };
    let started = panel::started_ms(app);
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_secs(secs));
        if started.is_some() && panel::started_ms(&app) == started {
            stop_recording(&app);
        }
    });
}

pub(crate) fn stop_recording(app: &AppHandle) -> bool {
    let recorder = app.state::<State>().recording.lock().unwrap().take();
    recorder.map(Recorder::stop).is_some()
}

pub(crate) fn record(app: &AppHandle, target: Target) -> bool {
    let state = app.state::<State>();
    let (dir, fps, quality) = {
        let cfg = state.cfg.read().unwrap();
        (crate::library::videos(&cfg), cfg.fps(), cfg.quality())
    };
    let proxy = state.proxy.lock().unwrap().clone();
    let mut capture = panel::audio(app);
    let chunks = capture.as_mut().filter(|c| c.has_audio()).and_then(audio::Capture::chunks);
    let started = std::fs::create_dir_all(&dir)
        .map_err(|e| e.to_string())
        .and_then(|_| Recorder::start(proxy, target, capture::unique_path(&dir, "mp4"), fps, quality, chunks));
    match started {
        Ok(recorder) => {
            *state.recording.lock().unwrap() = Some(recorder);
            *state.capture.lock().unwrap() = capture;
            let _ = state.stop.set_enabled(true);
            let _ = state.pause.set_enabled(true);
            panel::started(app);
            refresh(app);
            #[cfg(debug_assertions)]
            stop_after_limit(app);
            true
        }
        Err(e) => {
            tooltip(app, &format!("Snapr: {e}"));
            close_panel(app);
            false
        }
    }
}
