# live-captions Specification

## Purpose
Shows live captions of the speech heard on the microphone and/or system audio in a floating window, with a local translation of each finished sentence, started from the recording panel.

## Requirements

### Requirement: A captions session starts from the panel
When the panel is in Live captions mode, Start (the button or Enter) SHALL start a captions session without opening a picker and without recording anything. The panel SHALL become a compact bar with Stop and the elapsed time, and the captions window SHALL appear.

#### Scenario: Start live captions
- **WHEN** the user selects Live captions in the panel, the model for the spoken language is downloaded, and the user clicks Start
- **THEN** no picker opens, the compact bar shows Stop and a running timer, and the captions window appears above it

#### Scenario: Nothing is saved
- **WHEN** a captions session runs and then stops
- **THEN** no file is added to the output folder and no preview toast appears

### Requirement: Live captions from the action ring
The action ring SHALL offer a Live captions item. It SHALL be in the default ring and SHALL be available to add on the ring page for a customised ring that lacks it. Picking it SHALL open the panel in Live captions mode, or switch an open panel to that mode. While a session runs it SHALL read Stop live captions and end the session. It SHALL be left out while a recording or countdown runs, and the Record item SHALL be left out while a session runs.

#### Scenario: Open from the ring
- **WHEN** nothing is running and the user picks Live captions in the ring
- **THEN** the panel opens with the Live captions mode selected and the saved languages

#### Scenario: Stop from the ring
- **WHEN** a captions session runs and the user opens the ring
- **THEN** the item reads Stop live captions, the Record item is not shown, and picking the item ends the session

#### Scenario: During a recording
- **WHEN** a recording runs and the user opens the ring
- **THEN** the ring has no Live captions item

#### Scenario: Customised ring
- **WHEN** the user had customised the ring before this version
- **THEN** their ring is unchanged and the ring page offers Live captions among the items that can be added

#### Scenario: Ring opened from the setup panel
- **WHEN** the panel is open in area mode and the user picks Live captions in the ring
- **THEN** the panel switches to Live captions mode

### Requirement: Captions listen to the panel's audio choices
A captions session SHALL listen to the microphone and the system audio chosen in the panel, mixed together, with noise suppression applied to the microphone when it is on. Start SHALL be unavailable while neither a microphone nor system audio is selected.

#### Scenario: Caption a video playing on the computer
- **WHEN** the user selects no microphone and the default system audio, starts live captions and plays a video with speech
- **THEN** the speech of the video appears in the captions window

#### Scenario: No audio source
- **WHEN** the user selects no microphone and no system audio
- **THEN** Start is disabled and its tooltip says to choose an audio source

### Requirement: The line being spoken is shown as it is recognised
The captions window SHALL show the current line while it is being spoken and update it at least every half second, typically within one second of the words being said on a recent CPU. A line SHALL be finished after a pause of about 1.2 seconds, after about 10 seconds of continuous speech, or when the other audio source (microphone or system audio) takes over.

#### Scenario: Words appear while speaking
- **WHEN** the user speaks a sentence into the microphone during a session
- **THEN** its words appear in the captions window before the sentence ends

#### Scenario: Pause finishes a line
- **WHEN** the user finishes a sentence and stops speaking for about a second and a half
- **THEN** the sentence is finished and the next words start a new entry

### Requirement: Lines are grouped into sentences
Finished lines that do not end a sentence SHALL be joined into one sentence, shown as a single entry, until a line ends with sentence punctuation, the sentence reaches about 4 lines or 240 characters, or speech has paused for about half a second more.

#### Scenario: Short pause inside a sentence
- **WHEN** the user pauses briefly in the middle of a sentence and then continues it
- **THEN** the continuation is added to the same entry instead of starting a new one

### Requirement: Finished lines are translated
When a target language is chosen, each finished sentence (never part of one) SHALL be translated locally with the translation model and the translation shown under that line, while recognition of later speech continues. With Translate to set to Off, only the recognised text SHALL be shown.

#### Scenario: English to Vietnamese
- **WHEN** the spoken language is English, the target is Vietnamese and the user finishes a sentence
- **THEN** the Vietnamese translation appears under that sentence, typically within about a second on a recent CPU

#### Scenario: Sentence split by a pause
- **WHEN** the user pauses in the middle of a sentence and then finishes it
- **THEN** the whole sentence is translated once, not each half on its own

#### Scenario: Translation does not block captions
- **WHEN** a translation is still running and the speaker continues
- **THEN** the new words keep appearing in the current line

#### Scenario: Translation off
- **WHEN** Translate to is Off
- **THEN** the captions window shows only the recognised lines and the translation model is not loaded

