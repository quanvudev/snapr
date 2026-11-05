use crate::overlay::Pick;
use crate::recording::stop_recording;
use crate::tray::refresh;
use crate::{Action, State, config, mouse, ocr, paddle, panel};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

pub(crate) fn trigger(app: &AppHandle, action: &Action) {
    if matches!(action, Action::Record) {
        if stop_recording(app) {
            return;
        }
        if !app.state::<State>().counting.load(Ordering::Relaxed) {
            let handle = app.clone();
            let _ = app.run_on_main_thread(move || {
                panel::toggle(&handle);
            });
            return;
        }
    }
    let _ = app.state::<State>().proxy.lock().unwrap().send_event(action.clone());
}

pub(crate) fn ocr_available(app: &AppHandle) -> bool {
    let paddle = paddle::selected(app, &app.state::<State>().cfg.read().unwrap());
    ocr::available(paddle.as_ref())
}

pub(crate) fn apply(app: &AppHandle) {
    let shortcuts = app.global_shortcut();
    let _ = shortcuts.unregister_all();
    let state = app.state::<State>();
    let ocr = ocr_available(app);
    state.ocr.store(ocr, Ordering::Relaxed);
    let (combos, active, ring_enabled) = {
        let cfg = state.cfg.read().unwrap();
        paddle::unload_others(&cfg);
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
