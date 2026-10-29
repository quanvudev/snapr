# Design

## Context

The draw editor lives in three capture-thread modules:

- `overlay.rs` owns the `Annotation { sketch, toolbar, hover, palette, icon }` phase. It routes mouse and keys and returns `Outcome::Save(Shot::Ready(..))` on save.
- `draw.rs` holds `Sketch`. It has a fixed `selection`, a `background` crop of the frozen frame, shapes in selection-local coordinates, an undo `history` of `Vec<Shape>` snapshots and a cached `base` / `view` pixmap pair sized to the selection.
- `toolbar.rs` builds one slot row with `row(kind, fill)` (tools + colour + per-tool options + Save/Cancel). `layout` lays it out, `Toolbar::place` positions it right-aligned and sizes its pixmaps once for the widest row. `panel()` right-aligns the actual row inside that width, which is why the buttons move.

The overlay keeps the full frozen `frame`, so the selection can be re-cropped at any time without capturing again. See `proposal.md` for the motivation and `specs/draw-capture/spec.md` for the required behaviour.

## Goals / Non-Goals

**Goals:**
- Selection editing reuses the existing frozen frame and `Sketch` state, with no new windows.
- The toolbar keeps one panel renderer (`paint_panel`, tooltips) for both rows and the inline palette row.

**Non-Goals:**
- Storing shapes in overlay coordinates (see Decision 2).
- Moving the save, copy or pin work off the capture thread beyond what exists today.

## Decisions

### 1. Selection handles live in `overlay.rs`

The overlay gets an `Adjust` drag state: `Resize(Handle)` or `Move`, plus the start cursor and the start selection.
- `Handle` is one of the 8 positions. Hit-testing uses a square of about 10 logical px centred on each handle point, checked before the toolbar and before `Sketch::press`.
- Move starts only when Space is held (tracked from `KeyboardInput` pressed/released while no text box is open) and the press is inside the selection.
- While dragging, the overlay computes the new rect, normalises it (min 3x3, clamped to the frame) and calls `Sketch::reselect`. The cursor uses the matching `CursorIcon` (`NwResize`, `NResize`, ... , `Move`).
- On release it calls `Toolbar::place` again with the new selection.
- `outline()` also draws the handles as small filled squares in `BORDER` colour.

Alternative: put handles inside `Sketch`. Rejected, because `Sketch` is about shapes and the overlay already owns the frame, monitors and cursor icons.

### 2. `Sketch::reselect` shifts shapes instead of changing the coordinate space

`reselect(frame, rect)` re-crops `background` from the frame, reallocates `base` and `view` at the new size and translates every shape by `(old.x - new.x, old.y - new.y)` with the existing `Shape::translate`. It does the same for every snapshot in `history` and `redo`, and for an open text box anchor and an in-progress drag. Then it marks the sketch `dirty`.

Alternative: keep shapes in overlay coordinates and render with an offset. That touches `hit`, `bounds`, `render`, `constrain` and text placement. Translating on the rare reselect is much smaller and keeps every other code path unchanged. Clipping is free, because shapes render into a pixmap of the selection size.

Re-cropping reuses `overlay::crop`. During a drag this runs on every `CursorMoved`. A crop of a 4K area is a memcpy of about 33 MB, which is acceptable. If it stutters, an `Option<Rect>` preview can coalesce updates to `RedrawRequested`.

### 3. Redo stack in `Sketch`

Add `redo: Vec<Vec<Shape>>` to `Sketch`:
- `snapshot()` clears it (every mutation goes through `snapshot`).
- `undo` pushes the current shapes onto `redo` before restoring.
- `redo()` pushes the current shapes onto `history` and restores from `redo`, without clearing it.
- An undo that only discards an open text box does not touch `redo`.
- `can_undo()` / `can_redo()` drive the disabled state.
- The existing "a move with no movement pops its snapshot" path must restore the `redo` stack it cleared, so the snapshot is taken lazily on the first real move delta instead of on press.

### 4. Two-row toolbar with a fixed main row

`row(kind, fill)` becomes two functions:
- `main_row()`: a constant slot list with no colour button. Tools, divider, `Undo`, `Redo`, divider, `Pin`, `Copy`, divider, `Save` (the ✓, accent), `Cancel`.
- `options_row(kind, fill, palette) -> Vec<Slot>`: for shapes the Outline-only / Outline-and-fill icon pair, the outline colour button, the fill colour button in fill mode only, then widths; for the arrow the outline colour button and widths; for text the text colour button, size down / current / up, bold and italic. Empty when there are no options. While `palette` is `Some`, it returns the palette row instead: the active colour button (marked current), a divider and the 8 `Swatch(i)` buttons. `layout(rows, scale)` takes no language: there is no text in the toolbar.


`Toolbar` keeps two rects:
- `main` is centred on the selection: `x = sx + sw/2 - w/2`, clamped to the monitor.
- `options` is left-aligned with `main`'s left edge, one `GAP` away on the far side from the selection.

