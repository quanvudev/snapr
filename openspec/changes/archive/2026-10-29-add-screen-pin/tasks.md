# Tasks

## 1. Pin window and gallery entry

- [x] 1.1 Expose the ring's layered-window helpers (`bitmap`, `present`) for reuse by making them crate-visible; verify `cargo clippy` in `src-tauri/` and that the action ring still opens from its hotkey.
- [x] 1.2 Create `src-tauri/src/pin.rs` with a `Pin` that opens a frameless, topmost, non-activating layered winit window, renders an `RgbaImage` with per-pixel alpha, sizes it at 1:1 (downscaled to fit the cursor monitor's work area), centers it on the cursor and clamps it to the work area; verify `cargo clippy`.
- [x] 1.3 Add `Pins` with a `Vec<Pin>` and independent capture/recording hide reasons, and wire `Snap.pins`, `WindowId` routing, pin close/drop and creation-while-hidden behavior; verify `cargo clippy`.
- [x] 1.4 Add `Action::Pin(PathBuf)`, decode in `Snap::open_pin` with `image::open`, and report decode failures in the tray tooltip; verify `cargo clippy`.
- [x] 1.5 Add `gallery_pin` in `gallery.rs` (existing `guard` + `Kind::Photo` + `State.proxy`), register it, then add `api.galleryPin`, the `pin` action in `use-gallery` (hide settings after success), a Pin menu item for photos in `item-menu.tsx`, and `pinImage` en/vi strings; verify `bunx tsc --noEmit`.
- [x] 1.6 Manually pin a gallery photo and verify the settings window hides, the pin appears centered on the cursor, is always on top, is absent from the taskbar and Alt-Tab, and does not take focus from a foreground app; use `__TAURI_INTERNALS__.invoke("gallery_pin", ...)` in the devtools console to confirm a video and an outside path are refused.

## 2. Toast entry

- [x] 2.1 Add `pinnable` to `Toast::open` and pass true from `Action::Saved` and false from `Action::Recorded` through `Snap::preview`; verify a recording toast has no Pin action and a screenshot toast does.
- [x] 2.2 Add the hover Pin button (glyph `E718`) to `toast.rs` with hover and hit state and an outcome that pins the saved path and closes the toast, keeping a click elsewhere opening Explorer; verify `cargo clippy` and manually pin from a screenshot toast.
- [ ] 2.3 Verify the toast button survives the existing hover pause, works on a secondary/high-DPI monitor, and disappears with the toast; verify the gallery entry still works when preview is disabled.

## 3. Interactions

- [ ] 3.1 Implement dragging with `SetCapture`/`GetCursorPos`/`set_outer_position`, raise the pin on press, and exclude toolbar presses from dragging; verify dragging across monitors and that typing continues in another app.
- [ ] 3.2 Draw the hover toolbar (Copy `E8C8`, Close `E711`, dark pill, hover states) with hit-testing and a minimum window footprint so the actions stay reachable for a 16x16 image; verify hover show/hide and closing one pin among several.
- [x] 3.3 Implement Copy with `capture::copy` on the original full-resolution image, a one-second check flash (`E73E`) scheduled through `about_to_wait`, and a tray tooltip on failure; verify pasting a scaled pin's copy yields the full-resolution image and the pin stays open.
- [x] 3.4 Verify multiple pins: create two, move them independently, confirm the newer is above the older, close one, and quit Snapr; verify all pins disappear and the source files keep their timestamps and content.
- [x] 3.5 Draw a 2px accent-blue border around the pinned image, outside the image pixels, and account for it in window sizing, image centering and clamping; verify a pin whose edges match the desktop is visibly framed and Copy excludes the border.
- [x] 3.6 Cap a pin's larger side at about 250 logical px (scaled by monitor DPI), preserving aspect ratio, and keep smaller images at 1:1; verify a fullscreen pin is about 250x141 and a small image is unchanged.

## 4. Capture and recording visibility

- [x] 4.1 Hide pins around `Snap::pick`/`Overlay::open` and `Action::Fullscreen`, restoring them on save, cancel or a failed open while keeping them hidden when a picker starts a recording; verify area, window, text and draw captures plus fullscreen saves contain no pin, and pins return after each interaction.
- [x] 4.2 Hide pins when `record(...)` starts and restore them on `Action::Recorded`/`RecordFailed`; verify window and monitor recordings contain no pin, pins return on stop and failure, and a screenshot taken during a recording leaves pins hidden until the recording ends.
- [x] 4.3 Verify a pin created while pins are hidden stays hidden until the hiding reason ends, and that pins are visible in an external capture (for example Snipping Tool) while no Snapr capture or recording is active.
- [x] 4.4 Inspect a saved fullscreen and area capture taken with a large pin on screen for hidden-window ghosting; if a ghost appears, add a DWM flush or one-frame wait before the grab and re-verify.
- [x] 4.5 Verify by inspection of both the capture and recording start failure paths that the hide reason is cleared and the tray tooltip is shown; note any path that cannot be exercised manually.

## 5. Integration

- [ ] 5.1 Verify mixed-DPI behavior on a 100% and a 200% monitor: 1:1 image size, cursor centering, clamping, toolbar usability, and dragging across monitors keep the physical image size.
- [ ] 5.2 Check 4K pin creation, hover, copy flash and drag smoothness; if hover redraws stutter, cache the composited frame or avoid rebuilding the base image and re-verify.
- [x] 5.3 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`, then a `bun run tauri dev` smoke pass covering every scenario in `specs/screen-pin/spec.md`, `specs/capture-output/spec.md` and `specs/gallery/spec.md`; record any gap.
- [x] 5.4 Update `CLAUDE.md` with the pin module, the two entry points and the hide-during-capture/recording rule; verify the description matches the implemented behavior.

## Workflow follow-up

- Run `openspec validate add-screen-pin --strict` and resolve any warnings before review.
- Archive the change after the implementation is reviewed.
