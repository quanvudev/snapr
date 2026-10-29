# screen-recording Specification

## Purpose
Defines how snapr records the contents of a picked window to an MP4 video file: starting and stopping a recording, how the user knows a recording is running, and how the finished file is saved and previewed.

## Requirements

### Requirement: Record hotkey starts a window recording
When no recording is running, pressing the record hotkey SHALL open the window picker. Clicking a highlighted window SHALL close the picker and start recording that window. Dragging a rectangle with the left mouse button SHALL close the picker on release and start recording that area. Cancelling the picker SHALL start nothing.

#### Scenario: Pick a window
- **WHEN** no recording is running and the user presses the record hotkey and clicks a highlighted window
- **THEN** the picker closes and recording of that window starts

#### Scenario: Drag an area
- **WHEN** no recording is running and the user presses the record hotkey and drags a rectangle over part of a window
- **THEN** the picker closes on release and recording of that rectangle starts, not of the window under it

#### Scenario: Cancel the picker
- **WHEN** the user presses Escape or right-clicks in the picker opened by the record hotkey
- **THEN** the picker closes and no recording starts

#### Scenario: Recording cannot start
- **WHEN** the picked window cannot be captured or the output file cannot be created
- **THEN** no recording runs and the tray tooltip shows `snapr: <error>`

### Requirement: Only one recording runs at a time
While a recording is running, the record hotkey SHALL stop it and SHALL NOT open the picker.

#### Scenario: Record hotkey while recording
- **WHEN** a recording is running and the user presses the record hotkey
- **THEN** the recording stops and no picker opens

### Requirement: Recording can be stopped from the tray
The tray menu SHALL have a Stop recording item that is enabled only while a recording is running and stops it.

#### Scenario: Stop from the tray
- **WHEN** a recording is running and the user chooses Stop recording in the tray menu
- **THEN** the recording stops

#### Scenario: Nothing to stop
- **WHEN** no recording is running
- **THEN** the Stop recording item is disabled

### Requirement: A running recording is visible
While a recording is running, Windows SHALL draw its capture border around the recorded window or monitor and the tray tooltip SHALL read `snapr: recording…`. While an area recording is running, snapr SHALL also draw a frame just outside the recorded area that does not appear in the recording. When the recording ends the frame SHALL disappear and the tooltip SHALL return to the normal status text.

#### Scenario: During recording
- **WHEN** a recording is running
- **THEN** the recorded window has the capture border and the tray tooltip reads `snapr: recording…`

#### Scenario: During an area recording
- **WHEN** an area recording is running
- **THEN** a frame is drawn around the recorded area and the saved video does not contain that frame

#### Scenario: After recording
- **WHEN** the recording ends successfully
- **THEN** the border and any area frame disappear and the tray tooltip returns to the normal status text

### Requirement: Recording ends when the window closes
If the recorded window is closed, the recording SHALL end and the file SHALL be finalized as if the user had stopped it.

#### Scenario: Window closed
- **WHEN** the recorded window is closed during a recording
- **THEN** the recording ends and a playable MP4 file is saved

### Requirement: Quitting finalizes a running recording
Choosing Quit while a recording is running SHALL stop it and finalize the file before the app exits.

#### Scenario: Quit while recording
- **WHEN** a recording is running and the user chooses Quit in the tray menu
- **THEN** a playable MP4 file is saved and the app exits

### Requirement: Recordings are saved as MP4 with unique names
A recording SHALL be saved in the configured output folder as an H.264 MP4 file at the configured frame rate (30 or 60 frames per second), with the cursor and without audio. It SHALL be named by the local timestamp when recording started, with a `_N` suffix when the name is taken. A recording SHALL NOT be copied to the clipboard.

#### Scenario: File saved
- **WHEN** a recording ends
- **THEN** a `.mp4` file named by the start timestamp exists in the output folder and plays in a standard video player

#### Scenario: Name taken
- **WHEN** a file with the same timestamp name already exists
- **THEN** the recording is saved with a `_1` suffix

#### Scenario: Clipboard unchanged
- **WHEN** copy to clipboard is enabled and a recording ends
- **THEN** the clipboard content is unchanged

### Requirement: Video size is fixed for the whole recording
The video size SHALL be the window size when recording starts, rounded down to even width and height. If the window is resized while recording, the video size SHALL stay the same.

#### Scenario: Odd window size
- **WHEN** recording starts on a window that is 801x601 pixels
- **THEN** the video is 800x600

#### Scenario: Window resized
- **WHEN** the recorded window is resized during a recording
- **THEN** the saved video keeps the size it had when recording started

### Requirement: Preview toast after a recording
After a recording's file is finalized, and only if the preview duration is greater than 0, the preview toast SHALL show the last recorded frame. Clicking it SHALL open the saved video in Explorer. No toast SHALL appear while a capture overlay is open.

