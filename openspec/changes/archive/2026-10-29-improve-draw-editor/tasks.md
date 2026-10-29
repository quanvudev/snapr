# Tasks

No automated tests exist. Each task is verified with `cargo clippy` (in `src-tauri/`) plus a manual check in `bun run tauri dev` using the draw hotkey.

## 1. Sketch: redo and reselect

- [ ] 1.1 Add the `redo` stack, `redo()`, `can_undo()` and `can_redo()` to `Sketch`.
  - `snapshot()` clears `redo`.
  - A move takes its snapshot on the first real delta, so a click on a shape no longer pops history.
  - Verify: clippy is clean; draw, Ctrl+Z, then (temporarily wired) redo restores the shape.
- [ ] 1.2 Add `Sketch::reselect(frame, rect)`.
  - Re-crop the background, reallocate `base`/`view` and translate shapes, history, redo, the open text anchor and an active drag by the origin delta.
  - Verify: calling it with a shifted rect keeps shapes at the same screen position.

## 2. Selection adjusting in the overlay

- [ ] 2.1 Add `Handle` hit-testing (8 handles, ~10 logical px), the matching resize cursors and handle squares drawn by `outline()`. Verify: hovering each handle shows the right cursor.
- [ ] 2.2 Implement handle-drag resize.
  - Normalise to min 3x3, clamp to the frame and call `Sketch::reselect` live.
  - Call `Toolbar::place` again on release.
  - Verify: the "Resize from a corner", "Dragging past the opposite edge" and "Shape stays on its target" scenarios.
- [ ] 2.3 Implement the Space-held state (not while editing text) and Space-drag move inside the selection. Verify: the "Move with Space held" and "Space while typing" scenarios.
- [ ] 2.4 Add the cached `W × H` size label (above the selection, or inside near the monitor top), drawn only in `overlay::draw`. Verify: the label updates live and is absent from the saved PNG.

## 3. Two-row toolbar

- [ ] 3.1 Split `row()` into a constant `main_row()` and `options_row(kind, fill)`.
  - `Toolbar` keeps `main` and `options` rects (the main row has no colour button), centred on the selection with the combined-height placement.
  - Rows render into separate pixmaps blitted by the overlay.
  - `hit`, `slot` and `tooltip` search both rows.
  - Verify: switching tools never moves ✓/✗; the options row hides for no options; the "Centred below" and "Selection near the bottom" scenarios.
- [ ] 3.2 Add the `Undo`, `Redo`, `Pin` and `Copy` buttons with glyphs, en/vi tooltips and Save relabelled Done.
  - Add the disabled state (dim icon, no hover, no hit) driven by `can_undo`/`can_redo`.
  - Verify: "Undo and Redo disabled" before drawing.
- [ ] 3.3 Add the icon-only Outline-only / Outline-and-fill pair (hollow and filled tinted square, current one marked, en/vi tooltips) and the width buttons drawn at their real thickness. Verify: the "Rectangle options" and "Width buttons show thickness" scenarios.
- [ ] 3.4 Add the icon-only colour buttons and the inline palette row.
  - Buttons: outline colour (hollow ring), fill colour (filled square, fill mode only) and text colour (filled circle), with en/vi tooltips.
  - Clicking one replaces the options row with that button (marked current), a divider and the 8 swatches. Clicking it again, a swatch, Esc, a tool switch or anywhere else returns to the options row; remove the palette popup, `Back` and captions.
  - Verify: the "Colour palette replaces the options row" and "Rectangle options" scenarios; never more than two panels; no text drawn in the toolbar.

## 4. Actions and shortcuts

- [ ] 4.1 Add `Outcome::Copy(RgbaImage)` and `Outcome::Pin(RgbaImage)` from `Sketch::finish()`.
  - Handle them in `lib.rs`: `capture::copy` with tooltip on error; `pins.show_capture()` then `pins.add`.
  - Keep `self.style` in both cases.
  - Verify: the "Copy" and "Pin" scenarios; no new file in the output folder.
- [ ] 4.2 Wire the toolbar clicks and shortcuts:
  - Undo / Redo buttons, Ctrl+Y and Ctrl+Shift+Z.
  - Ctrl+C when no text box is open, and Ctrl+P.
  - Enter / Ctrl+S / ✓ unchanged.
  - Verify: the "Undo then redo", "New change clears redo", "Undo after resize" and "Done" scenarios; R/C/A/T/F still switch.

## 5. Docs and integration

- [x] 5.1 Update the draw capture paragraph and the `toolbar.rs` / `draw.rs` entries in `CLAUDE.md` (handles, Space-move, size label, two rows, Copy/Pin/Redo, Done). Verify: the text matches the shipped behaviour.
- [ ] 5.2 Do a full pass on a mixed-DPI, multi-monitor setup:
  - Select on the secondary monitor, resize across, Copy, Pin and Done.
  - Selection near each monitor edge.
  - `bunx tsc --noEmit` and `cargo clippy` clean.
  - Verify: no misplaced toolbar, label or handles.

## Workflow follow-up

- Archive the change with `/opsx:archive` after it ships; that creates `openspec/specs/draw-capture/spec.md`.
