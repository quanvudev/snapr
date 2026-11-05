## ADDED Requirements

### Requirement: Recording shows automatic English translation
The After recording card SHALL show an "Also translate to English" switch directly under the automatic transcription switch, only while automatic transcription is on. Changing it SHALL save immediately. While Large v3 Turbo is the selected Whisper model, the row SHALL show a note that English translation works best with Base or Small; the switch stays usable.

#### Scenario: Hidden while transcription is off
- **WHEN** automatic transcription is off
- **THEN** the Also translate to English switch is not shown

#### Scenario: Turn it on
- **WHEN** automatic transcription is on and the user turns Also translate to English on
- **THEN** the config is saved with it on and the next non-English recording with audio is also translated

#### Scenario: Turbo note
- **WHEN** Large v3 Turbo is the selected model and automatic transcription is on
- **THEN** the translation row shows the note recommending Base or Small
