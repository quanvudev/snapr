# Design

## Context

- The app runs two event loops: Tauri on the main thread (tray, hotkeys, config, settings window) and a winit loop on the capture thread (overlay, toast, ring). Pins belong to the capture thread because they need window input and per-pixel alpha.
- `Action::Saved(RgbaImage, PathBuf)` already carries both the image and the saved path to the capture thread. `Snap::preview` opens the toast from that action.
- `toast.rs` is an opaque softbuffer window with `WS_EX_NOACTIVATE` and `WDA_EXCLUDEFROMCAPTURE`. `ring.rs` is a transparent layered window using `UpdateLayeredWindow` with `WS_EX_LAYERED | WS_EX_TOOLWINDOW`, and `ring::present` already converts a tiny-skia pixmap into a layered window update.
- The capture thread tracks windows by `WindowId` (`ring`, `toast`, then `overlay`). `Snap` has no persistent desktop objects yet.
- All screenshot paths capture the desktop inside `Overlay::open` (area/window/text/draw/record picker) or `capture::fullscreen`; recordings start from `record(...)` after the picker closes and end via `Action::Recorded` / `Action::RecordFailed`.
- Gallery commands run on the Tauri side and can reach the capture loop through `State.proxy`; `gallery.rs` already centralizes output-folder validation in `guard`.

## Goals / Non-Goals

**Goals:**

- Add a persistent, interactive native window type that reuses the existing winit/tiny-skia stack and does not disturb the capture pipeline.
- Keep pins out of Snapr's own screenshots and recordings without making them invisible to other applications.
- Add only the two confirmed entry points (screenshot toast and gallery photo) and no new config, hotkey or capture mode.

**Non-Goals:**

