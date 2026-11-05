## MODIFIED Requirements

### Requirement: Automatic transcription after recording
When automatic transcription is on, a finished recording that has an audio track SHALL be queued for transcription with the selected model and language, ahead of any automatic compression of the same recording. When automatic translation is also on and that transcription finishes, a translation SHALL be queued: with the translation model downloaded, into the default target language; without it, an English Whisper translation, only when the language is not English. Both SHALL be skipped without a message when the recording has no audio track, the selected model is not downloaded, or the recording was discarded. The preview toast SHALL open as before.

#### Scenario: Recording with microphone
- **WHEN** automatic transcription is on, the Small model is downloaded and the user records with a microphone
- **THEN** after the recording ends a transcribe item appears in the queue, and when it finishes the recording's `.srt` exists

#### Scenario: Vietnamese recording with translation
- **WHEN** automatic transcription and automatic translation are on, the Small model is downloaded and the user records speaking Vietnamese
- **THEN** when the transcription finishes a translate item is queued, and when it finishes both the recording's `.srt` and `.en.srt` exist

#### Scenario: English recording with the translation model
- **WHEN** automatic transcription and automatic translation are on, the translation model is downloaded and the user records speaking English
- **THEN** when the transcription finishes a translation to Vietnamese is queued, and later the recording's `.vi.srt` exists

#### Scenario: English recording is not translated
- **WHEN** automatic transcription and automatic translation are on, the translation model is not downloaded and the user records speaking English
- **THEN** only the `.srt` is created and no translate item is queued

#### Scenario: Failed transcription
- **WHEN** automatic translation is on and the recording's transcription fails or is cancelled
- **THEN** no translate item is queued

#### Scenario: No audio
- **WHEN** automatic transcription is on and the user records with no microphone and no system audio
- **THEN** nothing is queued and no error is shown

#### Scenario: Model not downloaded
- **WHEN** automatic transcription is on and the selected model is not downloaded when a recording ends
- **THEN** nothing is queued and no error is shown

#### Scenario: With automatic compression
- **WHEN** automatic transcription, automatic translation and automatic compression are all on and a 2560x1440 Vietnamese recording with audio ends
- **THEN** the transcribe item is queued before the compress item, the translate item is queued when the transcription finishes, and every subtitle file keeps the recording's name

#### Scenario: Translation needs transcription on
- **WHEN** automatic translation is on but automatic transcription is off and a recording with audio ends
- **THEN** nothing is queued

#### Scenario: Off by default
- **WHEN** snapr loads a config without the automatic transcription or automatic translation settings
- **THEN** both are off and recordings are neither transcribed nor translated
