# Proposal

## Why

The draw capture editor is hard to use. The selected area is locked the moment the mouse is released, so a slightly wrong selection means pressing Esc and starting over. The toolbar mixes tools and tool options in one right-aligned row, so its length changes and Save/Cancel move whenever the tool changes. Several icons (Outline vs Fill, the three widths) are hard to tell apart. Undo, redo, copy and pin are reachable only through shortcuts or not at all.

## What Changes

- The selection stays editable after release. Eight handles on its corners and edges resize it, and holding Space while dragging moves it. The selection's size in physical pixels is shown next to it. Shapes already drawn keep their position on screen when the selection changes.
- The toolbar splits into two rows:
  - A main row whose buttons and positions never change: the four tools, Undo and Redo, Pin and Copy, then Done and Cancel. It has no colour button.
  - An options row for the active tool (or the selected shape), shown next to the main row only when that tool has options. Colours live here as icon-only buttons: a hollow ring (outline colour), a filled square (fill colour) or a filled circle (text colour), each in its current colour.
  - Clicking a colour button turns the options row itself into a palette row (that colour button, a divider and the 8 swatches) instead of opening a popup. Clicking the colour button again, a swatch, Esc, a tool switch or anywhere else closes it. At most two toolbar panels are shown at once.
  - The toolbar has no text: every control is an icon with a tooltip.
- The toolbar is centred horizontally on the selection instead of right-aligned, and is placed again after the selection is resized or moved.
- New actions:
  - Redo (toolbar, Ctrl+Y and Ctrl+Shift+Z).
  - Copy (toolbar, Ctrl+C) puts the annotated image on the clipboard and closes the editor without saving a file.
  - Pin (toolbar, Ctrl+P) pins the annotated image on screen and closes the editor without saving a file.
- Undo and Redo are shown disabled when there is nothing to undo or redo.
- Outline and Fill become two icon buttons (hollow square, filled tinted square) with tooltips; the current one is marked. Width buttons draw a line of the real thickness.
- Done (✓, Enter, Ctrl+S) keeps today's behaviour: save the file through the normal save pipeline, which also copies when "copy to clipboard" is on.
- Out of scope: new drawing tools (blur, highlighter, step numbers), undo of selection changes, a separate "save without copying" button, and theming the editor.

## Capabilities

### New Capabilities

- `draw-capture`: The draw capture editor after an area is selected: adjusting the selection, the toolbar layout and placement, and the Done, Copy, Pin, Cancel, Undo and Redo actions. It is already referenced by `capture-output` but has no spec yet. This change specifies only the behaviour it introduces or changes.

### Modified Capabilities

(none — `capture-output` already defers draw capture details to `draw-capture`, and its "Draw capture saved" scenario still holds.)

## Impact

- `src-tauri/src/overlay.rs`: selection handles, Space-drag moving, the size label, new outcomes for Copy and Pin, and new shortcuts.
- `src-tauri/src/draw.rs`: `Sketch` can change its selection (re-crop the background, shift shapes and history) and gains a redo stack.
- `src-tauri/src/toolbar.rs`: two-row layout, centred placement, the new buttons, disabled state, icon-only Outline / Fill and colour buttons, an inline palette row, width previews, tooltips (en/vi).
- `src-tauri/src/lib.rs`: handle the new overlay outcomes. Copy uses `capture::copy` and Pin uses the existing `Pins::add`; neither goes through the save worker.
- No config, settings UI or frontend changes.