- Zoom, opacity, click-through, pinning from the clipboard, annotating on a pin, persistence, or pin management beyond close.
- Changes to the capture image pipeline, output formats, gallery listing, config schema or hotkey slots.
- Theming the pin (it uses the capture toolbar's fixed dark palette).

## Decisions

### 1. A pin is a layered window on the capture thread, modeled on `ring.rs`

Each pin is a winit window created with the capture thread's `ActiveEventLoop`:

- Frameless, not resizable, not active, `WindowLevel::AlwaysOnTop`, skip taskbar.
- Extended style `WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`; shown with `SW_SHOWNOACTIVATE`, never `SetForegroundWindow`.
- Content is a tiny-skia `Pixmap` presented with `UpdateLayeredWindow` (the ring's path), so image alpha, the toolbar pill and rounded corners all composite correctly.

`ring::bitmap` and `ring::present` become crate-visible and are reused by `pin.rs` instead of duplicating the DIB code. This is the main new module; `Snap` gains `pins: Pins` and routes events by `WindowId` before the overlay.

Alternatives considered:

- **Softbuffer like the toast**: simpler, but the toast composits alpha over an opaque border. A pinned padded capture would show a black/opaque background instead of the desktop, and a semi-transparent toolbar would not blend.
- **Tauri webview per pin**: much heavier, adds a webview per image, and input handling would split from the native overlay/toast.
- **`WDA_EXCLUDEFROMCAPTURE` on pins**: keeps Snapr captures clean but also hides pins from Teams/Zoom and other screen shares, removing the reference-while-presenting use case.

### 2. Input is mouse-only; a pin never takes focus

A pin uses `NOACTIVATE`, so it cannot receive keyboard input. Its grammar is:

- Press on the image or transparent margin: start a drag; `SetCapture` the window, track `GetCursorPos`, move with `set_outer_position`, release capture on mouse-up. A press on a toolbar action does not drag.
- Press on a pin raises it above other pins (`SetWindowPos` to topmost, `SWP_NOACTIVATE`).
- Hover shows a two-button toolbar at the top-right: Copy (`E8C8`) and Close (`E711`), drawn with `toolbar::fill` / `toolbar::icon` in the capture toolbar's dark palette.
- Copy copies the full-resolution original with `capture::copy`; success flashes the check glyph `E73E` for about a second via a pin deadline, failure goes to the tray tooltip. Close removes only that pin.

Window size is `max(image + 2px border, toolbar footprint)` in each dimension: the image is drawn at 1:1 inside a 2px accent-blue frame (the toast's border colour) and transparent space on the right/bottom keeps the toolbar reachable for very small images. The frame is painted as four strips outside the image pixels, so image transparency and 1:1 are preserved and Copy still uses the untouched original. The image, not the frame, is centred on the cursor. The extra area is invisible but does capture clicks; this is accepted for small pins.

Alternatives considered:

- **Focusable pin with Esc/arrow shortcuts**: steals focus from the app the user is working in and breaks typing, which is the opposite of the feature's purpose.
- **Native context menu**: extra Win32 menu code for two actions; the hover toolbar matches the capture toolbar and stays discoverable.

### 3. Placement uses the cursor monitor's work area in physical pixels

`pin.rs` asks Windows for the cursor position and the nearest monitor's `rcWork`. The image is centered on the cursor, then the window is shifted to stay inside the work area. If the image's larger side exceeds 250 logical px (scaled by the monitor's DPI), the display image is downscaled with `imageops::thumbnail` so that side is about 250 px; smaller images stay 1:1. The original image is retained for Copy. Toolbar sizes are computed from the creation monitor's scale factor. Pins keep their initial pixel size when dragged to another monitor; only the toolbar keeps its creation size.

### 4. Hiding uses reasons, not a single boolean

`Pins` holds two independent reasons:

```
struct Pins { items: Vec<Pin>, capture: bool, recording: bool }
visible = !capture && !recording
```

Transitions:

```
capture starts (any picker or fullscreen)   -> capture = true, hide all
overlay closes / fullscreen grab completes  -> capture = false, show if !recording
recording starts                            -> recording = true, capture = false, hide all
Action::Recorded / RecordFailed             -> recording = false, show all
```

This matters for two cases a single flag would break: taking a screenshot while a recording is running (clearing the capture reason must not reveal pins), and a fullscreen grab while another overlay is open (its completion must not reveal pins the overlay still hides). New pins created while hidden start hidden.

Mechanism is `ShowWindow(SW_HIDE)` / `ShowWindow(SW_SHOWNOACTIVATE)`, not display affinity. Pins are hidden before `Overlay::open` captures the desktop and before `record(...)` starts; they return when the reason clears. If `Overlay::open` or `record(...)` fails, the reason is cleared and the tray tooltip reports the error. The toast's existing `WDA_EXCLUDEFROMCAPTURE` is unrelated and unchanged.

Alternatives considered:

- **One `hidden` boolean**: simpler, but the screenshot-during-recording and overlapping-grab cases would flash pins or leave them hidden incorrectly.
- **`WDA_EXCLUDEFROMCAPTURE` only**: would not require hiding before the grab, but removes pins from external screen shares and recordings made by other apps.

### 5. Entry points are additive; no new action slot

**Toast**

- `Toast::open` gains a `pinnable: bool` (true from `Action::Saved`, false from `Action::Recorded`) so recording toasts do not offer Pin.
- The toast stores the saved path. On hover it paints a small rounded pin button (`E718`) over the thumbnail, pauses its existing expiry, and hit-tests `CursorMoved`. `Toast::event` returns `Outcome::{Pending, Close, Pin}`; Pin closes the toast and calls `Snap::open_pin(event_loop, path)`. A click elsewhere keeps the current Explorer behavior.
- The button is composed into a temporary pixel buffer only while hovered; the toast surface is small, so the copy is cheap. If the Fluent icon font is unavailable, the button is not drawn, matching the existing capture-toolbar fallback; gallery pinning remains available.

**Gallery**

- `gallery_pin(app, path)` in `gallery.rs` validates with the existing `guard` and `Kind::Photo`, then sends `Action::Pin(PathBuf)` through `State.proxy`. It decodes nothing and returns immediately.
- The frontend adds `api.galleryPin`, a `pin` action in `use-gallery` that calls `api.hide()` after success, a context-menu item for photos, and `pinImage` strings (en `Pin`, vi `Ghim`). The item menu is shared by the grid and viewer.
- The capture thread decodes in `Snap::open_pin` with `image::open`. A decode failure reports through the tray tooltip; the settings window has already hidden, which is accepted because the file came from a valid gallery photo.

No `Action` slot, `Mode`, hotkey, ring wedge or config field is added. `Action::Pin(PathBuf)` is the only new action variant.

### 6. The pin widget and event routing

`Pin` owns:

- `window`/`hwnd`, display `Pixmap`, original `RgbaImage`, toolbar rects and local scale.
- `hover`, dragged button, drag offset, `check_until`.
- `event(WindowEvent) -> Outcome` (`Pending`, `Close`, `Copy`), `deadline()` for the copy flash, `tick()` to clear it, `id()`.

`Snap` keeps `pins: Pins`; `window_event` finds a pin by `WindowId` before the overlay branch. On `Copy` it calls `capture::copy` with the original image and calls `pin.copied()` or sets the tooltip. On `Close` it removes the pin. `about_to_wait` adds pin deadlines to the existing toast/ring minimum and ticks expired pins. Pins are dropped with `Snap` when the capture loop ends; no cleanup file or registry entry exists.

## Risks / Trade-offs

- **[Layered redraw cost]** Every hover/check change recomposites and re-uploads the full pin pixmap; a 4K pin is ~33 MB per upload. → Redraw only when toolbar visibility, hover or the check state changes; keep a base pixmap and a composited copy instead of rebuilding per frame. Verify hover latency on a 4K image.
- **[Memory]** A pin keeps the full-resolution image for Copy plus a display pixmap, roughly doubling image memory; several 4K pins can be hundreds of MB. → Pins are ephemeral and user-created; no cap in this change. Note the cost in the design and revisit if users keep many large pins.
- **[Hidden-window ghosting]** xcap may capture a frame before DWM removes a just-hidden layered window. → Hide pins before the grab on the same thread, verify with a 4K/fullscreen capture; if a ghost appears, flush DWM or wait one frame before capture.
- **[Drag with NOACTIVATE]** `SetCapture` should deliver mouse moves on a non-activating layered window, but this path is new. → Verify drag across windows/monitors; fall back to polling `GetCursorPos` on a short timer while the button is held if events stop.
- **[Toolbar reachability for tiny images]** The window grows to the toolbar footprint, leaving transparent click-blocking margins. → Only visible for images smaller than the toolbar; accepted trade-off. Right-clicking is not used in this change, so no gesture conflicts.
- **[Icon font missing]** Toast pin and pin toolbar glyphs depend on Segoe Fluent Icons / Segoe MDL2, like the draw toolbar. → Reuse the existing fallback and hide the toast action if no icon font is available; gallery pinning still works.
- **[Recording reason drift]** Visibility relies on every recording starting through the overlay path and ending via `Recorded`/`RecordFailed`. → Keep the two start sites in `Snap::window_event` as the only setters; audit any future recording entry point.
- **[Always-on-top limits]** Fullscreen games and some topmost apps can cover pins; UAC/secure desktops are unaffected. → Accepted Windows behavior; pins are for normal desktop work.

## Migration Plan

None. Pins are in-memory only; there is no config, file format, registry or stored state to migrate or roll back.
