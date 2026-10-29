# Design

## Context

`Recorder::start(proxy, Target, path, fps)` runs a `windows-capture` WGC session on a window or monitor item and passes each GPU frame straight to `VideoEncoder::send_frame`, throttled to the configured fps. The record picker is the window picker (`Pick::Window`) with `Snap.mode = Mode::Record`; `Snap::window_event` turns a click on a window into `Target::Window` and a click on the desktop into `Target::Monitor`, then calls `record(app, target)`, which creates the file via `capture::unique_path` and stores the recorder in `State.recording`. `trigger(Record)` stops a running recorder on the main thread; the tray Stop item calls `stop_recording`. Pins already have a `recording` hide reason, and the toast already uses `WDA_EXCLUDEFROMCAPTURE`. Layered per-pixel-alpha windows are drawn with `ring::bitmap` / `ring::present`.

## Goals / Non-Goals

**Goals:**
- Keep the window/monitor path byte-for-byte as today (GPU frames, no readback).
- Countdown and area are independent: the countdown wraps any `Target`.

**Non-Goals:**
- Audio, GIF, pause, multi-monitor areas, moving or resizing an area mid-recording.

## Decisions

### Area = monitor capture + CPU crop

WGC cannot capture an arbitrary rectangle, so `Target::Area { monitor, rect }` (rect in physical pixels relative to the monitor, already even-sized) starts a monitor session and, in `on_frame_arrived`, after the existing throttle check, calls `frame.buffer_crop(x, y, x + w, y + h)`, flattens it with `as_nopadding_buffer` and sends it with `VideoEncoder::send_frame_buffer(buf, timestamp)`. The encoder is built with the area size. `keep()` (the toast's last frame) reuses the same crop.

Alternative: a hidden full-monitor encode followed by a crop pass. Rejected: two encodes, a temp file and a long stop.

The readback only happens on frames that are actually sent, so the cost scales with fps x area, not with the monitor size. Window and monitor targets keep `send_frame`.

### Picker: drag in record mode only

`Overlay::open` gets a flag for record mode (or the existing `Mode` is passed through). In `Pick::Window` with that flag, a left press remembers the start point; once the cursor moves more than 4 px on either axis the overlay switches to drawing a rectangle exactly like `Pick::Area`, and the release returns a new `Shot::Region(rect)` in overlay coordinates. A release without that movement keeps today's click semantics, which `Snap::window_event` already routes to window or monitor. The press-time desktop interception in `window_event` moves to the release so a drag that starts on the desktop is an area, not a monitor.

`Snap` converts the region to screen coordinates with the overlay origin, takes the monitor under the drag start (`MonitorFromPoint`), clips the rect to it, rounds the size down to even and builds `Target::Area`. A clipped rect smaller than 16x16 is treated as a click at the start point.

### Area frame window

A new small module (e.g. `frame.rs`) owns a layered window sized to the area plus a 2 px border on each side, drawn once (accent-blue border, fully transparent interior), with `WS_EX_TRANSPARENT | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, topmost, and `SetWindowDisplayAffinity(WDA_EXCLUDEFROMCAPTURE)`, so it is click-through and never in the video. It is created when an area recording starts and dropped on `Action::Recorded` / `RecordFailed`. Being outside the rect, it would not be in the crop anyway; the exclusion is for other apps' captures and is cheap insurance.

### Countdown on the capture thread

`Snap.countdown: Option<Countdown { target, step, next: Instant, window }>`. After a pick with `record_countdown` on, `Snap` hides pins (`hide_recording`), opens a layered, non-activating, excluded-from-capture window showing the digit (Segoe UI via `draw::system_font`, white on a dark rounded square, about 160 logical px) centered on the target rect and clamped to its monitor, and sets the tray tooltip/Stop item. `about_to_wait` advances 3 -> 2 -> 1 using `ControlFlow::WaitUntil`, like the pin copy flash. On the last tick the window is dropped first and `record(app, target)` runs on the next turn, so the frame never contains a digit; the file name comes from `unique_path` at that moment.

Cancel paths:
- Record hotkey: `trigger` finds no recorder and sends `Action::Record`; `user_event` sees `countdown.is_some()`, drops it and returns instead of opening the picker.
- Tray Stop: enabled when the countdown starts; its handler sends a new `Action::CancelCountdown` when `stop_recording` finds nothing. Quit drops the countdown with the event loop.
- Cancel and a failed start both `show_recording()` pins, restore the tooltip and disable Stop.

Esc is not a cancel path: the countdown window must not take focus from the app being recorded.

### Setting

`Config.record_countdown: bool` with `#[serde(default)]` (off). `countdown-row.tsx` in `settings/recording/` uses the shadcn `Switch`, saved through the usual autosave. `record()` already reads the config at start; the countdown reads it at pick time.

## Risks / Trade-offs

- [`send_frame_buffer` row order or pixel format differs from `buffer_crop` output (e.g. bottom-up)] → verify on the first area recording; flip rows in the crop step if needed.
- [CPU readback at 60 fps on a large area raises CPU and may drop frames] → the throttle already drops late frames; document that area recording is heavier than window recording.
- [Windows' yellow capture border surrounds the whole monitor during an area recording, not the area] → the snapr frame marks the actual area; the system border cannot be limited to a sub-rect.
- [The picked window moves or closes during the countdown] → a window target is resolved by hwnd at start, so a moved window is still recorded; a closed one fails with the normal tooltip error. An area target records the fixed rectangle by design.
- [DPI: overlay coordinates vs. monitor-relative physical pixels] → the overlay frame is already physical; convert once with the overlay origin and the monitor rect from `desktop()`.
