## MODIFIED Requirements

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

## ADDED Requirements

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