Placement checks the combined height (main + gap + widest options row) for the below/above/inside choice, so the options row appearing or disappearing never flips the side. Each row is painted into its own pixmap with `paint_panel`. `origin()` becomes `origins()`, returning each visible row's pixmap and position for `overlay::draw` to blit. `hit`, `slot` and `tooltip` search both rows. There is no palette popup. `Toolbar` holds an optional open `Palette` (`Outline` / `Fill` / `Text`). While it is `Some`, the options panel is painted from `palette_row` instead of `options_row`, in the same rect, so at most two panels exist (tooltips aside). A swatch pick applies the colour and clears it; Clicking the same colour button again, Esc, a tool switch and any click elsewhere clear it without changing the colour. The palette row is shown even when the options row would otherwise be empty, and the placement height uses the same combined-height rule, so opening it never flips the side.

Alternative: one row with a fixed slot count, using blank placeholders for missing options. Rejected, because it leaves gaps and makes the bar wider than needed.

### 5. Icon-only Outline / Fill, colour buttons and width previews

- The Outline / Fill pair is two icon buttons, `Button::Outline` (hollow square) and `Button::Fill` (filled tinted square with an outline), with tooltips "Outline only (F)" / "Chỉ viền (F)" and "Outline and fill (F)" / "Viền và tô (F)". The current one is marked with the accent bar like the active tool.
- Colour buttons are icon-only and draw their swatch in the current colour: `Button::Colour` is a hollow ring (outline colour) or a filled circle (text tool), `Button::FillColour` a filled square. Tooltips: "Outline colour" / "Màu viền", "Fill colour" / "Màu tô", "Text colour" / "Màu chữ". Clicking one opens the palette row for that colour; while open, that button is marked current. There is no `Back` button and no caption.
- `Width(i)` draws a horizontal rounded bar of height `WIDTHS[i]` (scaled), centred in the button. This replaces the current fixed-height bars.

### 6. New buttons, glyphs and disabled state

- New `Button`s: `Undo` (`E7A7`), `Redo` (`E7A6`), `Copy` (`E8C8`, same as the pin), `Pin` (`E718`).
- `render` takes an `enabled(Button) -> bool` closure. Disabled buttons draw the icon at `DISABLED_ALPHA`, ignore hover and are skipped by `hit`.
- Tooltips get en/vi labels with shortcuts:
  - Undo (Ctrl+Z) / Hoàn tác
  - Redo (Ctrl+Y) / Làm lại
  - Pin (Ctrl+P) / Ghim
  - Copy (Ctrl+C) / Sao chép
  - Done (Enter) / Xong
- `Save` keeps its accent look and is relabelled "Done".

### 7. Copy and Pin as new overlay outcomes

`Outcome` gains `Copy(RgbaImage)` and `Pin(RgbaImage)`, produced by `Sketch::finish()` like `save`. In `lib.rs`:
- `Copy` calls `capture::copy` on the capture thread (as the pin Copy button already does) and reports errors via `tooltip`.
- `Pin` calls `self.pins.add(event_loop, image, ocr)` after `self.pins.show_capture()`, so the new pin is visible.

Both restore `self.style` from the overlay like save and cancel do. Neither touches the save worker, so there is no padding, no file and no toast. This matches the screen-pin behaviour of keeping in-memory images.

Alternative: send Copy through the save worker with a "no file" flag. Rejected, because it would complicate `work()` for no gain; copying a single annotated area is fast.

### 8. Size label

Add a small label pixmap: `W × H` in Segoe UI 12 px on a `PANEL` rounded rect, the same rendering helper style as tooltips. It is cached by `(w, h)` and blitted at the selection's top-left, above it when `sy - h - margin >= monitor top`, inside otherwise. It is drawn in `overlay::draw` only, never in `Sketch`, so it never reaches the output.

### 9. Shortcuts

In `key()`:
- Ctrl+C → Copy, but not while a text box is open (then it does nothing, since text boxes have no selection).
- Ctrl+P → Pin.
- Ctrl+Y and Ctrl+Shift+Z → Redo. Ctrl+Z without Shift stays Undo.
- Space press/release sets `space` only while `!sketch.editing()`. Space while editing still types.

## Risks / Trade-offs

- [Re-crop on every move can stutter on very large areas] → Measure on a 4K monitor. If needed, update the crop only on `RedrawRequested` with the latest pending rect.
- [Handle hit zones overlap shapes drawn at the selection edge] → Handles win; shapes there can still be selected from their inner part or moved after shrinking the hit zone. This is acceptable for a 10 px zone.
- [Space-drag to move is not discoverable] → The CHANGELOG entry states it. Resizing with the handles covers most corrections without it.
- [Toolbar width grows with four new buttons] → The main row is about 13 buttons ≈ 540 logical px, still narrower than today's widest row plus margins on any monitor ≥ 1024 px. Clamping keeps it on screen for small selections.
- [Translating history snapshots on each reselect costs O(history × shapes)] → History is small (tens of snapshots, few shapes). This is negligible.
