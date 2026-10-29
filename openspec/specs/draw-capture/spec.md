# draw-capture Specification

## Purpose
Defines the draw capture editor that opens after an area is selected with the draw hotkey: how the selection can still be adjusted, how the toolbar is laid out and placed, and how the user finishes, copies, pins or cancels the annotated image.

## Requirements

### Requirement: Selection can be resized after release
While annotating, the selection SHALL show eight handles, one on each corner and one in the middle of each edge. Dragging a handle SHALL resize the selection from that corner or edge, live. The selection SHALL stay inside the captured desktop and SHALL NOT become smaller than 3x3 physical pixels.

#### Scenario: Resize from a corner
- **WHEN** the user drags the bottom-right handle 100 px right and 50 px down
- **THEN** the selection grows by 100 x 50 px while the top-left corner stays fixed

#### Scenario: Handle takes priority over drawing
- **WHEN** the cursor is over a handle with any tool active
- **THEN** the cursor shows a resize arrow matching the handle and a press starts resizing instead of drawing

#### Scenario: Dragging past the opposite edge
- **WHEN** the user drags the left handle past the right edge
- **THEN** the selection does not invert and stays at least 3 px wide

### Requirement: Selection can be moved with Space
While annotating and no text box is open, holding Space and dragging inside the selection SHALL move the whole selection without changing its size. The selection SHALL stay inside the captured desktop.

#### Scenario: Move with Space held
- **WHEN** the user holds Space and drags inside the selection 200 px left
- **THEN** the selection moves 200 px left and shows the screen content under its new position

#### Scenario: Space while typing
- **WHEN** a text box is open and the user presses Space
- **THEN** a space is typed into the text box and the selection does not move

### Requirement: Shapes keep their screen position when the selection changes
When the selection is resized or moved, every drawn shape SHALL stay at the same position on screen. The saved, copied or pinned image SHALL contain only the parts of shapes that fall inside the final selection. Undo and redo SHALL keep working with shapes at their screen positions.

#### Scenario: Shape stays on its target
- **WHEN** the user draws an arrow pointing at a button and then extends the selection to the left
- **THEN** the arrow still points at the same button

#### Scenario: Shape partly outside
- **WHEN** a rectangle is half outside the selection after a resize and the user presses Done
- **THEN** the saved image contains only the half that is inside the selection

### Requirement: Selection size label
While annotating, the editor SHALL show the selection's width and height in physical pixels as `W × H` in a small label at the selection's top-left corner, above the selection when there is room and inside it otherwise. The label SHALL update live while resizing and SHALL NOT appear in the output image.

#### Scenario: Live size
- **WHEN** the user resizes the selection to 1280 by 720 physical pixels
- **THEN** the label reads `1280 × 720`

### Requirement: Fixed main toolbar row
The toolbar SHALL have a main row with, in this order: the rectangle, ellipse, arrow and text tools; Undo and Redo; Pin and Copy; Done and Cancel. The main row SHALL have no colour button and SHALL have the same buttons in the same positions for every tool and selection state.

#### Scenario: Switching tools does not move buttons
- **WHEN** the user switches from the rectangle tool to the text tool
- **THEN** Done and Cancel stay at the same screen position

#### Scenario: Undo and Redo disabled
- **WHEN** nothing has been drawn yet
- **THEN** Undo and Redo are shown dimmed and clicking them does nothing

### Requirement: Options row for the active tool
The toolbar SHALL show a second row with the options of the active tool or selected shape, hidden when there are none. The toolbar SHALL contain no text; every control is an icon with a tooltip. Colours SHALL be icon-only buttons drawn in the current colour: outline (hollow ring), fill (filled square, fill mode only) and text (filled circle). Shapes SHALL also show "Outline only" and "Outline and fill" buttons and widths.

#### Scenario: Rectangle options
- **WHEN** the rectangle tool is active
- **THEN** the options row shows the Outline only and Outline and fill buttons (tooltips "Outline only (F)" / "Chỉ viền (F)" and "Outline and fill (F)" / "Viền và tô (F)"), the outline colour button and three width buttons, and no fill colour button until fill mode is on

