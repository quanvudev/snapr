# Interface contract (implementation coordination)

Shared by the parallel workers. Change it only through the coordinator.

## File ownership

| Owner | Files |
|---|---|
| `audio` agent | `src-tauri/src/audio.rs` (new), `src-tauri/src/recorder.rs`, `src-tauri/src/config.rs` |
| `panel` agent | `src/**` (frontend only) |
| coordinator | `src-tauri/src/lib.rs`, `src-tauri/src/panel.rs` (new), `Cargo.toml`, `tauri.conf.json`, `capabilities/`, `CLAUDE.md` |

`cpal = "0.18"` is already in `Cargo.toml`. Do not touch it. The camera permission is granted with Tauri's `WebviewWindowBuilder::on_permission_request`, so `webview2-com` is not used.

## audio.rs (Rust)

```rust
pub struct Device { pub id: String, pub name: String, pub default: bool }
pub fn inputs() -> Vec<Device>;                 // input devices, id = cpal device name

pub type Chunk = Vec<i16>;                      // interleaved stereo, 48 kHz, 480 frames (10 ms) per chunk

pub struct Capture { /* streams + mixer thread; Drop stops everything */ }
impl Capture {
    // mic: None = no microphone, Some("default") = Windows default input, Some(id) = that device
    // record = false: level-only (no chunks are produced)
    // level: called ~15 times/s with mic RMS in 0.0..=1.0 (0.0 when no mic)
    // lost: called once when the mic stream errors / disappears
    pub fn start(
        mic: Option<String>,
        system: bool,
        record: bool,
        level: impl Fn(f32) + Send + 'static,
        lost: impl Fn() + Send + 'static,
    ) -> Result<Capture, String>;
    pub fn chunks(&mut self) -> Option<std::sync::mpsc::Receiver<Chunk>>; // Some once when record = true
    pub fn has_audio(&self) -> bool;            // mic or system active
}
```

## recorder.rs (Rust)

```rust
pub(crate) fn start(
    proxy: EventLoopProxy<Action>, target: Target, path: PathBuf, fps: u32,
    audio: Option<std::sync::mpsc::Receiver<audio::Chunk>>,   // None = no audio track
) -> Result<Recorder, String>;
pub(crate) fn pause(&self); pub(crate) fn resume(&self); pub(crate) fn paused(&self) -> bool;
pub(crate) fn stop(self);                        // unchanged semantics
```

## config.rs

```rust
#[serde(default = "default_mic")] pub record_mic: String,     // "default" | "" (none) | device id
#[serde(default)] pub record_camera: String,                    // "" = off | WebView2 deviceId
#[serde(default)] pub record_system_audio: bool,
```

The same fields go in `lib/api.ts` `Config` (snake_case).

## Tauri commands (implemented by the coordinator in lib.rs / panel.rs)

All return `Result<_, String>`. JS argument names are camelCase.

| Command | Args | Returns |
|---|---|---|
| `record_panel_state` | - | `PanelState` |
| `record_inputs` | - | `{ id, name, default }[]` |
| `record_select` | `{ mic: string, camera: string, systemAudio: boolean }` | - (setup only: restarts the level monitor, shows/hides/switches the camera window) |
| `record_start` | - | - (saves the choices, hides panel + camera, opens the picker) |
| `record_close` | - | - (setup only) |
| `record_pause` / `record_resume` / `record_stop` | - | - |
| `record_camera_toggle` | - | - (recording only: hide/show the webcam) |
| `open_camera_settings` | - | - (`ms-settings:privacy-webcam`) |

```ts
type PanelState = {
  phase: "closed" | "setup" | "picking" | "countdown" | "recording";
  mic: string; camera: string; system_audio: boolean;          // current choices
  started_ms: number | null;                                   // epoch ms when recording started
  paused_total_ms: number; paused_at_ms: number | null;        // timer = now - started - paused_total - (paused ? now - paused_at : 0)
  camera_hidden: boolean;
  warning: "mic_lost" | null;
};
```

Events (to `panel` and `camera` windows):
- `record-panel`: `PanelState`, emitted on every change.
- `record-level`: `number` (0..1).

## Windows

- `panel`: `index.html?view=panel`. It is resized by the backend per phase: setup is 340x300 logical, compact is 440x52. The frontend must lay out to fill the window (transparent body, rounded panel, `data-tauri-drag-region` on the empty areas).
- `camera`: `index.html?view=camera&device=<deviceId>`, 180x180, a round video. It follows `record-panel.camera` to switch devices. The whole window is a drag region.
