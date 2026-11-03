# screen-recording Specification

## Purpose
Defines how snapr records the contents of a picked window to an MP4 video file: starting and stopping a recording, how the user knows a recording is running, and how the finished file is saved and previewed.

## Requirements

### Requirement: Record hotkey starts a recording
When no recording is running, pressing the record hotkey SHALL open the recording panel, as described by the `recording-panel` capability. Start SHALL follow the chosen mode: full screen records the monitor under the cursor with no picker, window opens a picker where a click records the window, and area opens a picker where a drag records that rectangle. Cancelling a picker SHALL start nothing and return to the panel.

#### Scenario: Full screen
- **WHEN** the mode is full screen and the user clicks Start
- **THEN** recording of the monitor under the cursor starts with no picker

#### Scenario: Pick a window
- **WHEN** the mode is window, the user clicks Start and clicks a highlighted window
- **THEN** the picker closes and recording of that window starts

#### Scenario: Drag an area
- **WHEN** the mode is area, the user clicks Start and drags a rectangle over part of a window
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
A recording SHALL be saved in the configured output folder as an H.264 MP4 file at the configured frame rate (30 or 60 frames per second), with the cursor. It SHALL contain an AAC audio track when a microphone or a speaker was selected, and no audio track otherwise. It SHALL be named by the local timestamp when recording started, with a `_N` suffix when the name is taken. A recording SHALL NOT be copied to the clipboard.

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
- **WHEN** a recording ends that was started with no microphone and no speaker
- **THEN** the saved file has no audio track

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

### Requirement: Window picker falls back to the full screen
In the record window picker, a left click on the desktop SHALL close the picker and start recording the whole monitor under the cursor. The window screenshot picker SHALL NOT change.

#### Scenario: Click the desktop
- **WHEN** the mode is window and the user left-clicks the desktop in the picker
- **THEN** the picker closes and recording of the monitor under the cursor starts, with the capture border around that monitor and the tooltip `snapr: recording…`

#### Scenario: Multiple monitors
- **WHEN** the user records the full screen with the cursor on the second monitor
- **THEN** only the second monitor is recorded, at that monitor's size rounded down to even width and height

#### Scenario: Stop a full-screen recording
- **WHEN** a full-screen recording is running and the user presses the record hotkey or chooses Stop recording
- **THEN** the recording stops, the MP4 is saved with the usual name, and the toast shows its last frame

#### Scenario: Window screenshot picker unchanged
- **WHEN** the user presses the window screenshot hotkey and left-clicks the desktop
- **THEN** no full-screen capture is taken and the picker behaves as before

### Requirement: Area recording follows the drag
An area recording SHALL capture the dragged rectangle in physical pixels, clipped to the monitor where the drag started, with the video size equal to that rectangle rounded down to even width and height. A rectangle smaller than 16x16 pixels SHALL record the window under the start of the drag, or the monitor on the desktop. The window pickers SHALL NOT accept a drag.

#### Scenario: Odd area size
- **WHEN** the user drags a 641x481 pixel rectangle in the record area picker
- **THEN** the saved video is 640x480 and shows only that rectangle

#### Scenario: Drag across monitors
- **WHEN** the drag starts on the first monitor and ends on the second
- **THEN** the recorded area is clipped to the first monitor

#### Scenario: Tiny drag
- **WHEN** the mode is area and the user presses and releases with a movement of one or two pixels over a window
- **THEN** that window is recorded

#### Scenario: Window moves under the area
- **WHEN** an area recording is running and the window under it is moved away
- **THEN** the recording keeps showing the same screen rectangle

#### Scenario: Window pickers ignore drags
- **WHEN** the user drags in the record window picker or the window screenshot picker
- **THEN** no area is selected

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

### Requirement: Recordings capture audio
When a microphone is selected, the recording SHALL contain its sound. When a speaker is selected, the recording SHALL contain what that output device plays. When both are selected, they SHALL be mixed into one stereo track. Audio SHALL stay in sync with the video within about 100 ms over a 10-minute recording.

#### Scenario: Narrated tutorial
- **WHEN** the user records with a microphone and speaks
- **THEN** the saved video plays the voice in sync with the screen