#### Scenario: Fill colour in fill mode
- **WHEN** the user clicks "Outline and fill" for a rectangle
- **THEN** a fill colour button (filled square, tooltip "Fill colour" / "Màu tô") appears after the outline colour button

#### Scenario: Text options
- **WHEN** the text tool is active
- **THEN** the options row shows the text colour button (filled circle, tooltip "Text colour" / "Màu chữ"), size down, the current size, size up, bold and italic

#### Scenario: Width buttons show thickness
- **WHEN** the options row shows widths
- **THEN** each width button draws a line at that width, thin to thick, with the current width marked

### Requirement: Colour palette replaces the options row
Clicking a colour button SHALL turn the options row itself into a palette row: that colour button (marked current), a divider and the 8 swatches, with no Back button and no caption. There SHALL be no separate palette popup, so at most two toolbar panels show at once, tooltips aside. A swatch pick SHALL apply the colour; clicking the colour button again, Esc, a tool switch and a click elsewhere SHALL NOT. All of them SHALL return to the options row.

#### Scenario: Open the palette
- **WHEN** the user clicks the outline colour button
- **THEN** the options row is replaced by that button (marked current), a divider and 8 swatches, and the main row is unchanged

#### Scenario: Pick a swatch
- **WHEN** the palette row is shown and the user clicks a swatch
- **THEN** the colour is applied, the options row returns and the colour button shows the new colour

#### Scenario: Dismiss without picking
- **WHEN** the palette row is shown and the user presses Esc, clicks the colour button again or clicks outside the toolbar
- **THEN** the options row returns, the colour is unchanged and the editor stays open

### Requirement: Toolbar placement
The toolbar SHALL be centred horizontally on the selection and placed below it, then above it when there is no room below, then inside its top edge, clamped to the monitor that holds the selection's centre. The options row SHALL sit on the side of the main row away from the selection. After a resize or move ends, the toolbar SHALL be placed again.

#### Scenario: Centred below
- **WHEN** the selection has room below it on its monitor
- **THEN** the main row is centred under the selection and the options row is under the main row

#### Scenario: Selection near the bottom
- **WHEN** the selection reaches the bottom of its monitor
- **THEN** the main row is above the selection and the options row is above the main row

### Requirement: Done saves the image
Done (the ✓ button, Enter or Ctrl+S) SHALL close the editor at once and save the annotated image through the normal save pipeline, which also copies it when "copy to clipboard" is on. Enter SHALL add a newline instead while a text box is open.

#### Scenario: Done
- **WHEN** the user clicks ✓
- **THEN** the editor closes, the image is saved to the output folder and the preview toast appears

### Requirement: Copy without saving
Copy (the toolbar button, or Ctrl+C when no text box is open) SHALL put the annotated image on the clipboard and close the editor without saving a file, without padding and without opening the toast. A clipboard failure SHALL be reported in the tray tooltip.

#### Scenario: Copy
- **WHEN** the user presses Ctrl+C in the editor
- **THEN** the editor closes, the annotated image is on the clipboard and no new file appears in the output folder

### Requirement: Pin without saving
Pin (the toolbar button or Ctrl+P) SHALL close the editor and open a screen pin with the annotated image, without saving a file and without padding. The pin SHALL behave like any other screen pin.

#### Scenario: Pin
- **WHEN** the user clicks Pin
- **THEN** the editor closes, a pin with the annotated image appears at the cursor and no new file appears in the output folder

### Requirement: Redo
The editor SHALL keep a redo history. Redo (the toolbar button, Ctrl+Y or Ctrl+Shift+Z) SHALL reapply the last undone change. Any new change (draw, move, delete or restyle) SHALL clear the redo history.

#### Scenario: Undo then redo
- **WHEN** the user draws a rectangle, presses Ctrl+Z, then Ctrl+Y
- **THEN** the rectangle is back

#### Scenario: New change clears redo
- **WHEN** the user undoes a shape and then draws a new one
- **THEN** Redo is disabled

### Requirement: Selection changes are not undoable
Resizing or moving the selection SHALL NOT be added to the undo history. Undo and redo SHALL change shapes only and SHALL keep the current selection.

#### Scenario: Undo after resize
- **WHEN** the user draws a shape, resizes the selection and presses Ctrl+Z
- **THEN** the shape is removed and the selection keeps its new size
