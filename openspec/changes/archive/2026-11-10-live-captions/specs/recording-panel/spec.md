# Spec Delta

## MODIFIED Requirements

### Requirement: Device selection before recording
Before a recording, the panel SHALL show a round webcam preview of about 200 logical pixels above a pill-shaped bar. The bar SHALL hold, as icons only with a tooltip each:
- the recording mode: full screen, window, area or live captions;
- a microphone picker (input devices, default, none) with the live input level, and a Noise suppression on / off choice that is unavailable while no microphone is selected;
- a camera picker (video devices, off) with a Background choice (none, blur or image) that is unavailable while the camera is off;
- a speaker picker for system audio (output devices, default, none);
- Start and Close.

In the live captions mode the camera picker SHALL be unavailable, the camera SHALL be released, the round preview SHALL show the spoken and target languages instead of the camera image, and the bar SHALL also hold a Spoken language picker and a Translate to picker.

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

#### Scenario: Camera off
- **WHEN** the user turns the camera off
- **THEN** the Background choice is shown disabled

#### Scenario: Live captions mode
- **WHEN** the user selects the live captions mode
- **THEN** the camera light turns off, the camera picker is shown disabled, the preview shows the caption languages and the language pickers appear

### Requirement: Start opens the picker
Start (the panel button or Enter) SHALL hide the panel, release the preview camera and open the record picker. A pick SHALL show the panel as the compact recording bar and start the recording, after the countdown when it is on. Cancelling the picker SHALL return to the panel with the same choices. In the live captions mode Start SHALL instead start a captions session without a picker or a countdown.

#### Scenario: Start and pick
- **WHEN** the user clicks Start and then picks an area
- **THEN** the compact bar and the webcam window appear and the recording of that area starts

#### Scenario: Cancel the picker
- **WHEN** the user clicks Start and then presses Esc in the picker
- **THEN** the panel is shown again with the same microphone, camera and speaker choices

#### Scenario: Start live captions
- **WHEN** the mode is live captions and the user clicks Start
- **THEN** no picker opens and the captions session starts

### Requirement: Choices are remembered
The chosen mode, microphone, noise suppression choice, camera, background choice and speaker SHALL be saved when a recording or a captions session starts and preselected when the panel opens; a missing camera SHALL fall back to the first camera found and a missing microphone or speaker to the default one. A config without them SHALL load with full screen, the default microphone, noise suppression on, no background effect and no speaker.

#### Scenario: Next recording
- **WHEN** the user records with a USB microphone and a camera, then opens the panel again
- **THEN** the same microphone and camera are preselected

#### Scenario: Device unplugged
- **WHEN** the saved camera was unplugged before the panel opens and another camera is present
- **THEN** the other camera is preselected

#### Scenario: Noise suppression turned off is remembered
- **WHEN** the user turns noise suppression off, records, then opens the panel again
- **THEN** noise suppression is still off

#### Scenario: Background choice is remembered
- **WHEN** the user selects Blur background, records, then opens the panel again
- **THEN** Blur background is still selected

#### Scenario: Older config
- **WHEN** snapr starts with a config written before noise suppression or background effects existed
- **THEN** the panel opens with noise suppression on and no background effect

#### Scenario: Live captions mode is remembered
- **WHEN** the user runs a captions session and opens the panel again
- **THEN** the live captions mode is selected
