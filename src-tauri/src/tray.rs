use crate::recorder::Recorder;
use crate::recording::{pause_recording, stop_recording};
use crate::settings::show_settings;
use crate::{Action, State, queue, update};
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

pub(crate) fn tooltip(app: &AppHandle, text: &str) {
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_tooltip(Some(text));
    }
}

pub(crate) fn refresh(app: &AppHandle) {
    let state = app.state::<State>();
    let paused = state.recording.lock().unwrap().as_ref().map(Recorder::paused);
    let text = if let Some(paused) = paused {
        if paused { "Snapr: paused" } else { "Snapr: recording…" }.to_string()
    } else {
        state.status.lock().unwrap().clone()
    };
    tooltip(app, &text);
}

pub(crate) fn tray_menu(app: &AppHandle, stop: &MenuItem<tauri::Wry>, pause: &MenuItem<tauri::Wry>) -> tauri::Result<Menu<tauri::Wry>> {
    let gallery = MenuItem::with_id(app, "gallery", "Gallery", true, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Settings…", true, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open folder", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
    Menu::with_items(app, &[&gallery, &settings, &open, stop, pause, &quit])
}

pub(crate) fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = app.state::<State>().menu.clone();
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
            if !stop_recording(app) {
                let _ = app.state::<State>().proxy.lock().unwrap().send_event(Action::CancelCountdown);
            }
        }
        "pause" => {
            let paused = app.state::<State>().recording.lock().unwrap().as_ref().is_some_and(Recorder::paused);
            pause_recording(app, !paused);
        }
        "update" => update::from_tray(app),
        "quit" => {
            stop_recording(app);
            queue::shutdown(app);
            app.exit(0);
        }
        _ => {}
    })
    .build(app)?;
    Ok(())
}
