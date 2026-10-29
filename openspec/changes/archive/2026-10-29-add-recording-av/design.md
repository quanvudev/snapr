# Design

## Context

Recording today:
- `Action::Record` (capture thread) opens the window picker in `Mode::Record`.
- `Snap::begin` runs the optional countdown, then `record(app, target)` starts `Recorder` (`recorder.rs`, `windows-capture` WGC + `VideoEncoder`, audio disabled).
- The recorder lives in `State.recording`. The main thread stops it through the hotkey (`trigger` → `stop_recording`), the tray Stop item or Quit.
- `frame.rs` already makes click-through, non-activating, `WDA_EXCLUDEFROMCAPTURE` layered windows (area frame, countdown digit).

`windows-capture` 2.0.1's `VideoEncoder` accepts audio through `AudioSettingsBuilder` (default AAC, 48 kHz, 2 ch, 16-bit) and `send_audio_buffer(&[u8], timestamp)`. Two event loops exist: Tauri on the main thread (webview windows, tray), and winit on the capture thread (picker, recorder start).

See `proposal.md` for the motivation and `specs/` for the behaviour.

## Goals / Non-Goals

**Goals:**
- Reuse the existing picker, countdown, `Target` and `Recorder` paths. The panel only wraps them.
- Keep every WASAPI / `cpal` call in one audio module, and every webview-specific call (camera permission, display affinity) in one place.

**Non-Goals:**
- Native (Media Foundation) webcam capture, unless the spike in task 1 fails.
- Echo cancellation, a loopback device picker, a pause hotkey.

## Decisions

### 1. Panel and webcam are Tauri webview windows

Two windows, built on demand with `WebviewWindowBuilder` on the main thread:

- `panel`:
  - frameless, transparent, always on top, skip taskbar, not resizable;
  - sized to its content (setup about 340x260 logical, recording about 420x48);
  - dragged with `data-tauri-drag-region`.
- `camera`:
  - frameless, transparent, always on top, skip taskbar, not focusable;
  - 180x180 logical, drawn as a circle with CSS;
  - shows `getUserMedia({ video: { deviceId } })`.

Both load `index.html?view=panel` or `?view=camera`, and `main.tsx` renders `PanelApp` / `CameraApp` instead of `App`. The panel UI uses the existing shadcn `Select`, `Switch`, `Button` and `Tooltip`. After creation, `SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE)` is applied to `panel` only.

Alternatives considered:
- A native winit / tiny-skia panel (like the ring) means hand-drawn dropdowns and no webcam decoding. Rejected.
- Compositing the webcam into the frames (option B in explore). Rejected: dragging becomes a coordinate-mapping problem and a preview window is still needed.

### 2. Camera permission and devices

- WebView2 asks the host for camera permission. In `setup` (or on window creation) the code gets the `ICoreWebView2` through `with_webview` (the `webview2-com` version that Tauri already pulls in). It registers `PermissionRequested`, and grants `Camera` for the app's own origin only.
- Camera devices are listed in the panel with `navigator.mediaDevices.enumerateDevices()`, after a first `getUserMedia({ video: true })` so the labels are filled.
- The chosen `deviceId` is persisted as `record_camera` and passed to the `camera` window as a query parameter or event. WebView2 device IDs are stable per origin.
- If permission or opening fails, the panel shows the error and an "Open settings" button (`ms-settings:privacy-webcam`).

**This is the riskiest assumption, so task 1 spikes it first.**

### 3. Audio module (`audio.rs`, `cpal`)

- `inputs() -> Vec<Device { id, name, default }>` lists the input devices. The `id` is the `cpal` device name, which WASAPI keeps stable per endpoint.
- `Capture::start(mic: Option<id>, system: bool, sink)`:
  - opens a WASAPI input stream for the microphone and, when `system`, an input stream on the default output device (`cpal` performs loopback for an output device on WASAPI);
  - each callback converts to f32 stereo (mono duplicated, more channels down-mixed) and resamples linearly to 48 kHz;
  - the samples are pushed to a per-source ring buffer.
- A mixer thread runs every 10 ms:
  - it takes 480 frames from each active source, padding with silence when a source has fallen behind or is gone;
  - it sums and clamps to i16 and sends the chunk to the recorder through a `crossbeam` / `mpsc` channel;
  - it also computes the microphone RMS level, emitted to the panel as `record-level` about 15 times a second.
- Level-only mode (setup state): the microphone stream runs and only the level is used. The mixer does not send chunks.
- A stream error on the microphone (device removed) marks the source as lost:
  - silence fills its part from then on;
  - `Action::MicLost` reaches the panel through a `record-warning` event.

Alternative: raw WASAPI through `windows`. Rejected for now: `cpal` already wraps capture and loopback with far less code.

### 4. Encoding audio and timestamps

- `VideoEncoder::new` gets `AudioSettingsBuilder::new()` (AAC 48 kHz stereo 16-bit) when any source is active, and `.disabled(true)` otherwise.
- The recorder's `Handler` holds the audio receiver. On each `on_frame_arrived` it drains the pending chunks and calls `send_audio_buffer(chunk, ts)`.
- `ts = first_video_ts + samples_sent * 10_000_000 / 48_000`. Audio time is the sample count anchored to the first video frame, so it is continuous by construction. The drift between the device clock and QPC over 10 minutes is far below the 100 ms bound.

