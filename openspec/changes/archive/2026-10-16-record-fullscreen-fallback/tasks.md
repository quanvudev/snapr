# Tasks

## 1. Recorder target

- [x] 1.1 In `recorder.rs`, add `pub(crate) enum Target { Window(isize), Monitor(isize) }` (Copy). `Recorder::start` takes a `Target`, builds `Settings` from `Window::from_raw_hwnd` or `Monitor::from_raw_hmonitor`, and the handler's `new` reads the size from either `GraphicsCaptureItemType` variant. Update the existing window call site in `lib.rs` to `Target::Window(hwnd)`. Done when `cargo clippy` in `src-tauri/` is clean and window recording behaves as before.

## 2. Record-mode click fallback

- [x] 2.1 Add `pub fn hovered(&self) -> Option<isize>` to `overlay.rs`. It returns the hovered window's hwnd using the same id conversion as the window-pick branch. Do not change any other overlay code. Done when `cargo clippy` is clean.
- [x] 2.2 In `lib.rs`:
  - add `is_desktop(hwnd)` (`GetClassNameW` is `Progman` or `WorkerW`) and `cursor_monitor_handle()` (`GetCursorPos` + `MonitorFromPoint` nearest);
  - in `Snap::window_event`, when `record_pick` is set, the event is a left-button press and `overlay.hovered()` is `None` or a desktop window: close the overlay, clear `record_pick` and start the recording with `Target::Monitor(..)` through the same start path as window recordings (unique `.mp4` path, store in `State.recording`, enable Stop, tooltip).

  Done when `cargo clippy` is clean.

## 3. Check and docs

- [x] 3.1 Manual check (user):
  - record hotkey + click the desktop records the full monitor with the border;
  - on two monitors, clicking the desktop of the second records only it;
  - clicking a window still records that window;
  - window screenshot hotkey + click the desktop behaves as before;
  - stop with the hotkey or the tray saves an MP4 and shows the toast.
- [x] 3.2 Update `CLAUDE.md` Architecture (record mode desktop or empty click → monitor recording, `Recorder` `Target`, `Overlay::hovered`). Done when the doc matches the code.

## Workflow follow-up

- Run `/opsx:archive` after review to merge the spec delta into `openspec/specs/`.
