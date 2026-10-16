# Spec Delta

## ADDED Requirements

### Requirement: Record picker falls back to the full screen
In the picker opened by the record hotkey, a left click that is not on an application window SHALL close the picker and start recording the whole monitor under the cursor. The desktop counts as not an application window. The window screenshot picker SHALL NOT change.

#### Scenario: Click the desktop
- **WHEN** no recording is running, the user presses the record hotkey and left-clicks the desktop
- **THEN** the picker closes and recording of the monitor under the cursor starts, with the capture border around that monitor and the tooltip `snapr: recording…`

#### Scenario: Click a window
- **WHEN** the user left-clicks a highlighted application window in the record picker
- **THEN** that window is recorded, not the full screen

#### Scenario: Multiple monitors
- **WHEN** the user left-clicks the desktop on the second monitor in the record picker
- **THEN** only the second monitor is recorded, at that monitor's size rounded down to even width and height

#### Scenario: Stop a full-screen recording
- **WHEN** a full-screen recording is running and the user presses the record hotkey or chooses Stop recording
- **THEN** the recording stops, the MP4 is saved with the usual name, and the toast shows its last frame

#### Scenario: Window screenshot picker unchanged
- **WHEN** the user presses the window screenshot hotkey and left-clicks the desktop
- **THEN** no full-screen capture is taken and the picker behaves as before
