mod audio;
mod panel;
mod capture;
mod config;
mod draw;
mod events;
mod fetch;
mod frame;
mod gallery;
mod gif;
mod gpu;
mod hotkeys;
mod jobs;
mod mouse;
mod ocr;
mod onnx;
mod overlay;
mod paddle;
mod pin;
mod queue;
mod recorder;
mod recording;
mod redact;
mod ring;
mod settings;
mod snap;
mod subtitle;
mod toast;
mod toolbar;
mod tray;
mod transcript;
mod translate;
mod update;
mod video;
mod whisper;

use config::Config;
use image::RgbaImage;
use overlay::Pick;
use recorder::Recorder;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use tauri::menu::{Menu, MenuItem};
use tauri::{Manager, WindowEvent};
use winit::event_loop::EventLoopProxy;

#[derive(Clone)]
enum Action {
    Fullscreen,
    Pick(Pick),
    Text,
    Saved(RgbaImage, PathBuf),
    Record,
    Draw,
    Ring,
    Pin(PathBuf),
    PinMany(Vec<PathBuf>),
    Recorded(RgbaImage, PathBuf, bool),
    RecordFailed(String),
    CancelCountdown,
    RecordPick(config::RecordMode),
    Redactions { token: u64, rects: Vec<ocr::Bounds> },
}

pub struct State {
    path: PathBuf,
    cfg: RwLock<Config>,
    status: Mutex<String>,
    probe: Mutex<HashMap<String, bool>>,
    ocr: AtomicBool,
    proxy: Mutex<EventLoopProxy<Action>>,
    recording: Mutex<Option<Recorder>>,
    download: Mutex<Option<paddle::Download>>,
    whisper_download: Mutex<Option<whisper::Download>>,
    translate_download: Mutex<Option<translate::Download>>,
    runtime_download: Mutex<Option<onnx::Download>>,
    counting: AtomicBool,
    discard: AtomicBool,
    update: Mutex<update::Updater>,
    menu: Menu<tauri::Wry>,
    stop: MenuItem<tauri::Wry>,
    pause: MenuItem<tauri::Wry>,
    upgrade: MenuItem<tauri::Wry>,
    panel: Mutex<panel::Panel>,
    capture: Mutex<Option<audio::Capture>>,
    queue: queue::Queue,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    update::handoff();
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
            let proxy = snap::spawn_capture(handle.clone());
            mouse::start(proxy.clone());
            let stop = MenuItem::with_id(app, "stop", "Stop recording", false, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pause recording", false, None::<&str>)?;
            let upgrade = MenuItem::with_id(app, "update", "Update", true, None::<&str>)?;
            let menu = tray::tray_menu(&handle, &stop, &pause)?;
            app.manage(State {
                path,
                cfg: RwLock::new(cfg),
                status: Mutex::new("Snapr".into()),
                probe: Mutex::new(HashMap::new()),
                ocr: AtomicBool::new(false),
                proxy: Mutex::new(proxy),
                recording: Mutex::new(None),
                download: Mutex::new(None),
                whisper_download: Mutex::new(None),
                translate_download: Mutex::new(None),
                runtime_download: Mutex::new(None),
                counting: AtomicBool::new(false),
                discard: AtomicBool::new(false),
                update: Mutex::new(update::Updater::default()),
                menu,
                stop,
                pause,
                upgrade,
                panel: Mutex::new(panel::Panel::default()),
                capture: Mutex::new(None),
                queue: queue::Queue::default(),
            });
            queue::spawn(handle.clone());
            tray::build_tray(&handle)?;
            hotkeys::apply(&handle);
            update::start(&handle);
            if first_run {
                settings::show_splash(&handle);
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event
                && window.label() != "main"
            {
                api.prevent_close();
                if window.label() == panel::PANEL && panel::phase(window.app_handle()) == panel::Phase::Setup {
                    panel::close(window.app_handle());
                }
            } else if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
                hotkeys::apply(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            panel::record_panel_state,
            panel::record_inputs,
            panel::record_outputs,
            panel::record_panel_fit,
            panel::record_select,
            panel::record_start,
            panel::record_close,
            panel::record_pause,
            panel::record_resume,
            panel::record_stop,
            panel::record_discard,
            panel::record_camera_toggle,
            panel::open_camera_settings,
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
            gallery::gallery_copy_files,
            gallery::gallery_pin,
            gallery::gallery_pin_many,
            gallery::gallery_text,
            gallery::gallery_delete,
            gallery::gallery_delete_many,
            paddle::paddle_status,
            paddle::paddle_download,
            paddle::paddle_cancel,
            paddle::paddle_delete,
            onnx::acceleration_status,
            onnx::acceleration_download,
            onnx::acceleration_cancel,
            onnx::acceleration_delete,
            onnx::acceleration_restart,
            whisper::whisper_status,
            whisper::whisper_download,
            redact::redact_validate,
            translate::translate_status,
            translate::translate_download,
            translate::translate_cancel,
            translate::translate_delete,
            whisper::whisper_cancel,
            whisper::whisper_delete,
            transcript::transcript_detect,
            transcript::transcript_read,
            transcript::transcript_save,
            update::update_status,
            update::update_check,
            update::update_install,
            queue::queue_add,
            queue::queue_history,
            queue::queue_history_clear,
            queue::queue_retry,
            queue::queue_list,
            queue::queue_cancel,
            queue::queue_cancel_all,
            video::compress_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
