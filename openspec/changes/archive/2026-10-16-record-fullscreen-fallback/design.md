# Design

## Context

See proposal.md for motivation and `specs/screen-recording` for behavior.

- `Overlay` (v1, `overlay.rs`) keeps its window list and `hit()` private. In window mode, a left press with no hit returns `Outcome::Pending`, so `Snap` never sees it.
- `visible_windows` keeps every titled, non-minimized window that does not belong to snapr. The desktop shell window `Progman` ("Program Manager") can therefore be in the list and get highlighted, which is why "click the desktop" may currently pick it.
- `Snap::window_event` forwards every overlay event to `overlay.event()`, and `record_pick` marks record mode.
- `Recorder::start(proxy, hwnd, path)` builds a `windows_capture::window::Window` item, and the handler's `new` reads the size from `GraphicsCaptureItemType::Window`.
- `windows-capture` also offers `monitor::Monitor::from_raw_hmonitor(*mut c_void)`, which converts to `GraphicsCaptureItemType::Monitor`.

## Goals / Non-Goals

**Goals:**
- No behavior change to the window screenshot picker or to any v1 overlay logic.
- One recorder code path for both targets.

**Non-Goals:**
- Hiding the desktop highlight in record mode. Hovering the desktop may still draw a border around it, which reads naturally as "whole screen".

## Decisions

### 1. Intercept the click in `Snap`, with one additive overlay accessor
- Add `pub fn hovered(&self) -> Option<isize>` to `Overlay`. It returns the hwnd of `self.hit()`, using the same conversion as the existing window-pick branch.
- In `Snap::window_event`, before forwarding:
  - if `record_pick` is set and the event is a left-button press, and `overlay.hovered()` is `None` or a desktop window, then close the overlay, clear the flag and start a monitor recording;
  - otherwise forward the event as today.
- The window screenshot path never sets `record_pick`, so it can't be affected.

Alternatives considered:
- A `Pick::Record` variant. It was rejected because it touches every `Pick` match in v1 overlay code.
- Returning the frozen frame on no hit. It was rejected because it changes the window screenshot behavior, and the user chose record only.

### 2. Desktop detection by window class
`is_desktop(hwnd)` uses `GetClassNameW` and returns true for `Progman` and `WorkerW`. These are the shell's desktop windows, and they are stable across Windows 10 and 11. It lives in `lib.rs` next to the routing, because it only matters there.

### 3. Monitor under the cursor
- `GetCursorPos` + `MonitorFromPoint(.., MONITOR_DEFAULTTONEAREST)` gives the `HMONITOR`. The cursor position is read at click time.
- The HMONITOR is passed as `isize`, because `Flags` must be `Send`.

### 4. `Recorder::start(proxy, target, path, ..)` with `enum Target { Window(isize), Monitor(isize) }`
- `Target` is `Copy + Send` and becomes part of `Flags`.
- `start` matches on it to build `Settings` with either `Window::from_raw_hwnd` or `Monitor::from_raw_hmonitor`. These are two `Settings` types, so each branch calls `start_free_threaded`.
- The handler's `new` rebuilds the item from `Target` and reads `Size()` from either `GraphicsCaptureItemType::Window` or `::Monitor`.
- Everything after `new` is unchanged.
- If `add-recording-fps-setting` lands first, `fps` stays a separate argument.

Alternative: a separate `MonitorRecorder`. It was rejected because the code would be duplicated.

## Risks / Trade-offs

- [Other shell windows, such as the taskbar `Shell_TrayWnd`, are still pickable as windows] → Acceptable. The taskbar is a real window, and only the desktop is promised as a fallback.
- [High-DPI or mixed-DPI monitors] → WGC monitor capture uses physical pixels, so the size is correct. Even rounding still applies.
- [`MonitorFromPoint` vs the overlay's monitor] → The overlay already opens on `capture::cursor_monitor()`, and the click happens inside it, so both refer to the same monitor.
- [Monitor disconnected mid-recording] → WGC fires `Closed`, so `on_closed` finalizes, the same as a closed window.

## Migration Plan

No migration is needed. This is behavior-only, with no config changes.
