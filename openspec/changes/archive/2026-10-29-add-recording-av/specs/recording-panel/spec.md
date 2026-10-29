# Spec Delta

## Purpose

Defines the floating recording panel and the webcam window: choosing a microphone, camera and system audio before a recording, controlling the recording while it runs, and placing the webcam on screen so that it appears in the video while the panel does not.

## ADDED Requirements

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
- a microphone picker listing the available input devices, the default microphone and none, with the live input level beside the icon while a microphone is selected;
- a camera picker listing the available video devices and off;
- a system audio toggle;
- Start and Close.


#### Scenario: Pick a microphone
- **WHEN** the user selects a microphone in the panel and speaks
- **THEN** the level meter moves with the voice

#### Scenario: No microphone
- **WHEN** the computer has no input device
- **THEN** the microphone tooltip says that no microphone was found

#### Scenario: Camera available
- **WHEN** the panel opens and a camera is present
- **THEN** a camera is selected and its image shows in the round preview

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
- **THEN** the panel is shown again with the same microphone, camera and system audio choices

### Requirement: Devices are locked while recording
Once recording starts, the panel SHALL switch to a compact pill bar with only a Stop button, the elapsed time, Pause / Resume and Delete, without text labels other than the time. A lost-microphone warning icon SHALL appear only when the microphone was lost. The microphone, the camera device and the system audio choice SHALL NOT be changeable until the recording ends.

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
When a camera is selected, a round, always-on-top webcam window of about 180 logical pixels with a soft shadow SHALL show the camera image once the recording target is picked. It SHALL sit in the bottom-left corner inside the target. The webcam window SHALL appear in the recording whenever it lies inside the recorded monitor or area.

#### Scenario: Webcam in the video
- **WHEN** an area recording runs with the webcam inside the area
- **THEN** the saved video shows the round webcam

#### Scenario: Camera unavailable
- **WHEN** the selected camera cannot be opened, for example because another app uses it or access is blocked
- **THEN** the round preview shows that the camera cannot be opened with a button that opens the Windows camera privacy settings

### Requirement: Choices are remembered
The chosen microphone, camera and system audio setting SHALL be saved in the config when a recording starts. The next time the panel opens, the saved microphone and system audio setting SHALL be preselected, and the saved camera when it is present, else the first camera found. A config without these settings SHALL load with the Windows default microphone and system audio off. A saved microphone that is no longer present SHALL fall back to the default microphone.

#### Scenario: Next recording
- **WHEN** the user records with a USB microphone and a camera, then opens the panel again
- **THEN** the same microphone and camera are preselected

#### Scenario: Device unplugged
- **WHEN** the saved camera was unplugged before the panel opens and another camera is present
- **THEN** the other camera is preselected

### Requirement: Panel closes when the recording ends
When a recording ends (stopped, discarded, failed, window closed or Quit), the panel and the webcam window SHALL close and the camera SHALL be released.

#### Scenario: Stop from the panel
- **WHEN** the user clicks Stop in the panel
- **THEN** the recording is saved, the panel and webcam close and the camera light turns off
