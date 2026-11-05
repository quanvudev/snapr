# Tasks

Every task moves code without editing function bodies beyond import paths and visibility. "clippy clean" means `cargo clippy` in `src-tauri/` reports no new warnings and no errors. Commit after each group.

## 1. Shared download code (`fetch.rs`)

- [x] 1.1 Create `src-tauri/src/fetch.rs` and move `Remote`, `Fetch` (with `get`, `stream`, `file`), `agent`, `clean`, `part` and `disk_bytes` from `paddle.rs` unchanged, `pub(crate)`; register `mod fetch;` in `lib.rs`; verify `paddle.rs` no longer defines them and clippy is clean
- [x] 1.2 Point `paddle.rs`, `whisper.rs`, `onnx.rs` and `update.rs` at `crate::fetch` and drop their now-unused imports; verify `grep -rn "paddle::\(Fetch\|Remote\|agent\|clean\|part\|disk_bytes\)" src-tauri/src` finds nothing and clippy is clean
- [x] 1.3 Update `CLAUDE.md` (the `paddle.rs`, `whisper.rs` and `onnx.rs` entries) so `Remote`, `Fetch`, `part`, `clean`, `disk_bytes` and `agent()` are described as living in `fetch.rs`; verify no sentence still places them in `paddle.rs`

## 2. Event name constants (`events.rs`)

- [x] 2.1 Create `src-tauri/src/events.rs` with `pub const` names `GALLERY_CHANGED`, `PADDLE_PROGRESS`, `PADDLE_DONE`, `WHISPER_PROGRESS`, `WHISPER_DONE`, `QUEUE`, `QUEUE_ENDED`, `QUEUE_HISTORY`, `SETTINGS_OPEN`, `UPDATE_STATUS`, `RECORD_PANEL`, `RECORD_LEVEL`, each equal to its current literal; verify each value against `grep -rn "listen\|once(" src` in the frontend
- [x] 2.2 Replace the literal at every emit site (`lib.rs`, `queue.rs`, `paddle.rs`, `whisper.rs`, `update.rs`, `panel.rs`), keeping each site's target (`emit_to("main", ..)`, the broadcast `emit` for `record-panel`, `emit_to(PANEL, ..)` for `record-level`); verify `grep -rnE "emit(_to)?\([^)]*\"[a-z]+(-[a-z]+)*\"\s*," src-tauri/src` shows no event-name literal outside `events.rs` and clippy is clean
- [x] 2.3 Add a `CLAUDE.md` backend module line for `events.rs` (event name constants, emit targets unchanged); verify it is present

## 3. Tray (`tray.rs`)

- [x] 3.1 Move `tooltip`, `refresh`, `tray_menu` and `build_tray` (with its menu event closure) from `lib.rs` to `src-tauri/src/tray.rs`; `setup` still builds the `MenuItem`s before `app.manage` and calls `tray::build_tray` after it; update callers (`panel.rs`, `ring.rs`, `lib.rs`) to `crate::tray::tooltip`; verify clippy is clean
- [x] 3.2 Update `CLAUDE.md` where it attributes the tray menu, `tooltip` or `refresh` to `lib.rs`; verify those references now name `tray.rs`

## 4. Recording lifecycle helpers (`recording.rs`)

- [x] 4.1 Move `record`, `ended`, `close_panel`, `stop_recording`, `pause_recording` and the `#[cfg(debug_assertions)] stop_after_limit` from `lib.rs` to `src-tauri/src/recording.rs`; update callers (`panel.rs`, the tray handler, `Snap`) to `crate::recording::...`; verify clippy is clean in both `cargo clippy` and `cargo clippy --release`
- [x] 4.2 Update `CLAUDE.md` mentions of `record()`, `ended()`, `pause_recording`, `stop_recording` so they point at `recording.rs`; verify none still place them in `lib.rs`

## 5. Hotkeys and settings window (`hotkeys.rs`, `settings.rs`)

- [x] 5.1 Move `apply`, `ocr_available` and `trigger` from `lib.rs` to `src-tauri/src/hotkeys.rs`; update callers (`paddle.rs`, `settings.rs`, `panel.rs`, `ring.rs`, the window event handler and `setup` in `lib.rs`) to `crate::hotkeys::...`; verify clippy is clean
- [x] 5.2 Move `show_settings` and `show_splash` from `lib.rs` to `settings.rs`; update callers (`ring.rs`, `update.rs`, the tray handler, `setup`); verify clippy is clean
- [x] 5.3 Update `CLAUDE.md` mentions of `apply()` in `lib.rs`, `trigger`, `ocr_available(app)` in `lib.rs` and `show_settings`; verify they name `hotkeys.rs` and `settings.rs`

## 6. Save worker (`jobs.rs`)

- [x] 6.1 Move `Job`, `work` and `read_text` to `src-tauri/src/jobs.rs` and add `jobs::spawn(app, proxy) -> Sender<Job>` containing the current worker thread body (MTA init, `BackgroundCache`, the job loop); call it from `spawn_capture` at the same point as today; verify clippy is clean
- [x] 6.2 Update `CLAUDE.md` where the save worker, `read_text` and the en/vi notification strings are attributed to `lib.rs`; verify they name `jobs.rs`

## 7. Capture event loop (`snap.rs`)

- [x] 7.1 Move `Snap`, its `ApplicationHandler<Action>` impl and `impl Snap`, `Mode`, `Countdown`, the `COUNTDOWN` / `TICK` / `SETTLE` / `MIN_AREA` constants, `is_desktop`, `cursor_monitor`, `window_rect`, `region_target` and `spawn_capture` to `src-tauri/src/snap.rs`; `Snap` fields stay private; `setup` calls `snap::spawn_capture`; verify clippy is clean
- [x] 7.2 Verify `lib.rs` now contains only the `mod` list, `Action`, `State` and `run` (plugins, schemes, `setup`, window event handler, `invoke_handler`), with no `pub use` re-exports
- [x] 7.3 Update the `CLAUDE.md` Architecture overview (the two event loops, `spawn_capture`, `Snap::*` methods, `region_target`, `cursor_monitor`) so locations name `snap.rs`, and add one-line entries for `tray.rs`, `hotkeys.rs`, `recording.rs`, `jobs.rs` and `snap.rs` to the backend module list; verify `grep -n "lib.rs" CLAUDE.md` only refers to what `lib.rs` still contains

## 8. Integration checks

- [x] 8.1 Run `cargo clippy` and `cargo clippy --release` in `src-tauri/` and `bunx tsc --noEmit` at the root; verify all pass and `git diff --stat src/` shows no frontend changes
- [ ] 8.2 Smoke-test captures in `bun run tauri dev`: fullscreen, window, area, text (QR and plain text), draw (save, copy, pin), ring open / pick / cancel, toast Pin, gallery Pin; verify each behaves as before (file saved, clipboard, toast, notifications)
- [ ] 8.3 Smoke-test recording: start from the hotkey, the panel and the ring in fullscreen, window and area mode; countdown on and cancelled from the tray Stop and the hotkey; pause and resume from the panel and the tray; stop; discard; verify tooltips, tray items, panel phases and the toast match current behaviour
- [ ] 8.4 Smoke-test background work: queue one GIF, compress, transcribe and subtitle task, cancel one and retry it; download, cancel and delete a PaddleOCR size and a Whisper model; run Check now on About; change a hotkey and the OCR engine in settings; verify progress events, history, notifications and hotkey re-registration work

## Workflow follow-up

- Archive this change with `/opsx:archive` after the integration checks pass.
- Next change in the series: P2, typed `Config` enums, `Lang` and `TaskError`.
