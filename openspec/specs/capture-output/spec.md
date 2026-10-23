# capture-output Specification

## Purpose

Defines what happens after a capture is taken: saving the PNG, copying to the clipboard and showing the preview toast, while the app stays responsive during these steps.

## Requirements

### Requirement: Selection dismisses the overlay immediately
After the user selects an area or a window in the capture overlay, the overlay SHALL close at once, without waiting for the image to be produced, saved or copied.

#### Scenario: Area selected
- **WHEN** the user releases the mouse after dragging an area of at least 3x3 pixels
- **THEN** the overlay closes right away and the save continues in the background

#### Scenario: Window selected
- **WHEN** the user clicks a highlighted window
- **THEN** the overlay closes right away, and the window image is produced and saved in the background

#### Scenario: Cancel unchanged
- **WHEN** the user presses Escape or right-clicks in the overlay
- **THEN** the overlay closes and nothing is saved

### Requirement: Fullscreen capture saves in the background
The fullscreen hotkey SHALL grab the monitor under the cursor when the key is pressed, then save and copy the image in the background.

#### Scenario: Fullscreen hotkey
- **WHEN** the user presses the fullscreen hotkey
- **THEN** the screen contents at that moment are captured and the save continues in the background

### Requirement: Saving does not block new captures
While a capture is being saved, the app SHALL keep responding to capture hotkeys and overlay input.

#### Scenario: Hotkey during a save
- **WHEN** the user presses a capture hotkey while a previous capture is still being saved
- **THEN** the new capture starts immediately (the overlay opens, or the fullscreen grab happens)

### Requirement: Preview toast appears after the file is saved
The preview toast SHALL appear only after the capture's PNG file has been written, and only if the preview duration is greater than 0. Clicking the toast SHALL open the saved file in Explorer.

#### Scenario: Toast after save
- **WHEN** a capture's file has been written and the preview duration is greater than 0
- **THEN** the toast showing that capture appears in the bottom-right of the work area

#### Scenario: Toast does not wait for the clipboard
- **WHEN** copy to clipboard is enabled and a capture's file has been written
- **THEN** the toast appears without waiting for the clipboard copy to finish

#### Scenario: Preview disabled
- **WHEN** a capture's file has been written and the preview duration is 0
- **THEN** no toast appears

#### Scenario: Toast click
- **WHEN** the user clicks the toast
- **THEN** Explorer opens on the saved file, which exists

#### Scenario: Overlay open when save finishes
- **WHEN** a capture's file has been written while a capture overlay is open
- **THEN** no toast is shown for that capture

### Requirement: Captures are saved in order with unique names
Captures SHALL be saved one at a time in the order they were taken, into the configured folder, as PNG files with the default compression, named by local timestamp with a `_N` suffix when the name is taken.

#### Scenario: Two captures in the same second
- **WHEN** the user takes two captures within the same second
- **THEN** both files are kept, the second named with a `_1` suffix

#### Scenario: Clipboard holds the latest capture
- **WHEN** copy to clipboard is enabled and several captures are saved one after another
- **THEN** after the last save finishes, the clipboard holds the most recent capture

### Requirement: Save errors are reported in the tray tooltip
If producing or saving a capture fails, the app SHALL show the error in the tray tooltip and show no toast for that capture. If only the clipboard copy fails, the app SHALL show the error in the tray tooltip and still show the toast for the saved file. A successful save SHALL restore the normal tooltip.

#### Scenario: Folder not writable
- **WHEN** the output folder cannot be created or written
- **THEN** the tray tooltip shows `snapr: <error>` and no toast appears

#### Scenario: Clipboard copy fails
- **WHEN** copy to clipboard is enabled, the file is saved, and copying to the clipboard fails
- **THEN** the toast for the saved file is shown and the tray tooltip shows `snapr: <error>`

#### Scenario: Successful save after an error
- **WHEN** a capture is saved successfully after an earlier failure
- **THEN** the tray tooltip returns to the normal status text

### Requirement: Overlay covers exactly the monitor under the cursor
The capture overlay SHALL open on the monitor under the cursor and cover exactly that monitor's bounds, whatever that monitor's display scaling is relative to the primary monitor.

#### Scenario: Secondary monitor with different scaling
- **WHEN** the user presses the window or area hotkey with the cursor on a monitor whose scaling differs from the primary monitor
- **THEN** the overlay opens on that monitor, covers its full area, and does not extend onto another monitor

#### Scenario: Selection matches what is shown
- **WHEN** the user selects an area or a window in that overlay
- **THEN** the saved image matches the region highlighted in the overlay

### Requirement: Preview toast shows the thumbnail on every monitor
The preview toast SHALL show the capture thumbnail on any monitor, whatever that monitor's display scaling is relative to the primary monitor.

#### Scenario: Toast on a secondary monitor with different scaling
- **WHEN** a capture is saved while the cursor is on a monitor whose scaling differs from the primary monitor
- **THEN** the toast appears in the bottom-right of that monitor's work area and shows the capture thumbnail, not a blank window