#### Scenario: Toast after recording
- **WHEN** a recording's file is finalized and the preview duration is greater than 0
- **THEN** the toast shows the last frame of the recording

#### Scenario: Toast click
- **WHEN** the user clicks the recording toast
- **THEN** Explorer opens with the saved `.mp4` file selected

#### Scenario: Preview disabled
- **WHEN** a recording's file is finalized and the preview duration is 0
- **THEN** no toast appears

### Requirement: Screenshots keep working during a recording
While a recording is running, the fullscreen, window and area hotkeys SHALL keep working as usual.

#### Scenario: Screenshot while recording
- **WHEN** a recording is running and the user presses the area hotkey
- **THEN** the area overlay opens and the recording continues

### Requirement: Recording frame rate is configurable
The recording frame rate SHALL be 30 or 60 frames per second, taken from the config. The default SHALL be 30. A missing or unsupported value SHALL be treated as 30. A recording SHALL keep the frame rate it started with until it ends.

#### Scenario: 60 fps recording
- **WHEN** the frame rate is set to 60 and the user records a window that is constantly changing
- **THEN** the saved video plays at about 60 frames per second

#### Scenario: Default
- **WHEN** the config has no frame rate, or a value other than 30 or 60
- **THEN** recordings are made at about 30 frames per second

#### Scenario: Changed during a recording
- **WHEN** the user changes the frame rate while a recording is running
- **THEN** that recording keeps its starting frame rate and the next recording uses the new one

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

### Requirement: Area recording follows the drag
An area recording SHALL capture the dragged rectangle in physical pixels, clipped to the monitor where the drag started, with the video size equal to that rectangle rounded down to even width and height. A drag shorter than a few pixels in either direction SHALL count as a click. The window screenshot picker SHALL NOT accept a drag.

#### Scenario: Odd area size
- **WHEN** the user drags a 641x481 pixel rectangle in the record picker
- **THEN** the saved video is 640x480 and shows only that rectangle

#### Scenario: Drag across monitors
- **WHEN** the drag starts on the first monitor and ends on the second
- **THEN** the recorded area is clipped to the first monitor

#### Scenario: Tiny drag
- **WHEN** the user presses and releases with a movement of one or two pixels over a window
- **THEN** that window is recorded as with a click

#### Scenario: Window moves under the area
- **WHEN** an area recording is running and the window under it is moved away
- **THEN** the recording keeps showing the same screen rectangle

#### Scenario: Window screenshot picker unchanged
- **WHEN** the user drags in the picker opened by the window screenshot hotkey
- **THEN** no area is selected and the picker behaves as before

### Requirement: Countdown before recording
When the countdown setting is on, a pick in the record picker SHALL show a 3-second countdown (3, 2, 1) centered on the picked window, monitor or area before recording starts. The countdown SHALL NOT appear in the recording. The recording SHALL be named by the timestamp at which it actually starts. When the setting is off, recording SHALL start immediately as before.

#### Scenario: Countdown on
- **WHEN** the countdown setting is on and the user picks a window in the record picker
- **THEN** 3, 2, 1 is shown over the window for about three seconds and then recording of that window starts

#### Scenario: Countdown off
- **WHEN** the countdown setting is off and the user picks a window
- **THEN** recording starts immediately and no countdown is shown

#### Scenario: Countdown not recorded
- **WHEN** a recording starts after a countdown
- **THEN** no countdown digit appears in any frame of the saved video

### Requirement: Countdown can be cancelled
During a countdown, pressing the record hotkey or choosing Stop recording in the tray SHALL cancel it. A cancelled countdown SHALL start no recording, create no file and restore the tray tooltip and pins. The Stop recording tray item SHALL be enabled during a countdown.

#### Scenario: Cancel with the hotkey
- **WHEN** a countdown is running and the user presses the record hotkey
- **THEN** the countdown disappears, no recording starts, no file is created and no picker opens

#### Scenario: Cancel from the tray
- **WHEN** a countdown is running and the user chooses Stop recording
- **THEN** the countdown disappears and no recording starts

#### Scenario: Target closes during countdown
- **WHEN** the picked window is closed during the countdown
- **THEN** no recording runs when the countdown ends and the tray tooltip shows the error

### Requirement: Countdown setting
The countdown SHALL be controlled by a setting that defaults to off. A config without the setting SHALL load with the countdown off. Changing the setting SHALL apply to the next pick without restarting snapr.

#### Scenario: Older config
- **WHEN** snapr loads a config written before this setting existed
- **THEN** the countdown is off and every other setting is unchanged

#### Scenario: Turned on
- **WHEN** the user turns the countdown on in the Recording settings and then starts a recording
- **THEN** the countdown is shown before that recording
