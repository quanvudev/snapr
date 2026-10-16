# Proposal

## Why

With the record hotkey, users can only record a single window. Clicking the desktop either does nothing or picks the desktop shell window, so there is no way to record the whole screen. Treating a click on the desktop or on empty space as "record the full screen" adds full-screen recording without a new hotkey.

## What Changes

- In the picker opened by the **record** hotkey, a left click that is not on an application window records the **monitor under the cursor**. This covers the desktop (the shell's desktop windows count as "not a window") and empty space.
- Clicking an application window still records that window.
- Monitor recordings behave like window recordings:
  - stop with the hotkey or the tray;
  - the capture border and the "recording…" tooltip;
  - MP4 naming, the toast and the configured frame rate.
- The **window screenshot** hotkey is unchanged: clicking the desktop or empty space still selects nothing.

Out of scope: picking among several monitors other than by cursor position, area recording, and changing the window-screenshot picker.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `screen-recording`: The record picker records the monitor under the cursor when the click is not on an application window.

## Impact

- `src-tauri/src/overlay.rs`: one additive read-only method that exposes the hovered window's hwnd. Existing behavior is unchanged.
- `src-tauri/src/lib.rs`:
  - in record mode, a left press on the desktop or on no window closes the overlay and starts a monitor recording;
  - a helper finds the monitor under the cursor and classifies desktop shell windows (`Progman` / `WorkerW`).
- `src-tauri/src/recorder.rs`: `Recorder::start` takes a target (window hwnd or monitor handle). The handler reads the size from either item type.
- No new dependencies. `MonitorFromPoint` and `GetClassNameW` come from the `windows-sys` features already enabled.
