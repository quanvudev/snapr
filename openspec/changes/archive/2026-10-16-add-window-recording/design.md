# Design

## Context

See proposal.md for motivation and specs/screen-recording for behavior.

Current shape (`src-tauri/src/lib.rs`):
- Hotkeys are registered on the Tauri main thread in `apply()`. They zip `Hotkeys::combos()` (a `[String; 3]`) with a fixed `[Action; 3]` and send the action through the `EventLoopProxy` in `State`.
- The winit capture thread (`Snap`) owns the overlay and the toast. `Outcome::Save(shot)` is forwarded to the save worker, which sends back `Action::Saved(img, path)` to open the toast.
- The overlay's window pick already yields `Shot::Window { hwnd, .. }` (`overlay.rs`), so the picker can be reused unchanged.
- `overlay.rs`, `toast.rs` and `capture.rs` are v1 code kept verbatim. `Toast::open(event_loop, &RgbaImage, PathBuf, secs)` is format-agnostic: it shows an image and opens Explorer on the path.

## Goals / Non-Goals

**Goals:**
- Reuse the window picker and the toast without editing `overlay.rs` or `toast.rs`.
- Keep recording state reachable from both threads, so Quit on the main thread can finalize the file.
- Add exactly one new dependency.

**Non-Goals:**
- A generic "capture target" abstraction for future monitor or area recording. Add it when a second target exists.
- A constant frame rate. WGC only delivers frames when content changes, and MP4 timestamps handle variable timing.

## Decisions

### 1. `windows-capture` for capture and encoding
Use `windows-capture` 2.x. It uses Windows Graphics Capture to capture a window from its raw hwnd, and the Media Foundation `VideoEncoder` to write H.264 MP4 with audio disabled.
- The capture runs with `start_free_threaded`. It returns a `CaptureControl` that can be stopped from any thread, and `CaptureControl::callback()` gives access to the handler so we can finalize the encoder.
- Capture settings:
  - cursor on;
  - border `Default`, which gives the native yellow border required by the spec;
  - `MinimumUpdateIntervalSettings::Default`, with a 33 ms throttle on the frame timestamp in `on_frame_arrived` that caps the rate at about 30 fps (`Custom` fails to start where the OS lacks it; see Spike findings);
  - `ColorFormat::Bgra8`, matching the encoder's BGRA8 input;
  - encoder: H.264, audio disabled, built in the handler's `new` on the capture thread.

Alternatives considered:
- An ffmpeg sidecar adds a large binary and process management.
- xcap's video recorder only yields raw frames, so it still needs an encoder.
- Hand-written WGC + Media Foundation code through the `windows` crate is a lot of unsafe code.

### 2. New `recorder.rs` with one `Recorder` type
`Recorder::start(hwnd, path, proxy) -> Result<Recorder, String>` and `Recorder::stop(self)` are its whole API. The handler struct, which implements `GraphicsCaptureApiHandler`, holds:
- `Option<VideoEncoder>`;
- the fixed even video size;
- the last frame (`RgbaImage`);
- the output path;
- a proxy clone.

Finalizing is a single private step. It takes the encoder and calls `finish()`, then sends `Action::Recorded(last_frame, path)`, or `Action::RecordFailed(error)` if finishing or sending a frame fails, so the winit thread clears the recording and shows the error in the tooltip. Both `stop()` and the handler's `on_closed` run this step, and taking the encoder makes the second call a no-op.

Alternative: adding the logic to `capture.rs`. It was rejected because `capture.rs` is v1 code and recording has its own lifecycle.

### 3. Recorder lives in `State`, not in `Snap`
Add `recording: Mutex<Option<Recorder>>` to `State`.
- The record hotkey handler on the main thread checks it first. If a recording is running, it takes the recorder and stops it, with no proxy round-trip. Otherwise it sends `Action::Record` to open the picker.
- Tray "Stop recording" and "Quit" also take and stop it directly. Quit therefore finalizes synchronously before `app.exit(0)`.
- When the window is closed, `on_closed` finalizes. The `Action::Recorded` handler on the capture thread then clears `State.recording`, so the next press starts a new recording.

Alternative: keeping the recorder in `Snap`. It was rejected because Quit runs on the main thread and would need a blocking cross-thread handshake.

### 4. Picker routing with a flag in `Snap`, not a new `Pick` variant
`Action::Record` opens `Overlay::open(event_loop, Pick::Window)` and sets `Snap.record_pick = true`. Any other `Pick` clears the flag.
- On `Outcome::Save(Shot::Window { hwnd, .. })` with the flag set, `Snap` calls `Recorder::start` and stores the result in `State.recording` instead of sending the shot to the save worker.
- Every outcome clears the flag. That includes Cancel.

