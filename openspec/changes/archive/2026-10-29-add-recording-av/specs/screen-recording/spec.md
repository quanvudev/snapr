# Spec Delta

## MODIFIED Requirements

### Requirement: Record hotkey starts a window recording
When no recording is running, pressing the record hotkey SHALL open the recording panel, as described by the `recording-panel` capability; Start in the panel SHALL open the window picker. Clicking a highlighted window SHALL close the picker and start recording that window. Dragging a rectangle with the left mouse button SHALL close the picker on release and start recording that area. Cancelling the picker SHALL start nothing and return to the panel.

#### Scenario: Pick a window
- **WHEN** no recording is running and the user presses the record hotkey, clicks Start and clicks a highlighted window
- **THEN** the picker closes and recording of that window starts

#### Scenario: Drag an area
- **WHEN** no recording is running and the user presses the record hotkey, clicks Start and drags a rectangle over part of a window
- **THEN** the picker closes on release and recording of that rectangle starts, not of the window under it

#### Scenario: Cancel the picker
- **WHEN** the user presses Escape or right-clicks in the picker opened from the panel
- **THEN** the picker closes, no recording starts and the panel is shown again

#### Scenario: Recording cannot start
- **WHEN** the picked window cannot be captured or the output file cannot be created
- **THEN** no recording runs and the tray tooltip shows `snapr: <error>`

### Requirement: Only one recording runs at a time
While a recording is running, the record hotkey SHALL stop it and SHALL NOT open the panel or the picker. While the panel is open before a recording, the record hotkey SHALL close it.

#### Scenario: Record hotkey while recording
- **WHEN** a recording is running and the user presses the record hotkey
- **THEN** the recording stops and no panel or picker opens

#### Scenario: Record hotkey while the panel is open
- **WHEN** the panel is open before a recording and the user presses the record hotkey
- **THEN** the panel closes and nothing is recorded

### Requirement: Recording can be stopped from the tray
The tray menu SHALL have a Stop recording item that is enabled only while a recording is running and stops it, and a Pause recording item, enabled only while a recording is running, that reads Resume recording while the recording is paused.

#### Scenario: Stop from the tray
- **WHEN** a recording is running and the user chooses Stop recording in the tray menu
- **THEN** the recording stops

#### Scenario: Nothing to stop
- **WHEN** no recording is running
- **THEN** the Stop recording and Pause recording items are disabled

#### Scenario: Pause from the tray
- **WHEN** a recording is running and the user chooses Pause recording
- **THEN** the recording pauses and the item reads Resume recording

### Requirement: A running recording is visible
While a recording is running, Windows SHALL draw its capture border around the recorded window or monitor, the recording panel SHALL show the elapsed time and the tray tooltip SHALL read `snapr: recording…`, or `snapr: paused` while paused. While an area recording is running, snapr SHALL also draw a frame just outside the recorded area that does not appear in the recording. When the recording ends the frame SHALL disappear and the tooltip SHALL return to the normal status text.

#### Scenario: During recording
- **WHEN** a recording is running
- **THEN** the recorded window has the capture border and the tray tooltip reads `snapr: recording…`

#### Scenario: While paused
- **WHEN** a recording is paused
- **THEN** the tray tooltip reads `snapr: paused` and the panel timer does not advance

#### Scenario: During an area recording
- **WHEN** an area recording is running
- **THEN** a frame is drawn around the recorded area and the saved video does not contain that frame

#### Scenario: After recording
- **WHEN** the recording ends successfully
- **THEN** the border and any area frame disappear and the tray tooltip returns to the normal status text

### Requirement: Recordings are saved as MP4 with unique names
A recording SHALL be saved in the configured output folder as an H.264 MP4 file at the configured frame rate (30 or 60 frames per second), with the cursor. It SHALL contain an AAC audio track when a microphone or system audio was selected, and no audio track otherwise. It SHALL be named by the local timestamp when recording started, with a `_N` suffix when the name is taken. A recording SHALL NOT be copied to the clipboard.

#### Scenario: File saved
- **WHEN** a recording ends
- **THEN** a `.mp4` file named by the start timestamp exists in the output folder and plays in a standard video player

#### Scenario: Smooth motion
- **WHEN** a 60 fps recording captures continuous motion, such as a scrolling page
- **THEN** almost every frame of the saved video differs from the previous one, with no repeated runs of frames

#### Scenario: Upright video
- **WHEN** an area recording ends
- **THEN** the saved video is upright, not mirrored or upside down

#### Scenario: Silent recording
- **WHEN** a recording ends that was started with no microphone and system audio off
- **THEN** the saved file has no audio track

#### Scenario: Name taken
- **WHEN** a file with the same timestamp name already exists
- **THEN** the recording is saved with a `_1` suffix

#### Scenario: Clipboard unchanged
- **WHEN** copy to clipboard is enabled and a recording ends
- **THEN** the clipboard content is unchanged

## ADDED Requirements

### Requirement: Recordings capture audio
When a microphone is selected, the recording SHALL contain its sound. When system audio is on, the recording SHALL contain what the default output device plays. When both are selected, they SHALL be mixed into one stereo track. Audio SHALL stay in sync with the video within about 100 ms over a 10-minute recording.

#### Scenario: Narrated tutorial
- **WHEN** the user records with a microphone and speaks
- **THEN** the saved video plays the voice in sync with the screen

#### Scenario: Microphone and system audio
- **WHEN** the user records a playing video with system audio on and a microphone, and comments over it
- **THEN** the saved file contains both the video's sound and the voice

### Requirement: Recordings can be paused
Pausing SHALL stop adding video frames and audio to the file, and resuming SHALL continue it. The paused time SHALL be cut from the saved file so it plays without a frozen gap. Stopping while paused SHALL save the file as recorded up to the pause.

#### Scenario: Pause and resume
- **WHEN** the user records for 10 seconds, pauses for 20 seconds, resumes and records for 10 seconds
- **THEN** the saved video is about 20 seconds long and continuous

#### Scenario: Stop while paused
- **WHEN** a paused recording is stopped
- **THEN** a playable MP4 with the content before the pause is saved

### Requirement: Lost microphone does not end the recording
If the selected microphone disappears during a recording, the recording SHALL continue, its audio SHALL be silent from that moment (system audio continues when on), and the panel SHALL show that the microphone was lost.

#### Scenario: Headset unplugged
- **WHEN** the user unplugs the selected USB microphone during a recording
- **THEN** the recording continues, the panel shows a lost-microphone warning and the saved file plays with silence from that point

### Requirement: Window pick with the camera on records an area
When the camera is on, picking a window in the record picker SHALL record that window's current screen rectangle as an area recording, so the webcam window can appear in the video. When the camera is off, picking a window SHALL record the window as before.

#### Scenario: Window with webcam
- **WHEN** the camera is on and the user picks a window
- **THEN** the window's rectangle is recorded as an area and the webcam appears in the video when it is over that rectangle

#### Scenario: Window without webcam
- **WHEN** the camera is off and the user picks a window
- **THEN** that window is recorded on its own, as before
