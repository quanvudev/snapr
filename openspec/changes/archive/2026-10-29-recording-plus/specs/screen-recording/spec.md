# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