Alternative: `Pick::Record`. It was rejected because it means editing `overlay.rs` (v1 verbatim) for no behavioral difference inside the overlay.

### 5. Fixed even video size, frames cropped to it
The video size is the window's capture item size at start, rounded down to even numbers. Each frame is sent with `send_frame(frame)`, which crops larger frames from the top-left and pads smaller ones with black on the GPU.
- For each sent frame, the visible part (`frame.buffer_crop`) is read back, swapped from BGRA to RGBA and kept as the last-frame `RgbaImage` for the toast.

Alternative: restarting the encoder on resize. It was rejected because it produces multiple files or needs muxing.

### 6. Shared unique-name helper
Extract the timestamp + `_N` loop from `capture::save` into `capture::unique_path(dir, ext)`, which both PNG saves and recordings use. This is a small, behavior-preserving edit to v1 code and is better than duplicating the loop. The recording name uses the time the recording starts.

### 7. Toast and status
- `Action::Recorded(img, path)` does three things. It clears `State.recording`, restores the normal tooltip and disables the tray item. It then follows the same path as `Action::Saved`, opening the toast if `preview_seconds > 0` and no overlay is open.
- `Recorder::start` success sets the tooltip to `snapr: recording…` and enables the tray item.
- The tray item is kept as a `MenuItem` in `State` so both threads can call `set_enabled`.

### 8. Config and settings
- `Hotkeys` gets `record` (default `"4"`). `#[serde(default)]` on the struct fills it in for old configs.
- `combos()` becomes `[String; 4]`, `apply()`'s action array gets `Action::Record`, and `settings.rs::check` handles four slots.
- Frontend: add `"record"` to `SLOTS`, `record` to the `Hotkeys` type and the label to both languages in `i18n.ts`. Rows render from `SLOTS`.

## Risks / Trade-offs

- [windows-capture API details differ from the README, such as the pixel format of buffer encoding or the shape of the `callback()` access] → Task 1 is a short spike that confirms them before wiring. Decision 5 has a fallback.
- [Minimized window delivers no frames] → The video jumps over that period. This is acceptable and noted rather than handled.
- [`MinimumUpdateInterval` is missing on many builds, including Windows 11 22631, and `Custom` then fails to start] → It is left at `Default`, and the handler throttles to 33 ms itself.
- [Protected or elevated windows cannot be captured] → `Recorder::start` returns an error, and the tooltip shows it as required by the spec.
- [Encoder `finish()` blocks the main thread on Quit or tray stop] → It is short (it flushes the MP4 moov atom), so the blocking is acceptable.
- [Larger binary from the `windows` crate pulled in by `windows-capture`] → Accepted as the cost of not shipping ffmpeg.

## Migration Plan

No data migration is needed. Old `config.toml` files load with the default record hotkey `4`. If `Win+Shift+4` is already taken by another app, the existing tooltip shows "hotkey in use" and the user can rebind it. To roll back, revert the change. A `record` key left in `config.toml` is ignored by older builds, because serde ignores unknown fields by default.

## Spike findings

Read from `windows-capture` 2.0.1 (`~/.cargo/registry/src/index.crates.io-*/windows-capture-2.0.1/src/`). `cargo build` in `src-tauri/` succeeds with `windows-capture = "2"`. Send/Sync and the item-size code below were compile-checked in a throwaway crate.

**Capture item from a raw hwnd.** `window::Window::from_raw_hwnd(hwnd: *mut c_void) -> Window` (`window.rs:339`), so `Window::from_raw_hwnd(hwnd as *mut c_void)`. `Window` is `Copy` and `unsafe impl Send` (`window.rs:73-78`). Pass the `Window` itself as the `Settings` item: it implements `TryInto<GraphicsCaptureItemType>` through `IGraphicsCaptureItemInterop::CreateForWindow` (`window.rs:363-374`), and the conversion runs on the capture thread (`capture.rs:407`). A failure there (protected window, dead hwnd) returns `GraphicsCaptureApiError::ItemConvertFailed` synchronously from `start_free_threaded`. The size at start can be read in the handler's `new` with the same conversion: `GraphicsCaptureItemType::Window((item, _))` → `item.Size()?` (`.Width`, `.Height`). No direct `windows` dependency is needed for that.

