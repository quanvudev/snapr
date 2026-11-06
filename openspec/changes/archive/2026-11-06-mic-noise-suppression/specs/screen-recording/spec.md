## MODIFIED Requirements

### Requirement: Recordings capture audio
When a microphone is selected, the recording SHALL contain its sound. When a speaker is selected, the recording SHALL contain what that output device plays. When both are selected, they SHALL be mixed into one stereo track. Audio SHALL stay in sync with the video within about 100 ms over a 10-minute recording. When noise suppression is on, steady background noise in the microphone (fans, hum, keyboard) SHALL be reduced before mixing, the voice SHALL remain intelligible, and system audio SHALL NOT be processed. Noise suppression SHALL run on the computer with no download and SHALL NOT add more than about 20 ms of delay to the microphone.

#### Scenario: Narrated tutorial
- **WHEN** the user records with a microphone and speaks
- **THEN** the saved video plays the voice in sync with the screen

#### Scenario: Microphone and system audio
- **WHEN** the user records a playing video with its speaker and a microphone selected, and comments over it
- **THEN** the saved file contains both the video's sound and the voice

#### Scenario: Noisy room
- **WHEN** the user records with noise suppression on and a fan running near the microphone
- **THEN** the saved file has the voice with the fan noise clearly reduced

#### Scenario: System audio untouched
- **WHEN** the user records music playing through the speaker with noise suppression on and a microphone selected
- **THEN** the music in the saved file is not altered by the noise suppression

#### Scenario: Noise suppression off
- **WHEN** the user records with noise suppression off
- **THEN** the microphone sound is saved as captured
