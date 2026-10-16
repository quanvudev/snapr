# Design

## Context

The winit capture loop (`spawn_capture` in `lib.rs`) owns the overlay and toast windows. Today `Snap::report()` runs `capture::save()` there, which does a PNG encode for the file plus another inside `arboard::set_image` (it adds a `PNG` clipboard format next to `CF_DIBV5`), and then calls `Toast::open`. On window pick, `Overlay::event` also runs `capture::window()` (`PrintWindow` + `round_corners`) inside the click handler, before the overlay closes. While this runs, the loop processes no events.

Winit windows (`Toast`, `Overlay`) are `!Send` and must stay on the loop thread. `AppHandle` and `EventLoopProxy` are `Send`.

## Goals / Non-Goals

**Goals:**
- Nothing slow runs on the capture loop after a selection or the fullscreen grab.
- Reuse `capture::save()` and `capture::window()` without changes.

**Non-Goals:**
- Speeding up the encode itself (compression level, dev profile).
- Speeding up `Overlay::open` (the screen grab and `xcap::Window::all()` before the overlay appears).
- Finishing in-flight saves on Quit.

## Decisions

### One long-lived worker thread fed by an `mpsc` channel
`spawn_capture` starts one worker thread next to the event loop. `Snap` gets a `Sender<Shot>`, and the worker loops on `rx.recv()`.

- Alternative: one `thread::spawn` per capture. Rejected: two saves could run at the same time and race on `path.exists()` (same-second names), on `OpenClipboard`, and on which toast or clipboard image ends up last. A single FIFO worker makes the existing `save()` name check correct as it is, and puts the clipboard and toasts in capture order for free.

### `Shot`: the work handed from the loop to the worker
```
enum Shot {
    Ready(RgbaImage),
    Window { hwnd: isize, border: Option<[u8; 3]>, frame: RgbaImage, rect: Rect },
}
```
- Fullscreen: `capture::fullscreen()` stays on the loop (it is a fast grab, and it must capture the moment the key was pressed), then `Shot::Ready`.
- Area: `crop()` stays on the overlay (a cheap copy), then `Shot::Ready`.
- Window: the overlay hands over the hwnd, `edge_color`, the frozen frame (`std::mem::take(&mut self.frame)`, since the overlay is closing anyway) and the rect. The worker runs `capture::window(...)` and falls back to a crop of `frame`, which is the same logic as today's click handler, moved over.

`Outcome::Save` carries a `Shot` instead of an `RgbaImage`. The worker receives a plain hwnd, not the `xcap::Window`. `PrintWindow` works from any thread.

### Result back to the loop through the existing proxy
`Action` gets a `Saved(RgbaImage, PathBuf)` variant and changes from `Copy` to `Clone`. The hotkey closure in `apply()` sends `action.clone()`. The worker holds an `EventLoopProxy<Action>` clone (created in `spawn_capture`) and an `AppHandle` clone.

- On success: the worker restores the tooltip to `State.status` and sends `Saved(img, path)`. `user_event` replaces any current toast, then opens `Toast::open` if `preview_seconds > 0` and no overlay is open.
- On error: the worker calls `tooltip(app, "snapr: {e}")` and sends nothing.
- Alternative: a separate wrapper event enum. Rejected: it is more code for the same result.

The worker reads `cfg` from `State` per shot, as `report()` does today.

### Toast suppressed while an overlay is open
Opening a pick overlay already drops the toast. A save that finishes during a later selection would otherwise put an always-on-top toast over the overlay. It is skipped instead, and the file is still saved.

## Risks / Trade-offs

- [Quit during a save kills the worker mid-write, which can leave a partial PNG] → Saves take well under a second, so this is accepted.
- [Pressing Ctrl+V right after a capture may paste the previous image] → The clipboard is updated a moment later than before. This is inherent to saving in the background and is accepted.
- [The frozen frame is moved into the worker for window pick, so it stays in memory until that shot is processed] → The FIFO worker holds at most a few frames, and each one is freed after its save.
- [Toast feedback still waits for the encode] → This is what the user asked for (toast after the file exists). The overlay closing immediately is the instant feedback.
