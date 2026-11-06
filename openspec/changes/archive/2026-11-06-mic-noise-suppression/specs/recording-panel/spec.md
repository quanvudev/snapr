## MODIFIED Requirements

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

### Requirement: Devices are locked while recording
Once recording starts, the panel SHALL switch to a compact pill bar with only a Stop button, the elapsed time, Pause / Resume and Delete, without text labels other than the time. A lost-microphone warning icon SHALL appear only when the microphone was lost. The microphone, the noise suppression choice, the camera device and the system audio choice SHALL NOT be changeable until the recording ends.

#### Scenario: Compact bar
- **WHEN** a recording starts
- **THEN** the panel shows Stop, the timer, Pause and Delete

#### Scenario: Paused timer
- **WHEN** the recording is paused
- **THEN** the timer stops advancing and is shown dimmed

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
