## MODIFIED Requirements

### Requirement: Recordings are saved as MP4 with unique names
A recording SHALL be saved in the `Videos` subfolder of the configured output folder as an H.264 MP4 file at the configured frame rate (30 or 60 frames per second), with the cursor. It SHALL contain an AAC audio track when a microphone or a speaker was selected, and no audio track otherwise. It SHALL be named by the local timestamp when recording started, with a `_N` suffix when the name is taken in that subfolder. A recording SHALL NOT be copied to the clipboard.

#### Scenario: File saved
- **WHEN** a recording ends
- **THEN** a `.mp4` file named by the start timestamp exists in `<output>\Videos` and plays in a standard video player

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
- **WHEN** a file with the same timestamp name already exists in `Videos`
- **THEN** the recording is saved with a `_1` suffix

#### Scenario: Clipboard unchanged
- **WHEN** copy to clipboard is enabled and a recording ends
- **THEN** the clipboard content is unchanged
