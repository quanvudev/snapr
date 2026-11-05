## MODIFIED Requirements

### Requirement: Recording shows automatic English translation
The After recording card SHALL show an "Also translate subtitles" switch directly under the automatic transcription switch, only while automatic transcription is on. Its description SHALL say that the translation model translates into the default target language when downloaded, and that otherwise non-English recordings get English subtitles from Whisper. Changing it SHALL save immediately. While Large v3 Turbo is the selected Whisper model and the translation model is not downloaded, the row SHALL show a note that English translation works best with Base or Small; the switch stays usable.

#### Scenario: Hidden while transcription is off
- **WHEN** automatic transcription is off
- **THEN** the Also translate subtitles switch is not shown

#### Scenario: Turn it on
- **WHEN** automatic transcription is on and the user turns Also translate subtitles on
- **THEN** the config is saved with it on and the next recording with audio is translated as described in screen recording

#### Scenario: Turbo note
- **WHEN** Large v3 Turbo is the selected model, the translation model is not downloaded and automatic transcription is on
- **THEN** the translation row shows the note recommending Base or Small

#### Scenario: Turbo with the translation model
- **WHEN** Large v3 Turbo is selected and the translation model is downloaded
- **THEN** no Turbo note is shown

## ADDED Requirements

### Requirement: AI models shows the translation model
AI models SHALL show a translation card for HY-MT1.5 1.8B with a one-line description (translates transcripts between 38 languages on this computer), its size, the licence name with a link, its status line, and a "Used for" row naming Translate subtitles in the gallery and Also translate subtitles. Downloaded models SHALL list it with its size and Delete like the other models.

#### Scenario: Card
- **WHEN** the user opens AI models
- **THEN** the translation card shows the description, about 1.4 GB, "Tencent HY Community License" with a link, and Download

#### Scenario: Downloaded list
- **WHEN** the translation model is downloaded
- **THEN** Downloaded models lists HY-MT1.5 1.8B with its size, and the total includes it