**`start_free_threaded`** (`capture.rs:367-480`). It returns `Result<CaptureControl<Self, Self::Error>, GraphicsCaptureApiError<Self::Error>>`. It spawns a thread and blocks until that thread has built the handler, created the item and started the session, so `new`, item and session errors come back synchronously (`capture.rs:459-477`). `Self::new(ctx)` runs on the capture thread before the item is converted (`capture.rs:402`). That thread is MTA (`WinRT::new`, `RO_INIT_MULTITHREADED`).
- `CaptureControl::stop(self)` (`capture.rs:171-211`): sets the halt flag, posts `WM_QUIT` to the capture thread and **joins it**. The session and frame pool are closed and the `Closed` handler is removed (`graphics_capture_api.rs:481-499`), so **`on_closed` is not called on `stop()`**. A thread that has already exited is handled (`capture.rs:179-193`). `stop` consumes the control, so take `callback()` first.
- `callback(&self) -> Arc<parking_lot::Mutex<T>>` (`capture.rs:137`, `use parking_lot::Mutex` at `capture.rs:7`). `lock()` returns the guard directly, with no `Result`. The handler is dropped only when the last `Arc` is dropped. The crate keeps no clone after the thread exits, so a clone taken before `stop()` keeps the handler alive.
- On window close, `on_closed` runs on the capture thread **while the callback mutex is held** (`graphics_capture_api.rs:196-210`). Then it posts `WM_QUIT` and the thread exits by itself. The `CaptureControl` stays valid, and a later `stop()` on it just joins.
- `CaptureControl` has no `Drop`. Dropping it detaches the thread.
- Resulting stop sequence: `let cb = control.callback(); let _ = control.stop(); finalize(&mut cb.lock());`. After the join no frame callback can run, so the lock is uncontended. If `on_closed` already finalized, the encoder is `None` and finalize is a no-op.

**Send/Sync.** `CaptureControl<T, E>` holds `Option<JoinHandle<..>>`, `Arc<AtomicBool>` and `Arc<parking_lot::Mutex<T>>` (`capture.rs:95-99`). With `T: Send` (already required by the trait bounds) it is `Send + Sync`. `VideoEncoder` is `unsafe impl Send` (`encoder.rs:1370`) and is not `Sync`, which is fine behind a mutex. Compile-checked: `Mutex<Option<Recorder>>` with `Recorder { control: CaptureControl<H, String> }` and `H { Option<VideoEncoder>, RgbaImage, Sender }` is `Send + Sync`, so it can live in Tauri `State`. `Flags` must be `Send` (`capture.rs:372`), so keep the hwnd as `isize` in flags, not `HWND`/`*mut c_void`.

**Frame encoding.**
- The encoder's input stream is declared as uncompressed **BGRA8** (`encoder.rs:880-885`).
- `send_frame_buffer(&mut self, buffer: &[u8], timestamp: i64)` (`encoder.rs:1258`) expects **BGRA rows bottom-to-top**, according to its doc comment (`encoder.rs:1256`). The bytes go into `MediaStreamSample::CreateFromBuffer` unchanged (`encoder.rs:836-839`). That means a `buffer_crop` result would need a channel swap and a vertical flip. The timestamp is in **100 ns ticks** (WinRT `TimeSpan`, the same unit as `Frame::timestamp()`, which is `SystemRelativeTime`; `frame.rs:125`). It is rebased to the first frame (`encoder.rs:1262-1270`).
- `send_frame(&mut self, frame: &Frame)` (`encoder.rs:1176`) **handles size mismatches itself, on the GPU, without scaling** (`build_padded_surface`, `encoder.rs:1125-1171`). It clears a target-sized texture to black and copies the top-left `min(target, frame)` rectangle into it. Larger frames are cropped from the top-left and smaller frames are padded with black. There is no error and no crash. It uses the frame's own timestamp.
- Default codec is **HEVC** (`VideoSettingsBuilder::new`, `encoder.rs:270-280`), at 15 Mbps and 60 fps. For H.264, set `.sub_type(VideoSettingsSubType::H264)` explicitly. Disable audio with `AudioSettingsBuilder::new().disabled(true)`. The container defaults to MPEG4.
- `VideoEncoder::new(video, audio, container, path)` (`encoder.rs:862`) creates the file and blocks on several WinRT `.join()`s. Build it in the handler's `new` (capture thread, MTA). Its errors then come back from `start_free_threaded` as `NewHandlerError`.
- `finish(self)` (`encoder.rs:1314`) sends EOS and joins the transcode thread. It calls `.expect` on that join, so with `panic = "abort"` a panicked transcode thread aborts the app. Dropping an unfinished encoder also flushes it (`encoder.rs:1344-1367`).

