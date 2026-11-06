# recording-panel Specification

## Purpose
Defines the floating recording panel and the webcam window: choosing a microphone, camera and speaker for system audio before a recording, controlling the recording while it runs, and placing the webcam on screen so that it appears in the video while the panel does not.

## Requirements

### Requirement: Panel opens before a recording
When no recording or countdown is running, the record hotkey and the ring Record wedge SHALL open the recording panel near the bottom centre of the monitor under the cursor. The panel SHALL take focus so Enter and Esc work at once.

#### Scenario: Open with the hotkey
- **WHEN** no recording is running and the user presses the record hotkey
- **THEN** the recording panel appears on the monitor under the cursor and no picker opens yet

#### Scenario: Close the panel
- **WHEN** the panel is open before a recording and the user presses Esc, clicks its close button or presses the record hotkey again
- **THEN** the panel closes, the camera is released and nothing is recorded

### Requirement: Device selection before recording
Before a recording, the panel SHALL show a round webcam preview of about 200 logical pixels above a pill-shaped bar. The bar SHALL hold, as icons only with a tooltip each:
- the recording mode: full screen, window or area;
- a microphone picker (input devices, default, none) with the live input level, and a Noise suppression on / off choice that is unavailable while no microphone is selected;
- a camera picker (video devices, off);
- a speaker picker for system audio (output devices, default, none);
- Start and Close.

The live input level SHALL reflect the microphone after noise suppression while it is on, and SHALL follow a change of the choice within about half a second.

#### Scenario: Pick a microphone
- **WHEN** the user selects a microphone in the panel and speaks
- **THEN** the level meter moves with the voice

#### Scenario: No microphone
- **WHEN** the computer has no input device
- **THEN** the microphone tooltip says that no microphone was found

#### Scenario: Choose a mode
- **WHEN** the user selects the area mode and clicks Start
- **THEN** the area picker opens

#### Scenario: Camera available
- **WHEN** the panel opens and a camera is present
- **THEN** a camera is selected and its image shows in the round preview

#### Scenario: Noise suppression lowers the meter on background noise
- **WHEN** noise suppression is on, a fan runs near the microphone and the user is silent
- **THEN** the level meter stays near zero, and it rises when the user turns noise suppression off

#### Scenario: No microphone selected
- **WHEN** the user selects no microphone
- **THEN** the Noise suppression choice is shown disabled

### Requirement: Preview placeholder and device lists
The device pickers SHALL open their lists inside the panel, wide enough to read the device names. When the camera is off, no camera is found or the camera cannot be opened, the round preview SHALL show a placeholder saying so.

#### Scenario: No camera
- **WHEN** the computer has no video device
- **THEN** the round preview shows a "no camera found" placeholder

#### Scenario: Long device name
- **WHEN** the user opens the microphone list and a device has a long name
- **THEN** the list stays inside the panel and the name is shortened with an ellipsis

### Requirement: Start opens the picker
Start (the panel button or Enter) SHALL hide the panel, release the preview camera and open the record picker. A pick SHALL show the panel as the compact recording bar and start the recording, after the countdown when it is on. Cancelling the picker SHALL return to the panel with the same choices.

#### Scenario: Start and pick
- **WHEN** the user clicks Start and then picks an area
- **THEN** the compact bar and the webcam window appear and the recording of that area starts

#### Scenario: Cancel the picker
- **WHEN** the user clicks Start and then presses Esc in the picker
- **THEN** the panel is shown again with the same microphone, camera and speaker choices

### Requirement: Devices are locked while recording
Once recording starts, the panel SHALL switch to a compact pill bar with only a Stop button, the elapsed time, Pause / Resume and Delete, without text labels other than the time. A lost-microphone warning icon SHALL appear only when the microphone was lost. The microphone, the noise suppression choice, the camera device and the system audio choice SHALL NOT be changeable until the recording ends.

#### Scenario: Compact bar
- **WHEN** a recording starts
- **THEN** the panel shows Stop, the timer, Pause and Delete

#### Scenario: Paused timer
- **WHEN** the recording is paused
- **THEN** the timer stops advancing and is shown dimmed

