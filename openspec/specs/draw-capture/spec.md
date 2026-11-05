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
The toolbar SHALL have a main row with, in this order: the rectangle, ellipse, arrow, text and redact tools; Undo and Redo; Pin and Copy; Done and Cancel. The main row SHALL have no colour button and SHALL have the same buttons in the same positions for every tool and selection state.

#### Scenario: Switching tools does not move buttons
- **WHEN** the user switches from the rectangle tool to the text tool
- **THEN** Done and Cancel stay at the same screen position

#### Scenario: Undo and Redo disabled
- **WHEN** nothing has been drawn yet
- **THEN** Undo and Redo are shown dimmed and clicking them does nothing

#### Scenario: Redact tool position
- **WHEN** the toolbar is shown
- **THEN** the redact tool (tooltip "Redact (X)" / "Che thông tin (X)") is right after the text tool and before Undo

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

### Requirement: Redaction boxes
With the redact tool, a drag SHALL add a redaction box over the dragged rectangle. A box SHALL be Black (solid black) or Pixelate (square blocks of half the box's shorter side, at least 16 physical px, each the average colour of the frozen capture under it). A box SHALL hide the capture under it in the view and in every saved, copied or pinned image, and SHALL be selectable, movable, deletable, restylable and undoable like other shapes. Shapes drawn later SHALL be drawn over it.

#### Scenario: Draw a black box
- **WHEN** the redact tool is active and the user drags over an email address and presses Enter
- **THEN** the saved PNG has a solid black rectangle where the email was

#### Scenario: Pixelate
- **WHEN** the user picks Pixelate in the options row and drags over text
- **THEN** the area shows large coloured blocks and the text cannot be read

#### Scenario: Restyle a box
- **WHEN** the user selects a black redaction box and picks Pixelate
- **THEN** that box becomes pixelated and Pixelate becomes the current redaction style for the rest of this capture

### Requirement: Default redaction style
Each Draw capture SHALL start with the redaction style set in the redaction settings (Black when unset). Changing the style in the toolbar SHALL last only until that capture is saved, copied, pinned or cancelled, and SHALL NOT change the setting.

#### Scenario: Pixelate by default
- **WHEN** the default style setting is Pixelate and the user opens a Draw capture and drags with the redact tool
- **THEN** the new box is pixelated

#### Scenario: Toolbar choice is per capture
- **WHEN** the default style is Black, the user picks Pixelate in the toolbar, saves, and opens a new Draw capture
- **THEN** the redact tool starts with Black again

#### Scenario: Arrow over a box
- **WHEN** the user draws an arrow across an existing redaction box
- **THEN** the arrow is visible on top of the box

### Requirement: Redact options row
While the redact tool is active or a redaction box is selected, the options row SHALL show Black, Pixelate (the current one marked) and Detect. Detect SHALL be dimmed and ignored when no OCR is available, when every category is off and there is no valid custom pattern, or while a detection is already running. The key X SHALL switch to the redact tool, except while a text box is open.

#### Scenario: Options
- **WHEN** the redact tool is active
- **THEN** the options row shows Black, Pixelate and Detect with tooltips "Black" / "Tô đen", "Pixelate" / "Làm mờ khối" and "Detect sensitive info (D)" / "Tìm thông tin nhạy cảm (D)"

#### Scenario: No OCR
- **WHEN** neither a Windows OCR language nor a ready PaddleOCR model is available
- **THEN** Detect is dimmed, nothing is detected, and redaction boxes can still be drawn by hand

#### Scenario: Nothing to detect
- **WHEN** every category is turned off in the redaction settings and no custom pattern is set
- **THEN** Detect is dimmed and pressing D detects nothing

### Requirement: Detecting sensitive text
Detect SHALL recognise the text of the current selection without blocking the overlay and add one redaction box in the current style over each match, padded by 2 px and clipped to the selection, as one undoable step. A match already covered by a redaction box SHALL be skipped. Draw capture SHALL NOT run Detect on its own; it runs only when the user presses Detect or D. A result arriving after the overlay closed or the selection changed SHALL be dropped.

#### Scenario: Detect
- **WHEN** the user selects an area showing `alice@example.com` and `192.168.1.20` and presses D
- **THEN** after a moment two redaction boxes cover the email and the IP, and one Ctrl+Z removes both

#### Scenario: No detection on tool switch
- **WHEN** the user switches to the redact tool
- **THEN** nothing is detected until the user presses Detect or D

#### Scenario: Overlay stays usable
- **WHEN** detection is running
- **THEN** the user can still draw, move the selection, save or cancel, and the Detect button shows that it is busy

#### Scenario: Remove a false positive
- **WHEN** a detected box covers a harmless number
- **THEN** the user selects it, presses Delete and only that box is removed

#### Scenario: Rescan
- **WHEN** detection already ran and the user enlarges the selection and presses Detect
- **THEN** only matches not already covered get new boxes

#### Scenario: Closed before the result
- **WHEN** the user presses Enter while detection is still running
- **THEN** the image is saved with the boxes present at that moment and the late result is ignored

#### Scenario: Nothing found
- **WHEN** Detect finds no match
- **THEN** no box is added, no undo step is recorded and the tooltip area briefly shows "Nothing found" / "Không tìm thấy"

### Requirement: Detected categories
Detection SHALL match, each only while its category is on in the redaction settings, email addresses; phone numbers (9 to 15 digits, optional leading `+`, with spaces, dots, dashes or parentheses between groups); payment card numbers (13 to 19 digits, spaces or dashes allowed, passing the Luhn check); and IPv4 addresses (four octets 0 to 255). A box SHALL cover only the matched characters of a line, not the whole line.

#### Scenario: Card number
- **WHEN** the selection shows `4111 1111 1111 1111` and `1234 5678 9012 3456`
- **THEN** only `4111 1111 1111 1111` is covered, since the other fails the Luhn check

#### Scenario: Part of a line
- **WHEN** a line reads `Contact: bob@corp.vn for access`
- **THEN** the box covers `bob@corp.vn` and leaves `Contact:` and `for access` visible

#### Scenario: Category off
- **WHEN** the phone category is off and the selection shows `+84 912 345 678` and `alice@example.com`
- **THEN** only the email is covered

#### Scenario: Version numbers are not IPs
- **WHEN** the selection shows `v1.2.3` and `300.1.1.1`
- **THEN** neither is covered

### Requirement: Detected secrets
While the secrets category is on, detection SHALL also match secrets: tokens with a well-known prefix (`sk-`, `ghp_`, `gho_`, `github_pat_`, `xoxb-`, `xoxp-`, `AKIA`, `AIza`), JWTs (three base64url parts starting with `eyJ`) and strings of 32 or more letters and digits that mix both.

#### Scenario: API key
- **WHEN** the selection shows `OPENAI_API_KEY=sk-proj-abc123def456ghi789`
- **THEN** the key value starting at `sk-` is covered and `OPENAI_API_KEY=` stays visible

#### Scenario: Commit hash
- **WHEN** the selection shows a 40-character git commit hash
- **THEN** it is covered as a generic secret

### Requirement: Custom redaction patterns
Detection SHALL also match every custom pattern from the redaction settings against each recognised line, exactly as written (a leading `(?i)` makes it case-insensitive), covering only the matched characters. A stored pattern that is not a valid regular expression SHALL be skipped without an error. The settings and patterns SHALL be read when each detection starts.

#### Scenario: Employee ID
- **WHEN** the custom patterns include `EMP-\d{6}` and the selection shows `Owner: EMP-204518`
- **THEN** `EMP-204518` is covered and `Owner:` stays visible

#### Scenario: Invalid stored pattern
- **WHEN** the config file was edited by hand to hold the pattern `(abc` and a valid pattern `ACME-\d+`
- **THEN** detection covers `ACME-` matches and shows no error

#### Scenario: Change applies to the next detection
- **WHEN** the user adds a pattern in Settings while a Draw capture is open and then presses Detect
- **THEN** that detection uses the new pattern