**`Frame::buffer_crop`** (`frame.rs:188`): `buffer_crop(&mut self, start_x: u32, start_y: u32, end_x: u32, end_y: u32) -> Result<FrameBuffer<'_>, frame::Error>`. The end is exclusive, and `InvalidSize` is returned if start >= end. It does a GPU copy into a staging texture and maps it. Rows may be padded to `row_pitch`. `FrameBuffer::as_nopadding_buffer(&self, &mut Vec<u8>) -> &[u8]` (`frame.rs:375`) returns tightly packed `width*4*height` bytes, borrowing the raw mapping when there is no padding. `frame.buffer()` (`frame.rs:173`) does the same for the full frame. Bytes are in the capture `ColorFormat` order.

**Settings enums** (`settings.rs`):
- `CursorCaptureSettings::WithCursor`
- `DrawBorderSettings::Default`
- `SecondaryWindowSettings::Default`
- `MinimumUpdateIntervalSettings::Custom(Duration::from_millis(33))`
- `DirtyRegionSettings::Default`
- `ColorFormat::{Rgba16F, Rgba8, Bgra8}`

The constructor is `Settings::new(item, cursor, border, secondary, min_interval, dirty, color, flags)` (`settings.rs:134`).
- `DrawBorderSettings::Default` makes no API call (`graphics_capture_api.rs:323-335`). The OS default is the yellow border, so it works on every WGC-capable build.

**Windows-version constraints.** Every non-`Default` setting is probed with `ApiInformation::IsPropertyPresent`. If the property is missing, **`start_free_threaded` fails** (`graphics_capture_api.rs:138-158`) rather than ignoring the setting.
- `WithCursor` needs `IsCursorCaptureEnabled` (Windows 10 2004+).
- `Custom(min interval)` needs `MinUpdateInterval`, which recent Windows 11 builds have and Windows 10 does not.
- `WithBorder` / `WithoutBorder` need `IsBorderRequired` (Windows 11).
- Base WGC needs UniversalApiContract 8 (`graphics_capture_api.rs:431-434`).

I probed this dev box (Windows 11 23H2, build 22631.7582) with the public `GraphicsCaptureApi::is_*_supported()` functions (`graphics_capture_api.rs:431-479`):
- `supported=true`
- `cursor=true`
- `border=true`
- `min_interval=false`

So the risk line "`MinimumUpdateInterval` is ignored on older Windows 10 builds" is wrong. With `Custom(33ms)`, recording **fails to start even on this Windows 11 machine**. Instead, pass `MinimumUpdateIntervalSettings::Default` and throttle in `on_frame_arrived`: return early when `frame.timestamp()?.Duration` is less than 330 000 ticks (33 ms) after the last frame that was sent. That works on every build and also skips the toast readback for dropped frames.

**Verdict on decisions 2, 3 and 5.**
- **Decision 2: holds, with these adjustments.**
  - The handler is constructed by the crate inside `GraphicsCaptureApiHandler::new` on the capture thread. `Flags` is therefore `(hwnd: isize, path: PathBuf, proxy)`. `new` reads the item size, rounds it down to even, builds the `VideoEncoder` with `H264` and audio disabled, and stores encoder + path + proxy + last frame.
  - `Recorder` holds the `CaptureControl`.
  - `stop()` follows the sequence above. `on_closed` runs the same finalize under the lock the crate already holds.
  - The handler does not need to keep the video size after `new`.
  - The handler also keeps the last sent timestamp for the 33 ms throttle. Decision 1's `MinimumUpdateIntervalSettings::Custom(33ms)` becomes `Default` (see Windows-version constraints).
- **Decision 3: holds as written.** `Mutex<Option<Recorder>>` is `Send + Sync`. `stop()` joins the capture thread and finalizes synchronously on the caller's thread, so Quit can finish the file before `app.exit(0)`. Neither path waits on the winit thread, because `proxy.send_event` does not block. In the `Action::Recorded` handler, the taken recorder can be dropped or `stop()`ped: the thread has already exited after `on_closed`, and `stop()` just joins.
- **Decision 5: change it.**
  - Use `send_frame(frame)` instead of `buffer_crop` + `send_frame_buffer`. `send_frame` already crops larger frames to the fixed even size and pads smaller ones with black on the GPU. That removes the CPU crop, the BGRA swap and the bottom-up flip. Smaller frames get black padding instead of being skipped.
  - Capture in `ColorFormat::Bgra8` to match the encoder's declared BGRA8 input (`encoder.rs:882`; the crate's own `examples/cli.rs` does this, while the README uses `Rgba8` and relies on Media Foundation to convert).
  - For the toast's last frame, call `frame.buffer_crop(0, 0, min(w, vw), min(h, vh))` → `as_nopadding_buffer` → swap B and R into an `RgbaImage`, so it matches what the video shows. This is a CPU readback per sent frame, which costs about the same as the original crop plan.
