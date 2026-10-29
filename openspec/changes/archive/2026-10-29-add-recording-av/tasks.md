# Tasks

No automated tests exist. Each task is verified with `cargo clippy` (in `src-tauri/`), `bunx tsc --noEmit` and a manual check in `bun run tauri dev`. Groups 2, 3 and 4 can run in parallel once group 1 passes. Group 5 joins them.

## 1. Spike: webview camera

- [ ] 1.1 Prove the webview camera path:
  - open a throwaway transparent, frameless, always-on-top `camera` window that calls `getUserMedia({ video: true })`;
  - grant `Camera` from the host through `with_webview` + `PermissionRequested`.
  - Verify: the image shows with no permission prompt, `enumerateDevices()` returns camera labels, and a monitor recording contains the window.
  - If this fails, stop and report before any other group (design Risk 1).

## 2. Audio (Rust)

- [x] 2.1 Add `cpal` and `audio.rs` with `inputs()` and `Capture`:
  - a microphone stream and a loopback stream on the default output;
  - f32 stereo conversion, linear resample to 48 kHz and per-source ring buffers.
  - Verify: a temporary debug command logs input devices and non-zero RMS while speaking and while music plays.
- [x] 2.2 Add the 10 ms mixer thread:
  - silence padding, i16 clamp, a chunk channel and RMS level callbacks;
  - level-only mode for the panel setup state;
  - a lost-mic stream error marks the source lost and reports once.
  - Verify: unplugging a USB mic keeps chunks flowing with silence and reports one loss.

## 3. Recorder (Rust)

- [x] 3.1 Teach `recorder.rs` audio:
  - enable `AudioSettingsBuilder` when a capture is attached;
  - drain chunks in `on_frame_arrived` and send them with `send_audio_buffer` at `first_video_ts + samples * 10^7 / 48000`.
  - Verify: a 10-minute recording with a clap every minute stays within 100 ms of sync in a video player; a silent recording has no audio track.
- [x] 3.2 Add pause / resume (`paused: Arc<AtomicBool>`, frame skipping, video offset, discarded audio while paused) and `Recorder::pause` / `resume`. Verify the "Pause and resume" and "Stop while paused" scenarios.
- [x] 3.3 Add `record_mic`, `record_camera` and `record_system_audio` to `config.rs` with `#[serde(default)]` and the defaults from the design. Verify an older config loads unchanged with the new defaults.

## 4. Panel and camera UI (frontend)

- [x] 4.1 Make `main.tsx` render `PanelApp` / `CameraApp` for `?view=panel` / `?view=camera`, with `lib/api.ts` wrappers for the new commands and events and en/vi strings in `lib/i18n.ts`. Verify `bunx tsc --noEmit`.
- [x] 4.2 Build the setup panel:
  - microphone `Select` with a level meter, camera `Select` (after a first `getUserMedia` for labels), system audio `Switch`;
  - Start (Enter), close (Esc / ✕), drag region;
  - "no microphone" and camera-error states with an Open settings button.
  - Verify the "Pick a microphone", "No microphone" and "Camera unavailable" scenarios.
- [x] 4.3 Build the compact recording panel:
  - dot, a timer from `started` / pause data, the level, device icons with name tooltips;
  - camera hide / show, Pause / Resume, Stop;
  - the lost-mic warning.
  - Verify the "Compact panel" and "Device locked" scenarios.
- [x] 4.4 Build the camera view: a round `<video>` filling the window, draggable, following the `deviceId` from the panel. Verify the "Camera selected" scenario.

## 5. Wiring (Rust, main and capture threads)

- [x] 5.1 Create and destroy the `panel` and `camera` windows and add them to the capabilities:
  - `WDA_EXCLUDEFROMCAPTURE` on `panel` only;
  - placement: panel at bottom centre of the cursor monitor, camera at bottom-left;
  - `State.panel`, the `record_panel_state` command and the `record-panel` event.
  - Verify the "Recording the whole monitor" scenario (panel absent, camera present).
- [x] 5.2 Rework `trigger(Record)` per the design flow (open, close or stop) and add `Action::RecordPick` / `PanelShow` / `PanelRecording`:
  - `record_start` hides both windows, saves the choices and opens the picker;
  - a cancel shows the panel again;
  - a pick re-shows the windows, moves the camera into the target unless dragged and goes through `begin`.
  - Verify the "Open with the hotkey", "Close the panel", "Start and pick" and "Cancel the picker" scenarios.
- [x] 5.3 Start `audio::Capture` with the config choices when recording starts and hand it to the recorder:
  - stop it on Recorded / RecordFailed / Quit and close both windows (releasing the camera);
  - level-only capture while in Setup.
  - Verify the "Narrated tutorial", "Microphone and system audio" and "Stop from the panel" scenarios.
- [x] 5.4 Add the tray Pause / Resume item, the `record_pause` / `record_resume` commands and the `snapr: paused` tooltip. Verify the "Pause from the tray" and "While paused" scenarios.
- [x] 5.5 Implement the window pick with the camera on (`region_target` on the window rect) and the panel note. Verify the "Window with webcam" and "Window without webcam" scenarios.
- [x] 5.6 Handle a lost mic: the `record-warning` event reaches the panel warning. Verify the "Headset unplugged" scenario.

## 6. Docs and integration

- [x] 6.1 Update the CLAUDE.md recording paragraph, the module list (`audio.rs`, panel and camera windows, commands, config fields) and the frontend section. Verify it matches the shipped code.
- [ ] 6.2 Do a full pass:
  - record a monitor, an area and a window, each with and without the camera, mic and system audio;
  - pause, resume and stop from the panel, the tray and the hotkey;
  - test on a second monitor at a different DPI;
  - `cargo clippy` and `bunx tsc --noEmit` clean.
  - Verify every scenario in both delta specs.

## Workflow follow-up

- Archive with `/opsx:archive` after release. That creates `openspec/specs/recording-panel/spec.md` and updates `screen-recording`.
