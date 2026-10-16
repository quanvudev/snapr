# Proposal

## Why

After a capture, snapr encodes the PNG file, then encodes a second PNG for the clipboard, and only then shows the preview toast. All of this runs on the capture event loop. On large screens the user waits with no feedback: the area/window overlay stays up after the click, and new hotkeys are ignored until both encodes finish.

## What Changes

- After the user picks an area or window, the overlay closes immediately. Producing the final image, writing the PNG and copying to the clipboard move to a background worker.
- The fullscreen hotkey uses the same background path: the screen is grabbed immediately, and the save happens in the background.
- The preview toast appears once the file has been written (so clicking it always opens an existing file), not before.
- While a save is in progress, the capture loop stays responsive: new hotkeys and overlays work right away.
- Captures are saved one at a time, in order. The output folder, timestamped file names and default PNG compression stay the same.
- Save errors are still reported through the tray tooltip.

## Capabilities

### New Capabilities
- `capture-output`: what happens after a capture is taken: saving the PNG, copying to the clipboard, showing the preview toast, and how responsive the app stays during these steps.

### Modified Capabilities
(none)

## Impact

- `src-tauri/src/lib.rs`: `Action` gets a "saved" event, a background save worker is fed by a channel, and `report()` is split into "queue the save" and "show the toast".
- `src-tauri/src/overlay.rs`: `Outcome::Save` hands over the work to do (a ready image, or a window handle with its crop fallback) instead of a finished image.
- `src-tauri/src/capture.rs`: `save()` and `window()` are reused unchanged, now called from the worker.
- No new dependencies, no config or frontend changes.
