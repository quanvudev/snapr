## MODIFIED Requirements

### Requirement: Device selection before recording
Before a recording, the panel SHALL show a round webcam preview of about 200 logical pixels above a pill-shaped bar. The bar SHALL hold, as icons only with a tooltip each:
- the recording mode: full screen, window or area;
- a microphone picker (input devices, default, none) with the live input level, and a Noise suppression on / off choice that is unavailable while no microphone is selected;
- a camera picker (video devices, off) with a Background choice (none, blur or image) that is unavailable while the camera is off;
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

#### Scenario: Camera off
- **WHEN** the user turns the camera off
- **THEN** the Background choice is shown disabled

### Requirement: Devices are locked while recording
Once recording starts, the panel SHALL switch to a compact pill bar with only a Stop button, the elapsed time, Pause / Resume and Delete, without text labels other than the time. A lost-microphone warning icon SHALL appear only when the microphone was lost. The microphone, the noise suppression choice, the camera device, the background choice and the system audio choice SHALL NOT be changeable until the recording ends.

#### Scenario: Compact bar
- **WHEN** a recording starts
- **THEN** the panel shows Stop, the timer, Pause and Delete

#### Scenario: Paused timer
- **WHEN** the recording is paused
- **THEN** the timer stops advancing and is shown dimmed

### Requirement: Choices are remembered
The chosen mode, microphone, noise suppression choice, camera, background choice and speaker SHALL be saved when a recording starts and preselected when the panel opens; a missing camera SHALL fall back to the first camera found and a missing microphone or speaker to the default one. A config without them SHALL load with full screen, the default microphone, noise suppression on, no background effect and no speaker.

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

## ADDED Requirements

### Requirement: Webcam background blur
With Blur background on, the round preview and the webcam window SHALL show the person sharp and everything behind them blurred, and the recording SHALL contain the blurred webcam. The effect SHALL run on the computer with no download and no network access. Until the person is detected, the whole webcam image SHALL be shown blurred, so the background is never shown unblurred while the choice is on.

#### Scenario: Messy room
- **WHEN** the user turns Blur background on and records with the camera
- **THEN** the saved video shows the user's face sharp and the room behind them blurred

#### Scenario: No unblurred flash
- **WHEN** the camera window opens with Blur background on
- **THEN** the first image it shows is already blurred

#### Scenario: Blur off
- **WHEN** Blur background is off
- **THEN** the webcam image is shown as captured

### Requirement: Webcam background image
The user SHALL be able to pick a photo in Settings → Recording and use it in place of the room behind them. With Image selected, the preview, the webcam window and the recording SHALL show the person in front of that photo, which SHALL cover the frame and SHALL NOT appear mirrored. The Image choice SHALL be unavailable until a photo is picked. Until the person is detected or while the photo loads, the background SHALL be shown blurred.

#### Scenario: Replace the background with an image
- **WHEN** the user chooses Image… in Settings → Recording, picks a photo, then selects Image in the camera menu of the panel and records
- **THEN** the saved video shows the user in front of that photo, which is not mirrored, and the room is not visible

#### Scenario: No image chosen
- **WHEN** no background image has been chosen
- **THEN** the Image choice in the camera menu is disabled

### Requirement: Background effect look is adjustable
Settings → Recording SHALL offer, per effect: blur strength (1 to 10, default 4) for Blur; edge smoothing (0 to 90 %, default 50 %) for both; and for Image the edge fit (10 to 60, default 30), edge softness (0 to 4 px, default 1), edge blend (0 to 100 %, default 50 %) and blend spread (1 to 10, default 4). Out-of-range stored values SHALL be clamped. Changes SHALL apply the next time the panel opens.

#### Scenario: Stronger blur
- **WHEN** the user raises Strength in Settings → Recording and opens the recording panel
- **THEN** the background in the preview is more blurred than before

#### Scenario: Smoother edges
- **WHEN** the user raises Edge smoothing and moves in front of the camera
- **THEN** the outline of the person flickers less and follows movement a little later

#### Scenario: Tune the image edge
- **WHEN** the user, with Image selected, lowers Edge fit and raises Edge softness and Edge blend
- **THEN** more hair is kept, the outline gets softer and takes on more of the image's colour

### Requirement: Live preview of the background effect
Settings → Recording SHALL show a round webcam preview of the chosen effect that follows the sliders while they move. The camera SHALL turn on only after the user presses Preview and SHALL turn off when the settings window loses focus or is hidden, when the effect is turned off, or when the user leaves the page. Opening the preview SHALL NOT show a camera permission prompt.

#### Scenario: Tune with a live preview
- **WHEN** the user presses Preview in the Camera card and drags the Strength slider
- **THEN** the round preview shows the blur change while dragging

#### Scenario: Camera turns off
- **WHEN** the preview is on and the user hides the settings window
- **THEN** the camera turns off
