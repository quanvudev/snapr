# capture-padding Specification

## Purpose

Adds optional padding around every screenshot, filled with a user-selected background image or left transparent, so captures are ready to share without editing.

## Requirements

### Requirement: Padding is applied to every screenshot
When padding is enabled, fullscreen, window and area captures SHALL be saved, copied to the clipboard and shown in the preview with padding. The output SHALL measure `(w + 2p) x (h + 2p)` for a `w x h` capture and padding size `p`, with the capture unscaled at offset `(p, p)`. Recordings SHALL NOT be padded.

#### Scenario: Window capture with padding
- **WHEN** padding is enabled with size 64 and the user captures a 1200x800 window
- **THEN** the saved PNG is 1328x928 with the window image unscaled at (64, 64)

#### Scenario: Clipboard gets the padded image
- **WHEN** padding and copy to clipboard are both enabled and a capture is saved
- **THEN** the clipboard holds the same padded image as the saved PNG

#### Scenario: Fullscreen and area captures
- **WHEN** padding is enabled and the user takes a fullscreen or area capture
- **THEN** the saved image is padded the same way

#### Scenario: Padding disabled
- **WHEN** padding is disabled
- **THEN** captures are saved exactly as before, at their original size

#### Scenario: Recording unaffected
- **WHEN** padding is enabled and the user records the screen
- **THEN** the recording is not padded

### Requirement: Padding is transparent without a background image
When padding is enabled and no background image is selected, the padding area SHALL be fully transparent.

#### Scenario: No image selected
- **WHEN** padding is enabled and no background image is set
- **THEN** every padding pixel in the saved PNG has alpha 0

### Requirement: Background image fills the padded canvas
When a background image is selected, it SHALL be scaled to cover the whole padded canvas, keeping its aspect ratio, and center-cropped; the capture is drawn on top. PNG and JPEG images SHALL be supported.

#### Scenario: Wide wallpaper behind a tall capture
- **WHEN** a 1920x1080 background is selected and the padded canvas is 600x900
- **THEN** the background is scaled to cover 600x900 without distortion, centered, and cropped at the sides

#### Scenario: JPEG background
- **WHEN** the selected background is a JPEG file
- **THEN** it is used the same as a PNG

### Requirement: Screenshot corners can be rounded
When padding and rounded corners are both enabled, the screenshot's four corners SHALL be rounded with the configured radius (anti-aliased) before it is placed on the padded canvas, so the background or transparency shows through. The effective radius SHALL be capped at half the screenshot's shorter side. Rounding SHALL only remove pixels: corners already transparent stay transparent, and no outline is drawn. The outer canvas corners SHALL stay square.

#### Scenario: Area capture with rounded corners
- **WHEN** padding is enabled with a background image, rounded corners are enabled with radius 16, and the user captures an area
- **THEN** the screenshot's corners are rounded with a 16 px radius and the background shows through them

#### Scenario: Smaller radius than the window's own corners
- **WHEN** rounded corners are enabled with radius 4 and the user captures a window whose corners are already rounded more
- **THEN** the window's existing transparent corners stay transparent, with no dark pixels

#### Scenario: Radius larger than the screenshot
- **WHEN** the radius is larger than half of the screenshot's shorter side
- **THEN** the radius used is half of the shorter side

#### Scenario: Rounded corners disabled
- **WHEN** padding is enabled and rounded corners are disabled
- **THEN** the screenshot keeps its original corners

#### Scenario: Padding disabled
- **WHEN** padding is disabled and rounded corners are enabled
- **THEN** no rounding is applied

### Requirement: Unreadable background falls back to transparent
If the selected background image is missing or cannot be decoded, the capture SHALL still be saved with transparent padding, and the tray tooltip SHALL show the error.

#### Scenario: Background file deleted
- **WHEN** padding is enabled, the selected background file no longer exists, and the user takes a capture
- **THEN** the capture is saved with transparent padding and the tray tooltip shows `snapr: <error>`