#### Scenario: Microphone and system audio
- **WHEN** the user records a playing video with its speaker and a microphone selected, and comments over it
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

### Requirement: Automatic transcription after recording
When automatic transcription is on, a finished recording that has an audio track SHALL be queued for transcription with the selected model and language, ahead of any automatic compression of the same recording. It SHALL be skipped without a message when the recording has no audio track, the selected model is not downloaded, or the recording was discarded. The preview toast SHALL open as before.

#### Scenario: Recording with microphone
- **WHEN** automatic transcription is on, the Small model is downloaded and the user records with a microphone
- **THEN** after the recording ends a transcribe item appears in the queue, and when it finishes the recording's `.srt` exists

#### Scenario: No audio
- **WHEN** automatic transcription is on and the user records with no microphone and no system audio
- **THEN** nothing is queued and no error is shown

#### Scenario: Model not downloaded
- **WHEN** automatic transcription is on and the selected model is not downloaded when a recording ends
- **THEN** nothing is queued and no error is shown

#### Scenario: With automatic compression
- **WHEN** automatic transcription and automatic compression are both on and a 2560x1440 recording with audio ends
- **THEN** the transcribe item is queued before the compress item, and the `.srt` keeps the recording's name

#### Scenario: Off by default
- **WHEN** snapr loads a config without the automatic transcription setting
- **THEN** automatic transcription is off and recordings are not transcribed

### Requirement: Recording bitrate follows the quality setting
A recording's video bitrate SHALL be derived from the recorded width and height, its frame rate and the configured quality (Low, Medium or High), so a smaller area, a lower frame rate or a lower quality gives a smaller file. Higher quality SHALL never give a lower bitrate than lower quality for the same size and frame rate. The bitrate SHALL stay within fixed minimum and maximum bounds.

#### Scenario: Smaller area
- **WHEN** the user records a 640x360 area and then the full 1920x1080 screen for the same time with the same settings
- **THEN** the area recording is much smaller than the full-screen recording

#### Scenario: Quality order
- **WHEN** the same content is recorded at Low, Medium and High
- **THEN** the file sizes are in the order Low < Medium < High

#### Scenario: Smaller than before
- **WHEN** a 1920x1080, 30 fps screen is recorded for one minute at Medium
- **THEN** the file is well below the 110 MB a fixed 15 Mbps bitrate would give

### Requirement: Recording quality setting
The quality SHALL be Low, Medium or High, taken from the config, defaulting to Medium. A config without the setting, or with an unknown value, SHALL load as Medium with every other setting unchanged. A recording SHALL keep the quality it started with until it ends.

#### Scenario: Older config
- **WHEN** snapr loads a config written before this setting existed
- **THEN** recordings use Medium and every other setting is unchanged

#### Scenario: Changed during a recording
- **WHEN** the user changes the quality while a recording is running
- **THEN** that recording keeps its starting quality and the next recording uses the new one

### Requirement: Automatic compression after recording
When automatic compression is on and a finished recording is taller than the configured target (1080p or 720p), Snapr SHALL queue it for compression to that height at the recording quality. When it finishes, the original SHALL move to the Recycle Bin and the compressed file SHALL take the original's name. A recording at or below the target, or a discarded recording, SHALL NOT be compressed. A failed or cancelled compression SHALL keep the original unchanged.

#### Scenario: Large recording
- **WHEN** automatic compression is on with 1080p and the user records a 2560x1440 screen
- **THEN** after the recording ends a compress item appears in the queue, and when it finishes the file with the original name is 1920x1080 and the original is in the Recycle Bin

#### Scenario: Small recording
- **WHEN** automatic compression is on with 1080p and the user records a 1280x720 area
- **THEN** nothing is queued and the recording is kept as recorded

#### Scenario: Off by default
- **WHEN** snapr loads a config without the automatic compression settings
- **THEN** automatic compression is off with 1080p as the target, and recordings are not compressed

#### Scenario: Cancelled
- **WHEN** the user cancels the automatic compression item in the queue toast
- **THEN** the original recording stays in place unchanged and no partial file remains

#### Scenario: Preview toast
- **WHEN** a recording ends with automatic compression on
- **THEN** the preview toast opens at once as before, and opening the file later opens the compressed file under the same name