### Requirement: A recording can be discarded
The Delete button SHALL ask for a second click within 3 seconds. The second click SHALL stop the recording, delete its file and show no preview toast. During the countdown, Delete SHALL cancel the countdown.

#### Scenario: Discard
- **WHEN** a recording is running and the user clicks Delete twice
- **THEN** the recording ends, no file is left in the output folder and no toast appears

#### Scenario: Single click
- **WHEN** the user clicks Delete once and waits 3 seconds
- **THEN** the recording continues and the button returns to normal

### Requirement: Panel is not recorded
The panel SHALL stay visible on screen during the recording and SHALL NOT appear in any frame of the saved video.

#### Scenario: Recording the whole monitor
- **WHEN** the user records the monitor that shows the panel
- **THEN** the saved video does not contain the panel

### Requirement: Panel and webcam can be dragged
The user SHALL be able to drag the panel before a recording and the webcam window during a recording anywhere on any monitor. Dragging the webcam during a recording SHALL move it in the video live. While the webcam is shown, the compact bar SHALL stay centred just below it, overlapping its bottom edge slightly, and follow it while it is dragged, kept inside the work area.

#### Scenario: Move the webcam while recording
- **WHEN** a monitor recording is running and the user drags the webcam to the other corner
- **THEN** the saved video shows the webcam moving to that corner at that moment and the bar follows it on screen

#### Scenario: No camera
- **WHEN** a recording runs with the camera off
- **THEN** the bar stays at the bottom centre of the monitor

### Requirement: Webcam window
When a camera is selected, a round, always-on-top webcam window of about 180 logical pixels with a soft shadow SHALL show the camera image once the recording target is picked. In the full screen and window modes it SHALL appear where the round preview was in the panel, unless that place is outside the recorded target. In the area mode, or when that place is outside the target, it SHALL sit in the bottom-left corner inside the target.

#### Scenario: Full screen keeps the place
- **WHEN** the camera is on, the mode is full screen and the user clicks Start
- **THEN** the webcam appears where the preview was and the recording bar sits below it, clickable

#### Scenario: Area moves the webcam
- **WHEN** the camera is on and the user records an area away from the panel
- **THEN** the webcam appears in the bottom-left corner inside that area

#### Scenario: Webcam in the video
- **WHEN** an area recording runs with the webcam inside the area
- **THEN** the saved video shows the round webcam

#### Scenario: Camera unavailable
- **WHEN** the selected camera cannot be opened, for example because another app uses it or access is blocked
- **THEN** the round preview shows that the camera cannot be opened with a button that opens the Windows camera privacy settings

### Requirement: Recording bar stays above the webcam
The recording bar SHALL stay above the webcam window so it can always be clicked.

#### Scenario: Webcam near the bottom
- **WHEN** the user drags the webcam to the bottom edge of the screen
- **THEN** the bar overlaps the webcam and its buttons can still be clicked

### Requirement: Choices are remembered
The chosen mode, microphone, noise suppression choice, camera and speaker SHALL be saved when a recording starts and preselected when the panel opens; a missing camera SHALL fall back to the first camera found and a missing microphone or speaker to the default one. A config without them SHALL load with full screen, the default microphone, noise suppression on and no speaker.

#### Scenario: Next recording
- **WHEN** the user records with a USB microphone and a camera, then opens the panel again
- **THEN** the same microphone and camera are preselected

#### Scenario: Device unplugged
- **WHEN** the saved camera was unplugged before the panel opens and another camera is present
- **THEN** the other camera is preselected

#### Scenario: Noise suppression turned off is remembered
- **WHEN** the user turns noise suppression off, records, then opens the panel again
- **THEN** noise suppression is still off

#### Scenario: Older config
- **WHEN** snapr starts with a config written before noise suppression existed
- **THEN** the panel opens with noise suppression on

### Requirement: Panel closes when the recording ends
When a recording ends (stopped, discarded, failed, window closed or Quit), the panel and the webcam window SHALL close and the camera SHALL be released.

#### Scenario: Stop from the panel
- **WHEN** the user clicks Stop in the panel
- **THEN** the recording is saved, the panel and webcam close and the camera light turns off