### Requirement: Captions window
The captions window SHALL be frameless, always on top, never take focus when shown, and show the most recent lines with the newest at the bottom, keeping earlier lines of the session scrollable. It SHALL be draggable anywhere on any monitor and SHALL close when the session ends.

#### Scenario: Window does not steal focus
- **WHEN** a captions session starts while the user types in another app
- **THEN** the typing continues in that app

#### Scenario: Scroll back
- **WHEN** a session has produced more lines than the window shows
- **THEN** the user can scroll up to earlier lines, and new lines scroll into view again once the user is back at the bottom

### Requirement: Speakers are told apart
Each entry SHALL be attributed to the microphone (the user) or to system audio (the speaker), whichever was louder while it was spoken, and a sentence SHALL never mix both. The captions window SHALL show entries as chat bubbles: the user's on the right labelled You, system audio on the left labelled Speaker, with the label shown when the side changes and the translation inside the same bubble. When both talk at once, the entry SHALL go to the louder side.

#### Scenario: Meeting with a remote participant
- **WHEN** a remote participant speaks through the speakers and the user answers into the microphone
- **THEN** the participant's sentences appear on the left as Speaker and the user's on the right as You

#### Scenario: Turn change without a pause
- **WHEN** the user starts talking right as the remote participant stops
- **THEN** the participant's entry ends and the user's words start a new entry on the right

#### Scenario: Speakers heard by the microphone
- **WHEN** the user has no headphones and the microphone picks up the speakers
- **THEN** that speech is still attributed to Speaker

### Requirement: Captions window is not captured
The captions window and the compact bar SHALL NOT appear in screenshots taken by snapr, in snapr recordings, or in other apps' screen captures.

#### Scenario: Screen share
- **WHEN** a captions session runs while the user shares the screen in a meeting app
- **THEN** the other participants do not see the captions window

### Requirement: Session text can be copied
The captions window SHALL offer Copy all, which copies every finished entry of the session, one per line, prefixed with You or Speaker, each followed by its translation on the next line when it has one.

#### Scenario: Copy after a meeting
- **WHEN** the user clicks Copy all after several lines were translated
- **THEN** the clipboard holds each entry with its speaker, followed by its translation

### Requirement: Supported spoken languages
The Spoken language picker SHALL list English, Vietnamese, Chinese, Japanese, Spanish, German, Arabic and Filipino. Translate to SHALL list Off and every translation target, defaulting to the same default target as subtitle translation for the spoken language.

#### Scenario: Default target for English
- **WHEN** the user picks English as the spoken language for the first time
- **THEN** Translate to is Vietnamese

#### Scenario: Same language
- **WHEN** the target equals the spoken language
- **THEN** the lines are shown without translation

### Requirement: Caption models are optional and downloaded on request
Nothing for live captions SHALL be downloaded at startup. The AI models page SHALL show a Live captions card where the model of each spoken language can be downloaded, cancelled and deleted, with its size, and Downloaded models SHALL list the downloaded ones. A model in use by a running session SHALL NOT be deletable.

#### Scenario: Model missing
- **WHEN** the panel is in Live captions mode and the model for the chosen spoken language is not downloaded
- **THEN** Start is disabled and the panel shows that the model is missing with a link that opens AI models

#### Scenario: Translation model missing
- **WHEN** a target language is chosen and the translation model is not downloaded
- **THEN** the panel says that translation needs the translation model, with a link to AI models, and Start still starts captions without translation

#### Scenario: Delete while in use
- **WHEN** a session runs and the user tries to delete its caption model
- **THEN** the delete is refused and the model stays

### Requirement: A captions session ends cleanly
Stop in the compact bar, the record hotkey, and Quit SHALL end the session. Ending SHALL close the captions window and the panel, release the audio devices and unload the models. When the microphone is lost during a session, the compact bar SHALL show the lost-microphone warning while system audio, if selected, keeps being captioned.

#### Scenario: Stop
- **WHEN** the user clicks Stop in the compact bar
- **THEN** the captions window and the panel close and the microphone is released

#### Scenario: Hotkey stops captions
- **WHEN** a captions session runs and the user presses the record hotkey
- **THEN** the session ends instead of a recording starting

### Requirement: Caption languages are remembered
The chosen spoken language and target language SHALL be saved when a session starts and preselected the next time the panel is in Live captions mode. A config without them SHALL load with English as the spoken language and the default target.

#### Scenario: Next session
- **WHEN** the user runs a session with Japanese to Vietnamese and later opens the panel in Live captions mode
- **THEN** Japanese and Vietnamese are preselected

### Requirement: Captions run locally
Live captions SHALL run entirely on the computer: no audio or text SHALL be sent over the network.

#### Scenario: Offline
- **WHEN** the computer has no network connection and the models are downloaded
- **THEN** live captions and translation work as usual