### 5. Pause and resume

- `Recorder` gets `paused: Arc<AtomicBool>`. Pause and resume flip it; the main thread holds the recorder, as for Stop.
- In `on_frame_arrived`:
  - while paused, frames are skipped and the audio channel is drained and discarded;
  - on the first frame after a resume, `offset += time - last_sent - interval`;
  - every video timestamp sent is `time - offset`.
- Audio needs no offset: discarded chunks are never counted in `samples_sent`.
- The tray Pause item is a new `MenuItem` in `State`, with its text toggled.
- The tooltip reads `snapr: paused`.
- `update_install` still refuses while recording (paused included).

### 6. Flow and state

```
main thread                              capture thread
-----------                              --------------
trigger(Record):
  recording  -> stop (as today)
  countdown  -> Action::Record (cancel, as today)
  panel open -> close panel + camera
  else       -> open panel (Setup)
panel Start (command record_start):
  save choices to config
  hide panel + camera
  send Action::RecordPick ------------>  pick(Window, Mode::Record)
                                          cancel -> Action::PanelShow --> show panel (Setup)
                                          pick   -> camera on & Window? -> region_target(window rect)
                                                    begin(target) (countdown as today)
                          <------------- Action::PanelRecording(bounds) -> show panel compact,
                                                                             camera to bottom-left of bounds
                                                                             unless the user moved it
                                          record(app, target, audio)
Stop/Recorded/RecordFailed -> close panel + camera, stop audio
```

- `State.panel: Mutex<PanelState>` (`Closed | Setup | Picking | Recording { started, paused_total, paused_at }`) is the single source of truth.
- The panel reads it with a `record_panel_state` command and follows a `record-panel` event.
- The timer is computed in the panel from `started` and the pause data, so there is no tick event.

### 7. Window pick with the camera on

In `window_event` for `(Mode::Record, Shot::Window { hwnd, .. })`, when `record_camera` is set:
- the window rectangle (`window_rect(hwnd)`) goes through the same `region_target` used for drags, so it is clipped to its monitor and rounded to even;
- the result starts a `Target::Area`.

### 8. Config

These go in `Config`, each with `#[serde(default)]`:
- `record_mic: String`: `"default"` = the Windows default input, `""` = none, otherwise a device id. The default is `"default"`.
- `record_camera: String`: `""` = off, otherwise a WebView2 `deviceId`. The default is `""`.
- `record_system_audio: bool`, default `false`.

They are written by `record_start` through the normal `save` path, so `apply()` runs as usual. There is no Settings UI for them: the panel is the UI.

## Risks / Trade-offs

- [WebView2 camera permission cannot be auto-granted or `getUserMedia` fails in a transparent window] → Spike first (task 1). The fallback is a native Media Foundation webcam in a layered window, which is a scope change to be confirmed with the user.
- [`cpal` loopback quirks: no stream while nothing plays] → The mixer pads missing system audio with silence, so the timeline never stalls.
- [Echo when the mic and system audio are both on with speakers] → Not handled. Headphones are noted in the CHANGELOG.
- [Window pick with the camera becomes an area recording, so moving the window is not followed and "ends when the window closes" no longer applies] → Accepted. The spec states it, and the panel shows a note when the camera is on.
- [The webcam drag fights `WS_EX_NOACTIVATE`] → The `camera` window can be focusable while dragging. Focus is returned to the previous window after the drag ends.
- [The extra step before every recording] → Enter starts immediately. The hotkey flow is otherwise unchanged.

## Migration Plan

An older config loads with the new fields at their defaults (default mic, camera off, system audio off). Nothing else migrates. Rollback is a plain version downgrade, because the new fields are ignored by older builds.

## Open Questions

- The exact panel sizes and the webcam border style can be tuned during implementation without changing the spec.

## Revisions after the first manual pass

- Setup is Loom-style: the panel holds its own round camera preview above an icon-only pill bar (mic and camera Selects, system audio Toggle, Start, Close). The `camera` window is created only at `picked`, so setup never shows two camera streams; the first camera is auto-selected when the saved one is missing.
- The recording bar has only Stop, the timer, Pause / Resume and Delete (two clicks; `record_discard` sets `State.discard` so the `Recorded` handler deletes the file and skips the toast). The camera hide toggle was dropped from the bar; `record_camera_toggle` stays unused.
- `attach` keeps the bar centred under the webcam, overlapping its bottom slightly, and follows every camera `Moved`. The camera window is 212 px to leave room for its shadow; the bar window has transparent room below for tooltips.
- The panel window sizes itself to its content through `record_panel_fit`.
- Frame rate: the encoder read surfaces asynchronously, so a reused surface (and the recycled WGC frame-pool surface) was overwritten before encoding and the video repeated frames (about 20 distinct frames per second at 60 fps). `windows-capture` is vendored with `send_frame_region` (GPU crop with an explicit timestamp) and a ring of 8 surfaces, and every target uses it. The toast thumbnail is converted only at stop. A TestUFO run then measured 598 distinct frames out of 600. Raw-buffer sends also came out upside down, which this path avoids.
