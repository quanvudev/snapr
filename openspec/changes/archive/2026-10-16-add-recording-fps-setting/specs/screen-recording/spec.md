# Spec Delta

## MODIFIED Requirements

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

## ADDED Requirements

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
