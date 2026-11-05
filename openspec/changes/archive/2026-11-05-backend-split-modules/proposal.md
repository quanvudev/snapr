# Proposal

## Why

`src-tauri/src/lib.rs` (about 1000 lines) mixes app wiring, the tray, hotkey registration, the screenshot save worker, the capture event loop and the recording lifecycle, so every backend change touches it and other modules reach back into it through `crate::`. Shared download code lives in `paddle.rs` even though `whisper.rs`, `onnx.rs` and `update.rs` use it, and event names are string literals spread over many files. This is the first of four sequenced backend refactors (then typed config, the model store trait with the `State` split, and the recording session object), and it only moves code so the later changes start from small, single-purpose modules.

## What Changes

- Split `lib.rs` by responsibility, moving code without changing logic:
  - `tray.rs`: tray menu construction, the menu event handler, `tooltip` and `refresh`.
  - `hotkeys.rs`: `apply` (shortcut and mouse-hook registration), `ocr_available` and `trigger` (hotkey and ring dispatch).
  - `jobs.rs`: the `Job` enum, the save worker thread, `work` (pad, save, clipboard) and `read_text` (QR / OCR to clipboard).
  - `recording.rs`: `record`, `ended`, `close_panel`, `stop_recording`, `pause_recording` and the debug `stop_after_limit`.
  - `snap.rs`: `Snap`, its `ApplicationHandler` impl, `Mode`, `Countdown`, the pick / ring / pin / preview methods, the record target helpers (`is_desktop`, `cursor_monitor`, `window_rect`, `region_target`) and `spawn_capture`.
  - `settings.rs` gains `show_settings` and `show_splash`.
  - `lib.rs` keeps the module list, `Action`, `State`, plugin setup, `setup`, the window event handler and `invoke_handler`.
- Add `fetch.rs` holding `Remote`, `Fetch`, `agent`, `clean`, `part` and `disk_bytes`, moved from `paddle.rs`; `paddle.rs`, `whisper.rs`, `onnx.rs` and `update.rs` import from it.
- Add `events.rs` with one constant per emitted event name and use them at every emit site.
- Update the Architecture section of `CLAUDE.md` to the new file locations.
- No change to commands, command arguments, event names or payloads, config format, the frontend, or any user-visible behaviour.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. This is a pure refactor with no requirement changes, so the change sets `skip_specs: true`.

## Impact

- Code: `src-tauri/src/lib.rs`, `paddle.rs`, `whisper.rs`, `onnx.rs`, `update.rs`, `settings.rs`, `panel.rs`, `ring.rs`, `queue.rs`, `gallery.rs`, `video.rs`, `transcript.rs` (imports and emit sites only), plus the new `tray.rs`, `hotkeys.rs`, `jobs.rs`, `recording.rs`, `snap.rs`, `fetch.rs`, `events.rs`.
- Docs: `CLAUDE.md` Architecture section.
- No dependency, IPC, config or frontend changes. Verification is `cargo clippy`, `bunx tsc --noEmit` and a manual smoke test, since the project has no tests.
- Out of scope: the internals of the capture UI modules (`overlay`, `toolbar`, `draw`, `ring`, `pin`, `toast`, `frame`) and of `transcript/`, `gif`, `subtitle`, `audio`; type changes (`Config` enums, `Lang`, `TaskError`); the `ModelStore` trait; splitting `State`; the recording session object.
