# Proposal

## Why

Snapr's images have only two afterlives: a row in the gallery or a one-time clipboard paste. While working, users often need a screenshot as a persistent visual reference — a design to trace against, a dashboard to compare, an error dialog to keep in view. Reopening the file or recapturing breaks the flow. A pin turns a capture into an always-on-top desktop object that can be moved, compared and dismissed without leaving the current app.

## What Changes

- Add screen pins: a saved image becomes a frameless, always-on-top window with no focus stealing, shown at 1:1, centered on the cursor and clamped to the monitor work area. Images larger than the work area are scaled down to fit.
- Add entry points:
  - A Pin action on the screenshot preview toast, shown on hover, which closes the toast and pins the saved image.
  - A Pin action in the gallery item context menu for photo items, which hides the settings window and pins the file.
- Pin interactions: drag to move, a hover toolbar with Copy and Close, multiple pins at once, and pins disappear when Snapr quits. Copy places the full-resolution image on the clipboard.
- Hide pins while Snapr captures or records, so new screenshots and recordings never contain Snapr's own pins; pins stay visible to other applications, including screen shares. Restore them when the capture or recording ends.
- Explicitly out of scope: a dedicated pin hotkey or capture mode, wheel zoom, opacity, click-through, pinning from the clipboard, annotating on a pin, and persisting pins across restarts.

## Capabilities

### New Capabilities

- `screen-pin`: Creating, placing, rendering, moving, copying, closing and hiding pins, including the rules that keep Snapr's own captures and recordings free of pins.

### Modified Capabilities

- `capture-output`: The screenshot preview toast gains a Pin action; clicking the toast elsewhere still opens the saved file in Explorer.
- `gallery`: The item context menu gains a Pin action for photos, and pinning is limited to files inside the output folder like the other file actions.

## Impact

- Backend: a new `pin.rs` capture-thread window type; an `Action::Pin` path; `Snap` owns the live pins; visibility coordination around `Overlay::open`, fullscreen capture and recording; a `gallery_pin` command; a Pin control on the toast.
- Frontend: typed wrapper in `lib/api.ts`, a `pin` gallery action in `hooks/use-gallery.ts`, a menu entry in `components/settings/gallery/item-menu.tsx`, and new strings in `lib/i18n.ts`.
- No config schema, hotkey slots, dependencies, output formats or gallery file handling change. Pins are in-memory only.
- Windows-only, using the existing winit, tiny-skia and layered-window stack.
