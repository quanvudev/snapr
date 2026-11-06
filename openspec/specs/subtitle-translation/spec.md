# subtitle-translation Specification

## Purpose
Lets Snapr translate a video's transcript into another language on this computer with the optional HY-MT1.5 model, after the user has seen and accepted its licence.

## Requirements

### Requirement: Translation model is optional and downloaded on request
AI models SHALL list a translation model "HY-MT1.5 1.8B" with its download size (about 1.4 GB), state Downloaded / Not downloaded / Downloading N% and Download, Cancel, Retry and Delete actions like the other models. Snapr SHALL NOT download it at startup or without a user action. Downloaded files SHALL be verified against pinned checksums; a failed or cancelled download SHALL leave nothing behind.

#### Scenario: Not downloaded
- **WHEN** the user opens AI models on a fresh install
- **THEN** the translation model shows Not downloaded with its size and a Download button, and nothing is being downloaded

#### Scenario: Cancel
- **WHEN** the user cancels the translation model download at 40%
- **THEN** the model shows Not downloaded and no partial file remains

#### Scenario: Delete
- **WHEN** the translation model is downloaded and the user deletes it from Downloaded models and confirms
- **THEN** its files are removed and translation falls back to Whisper English

### Requirement: Licence is confirmed before the first download
Before the first translation model download, Snapr SHALL show a licence dialog. The download SHALL start only from "I agree and download"; Cancel SHALL start nothing. The acceptance SHALL be saved so later downloads do not ask again.

#### Scenario: Dialog content
- **WHEN** the licence dialog is shown
- **THEN** it states the download size, that the model is under the Tencent HY Community License, that the licence does not apply in the European Union, the United Kingdom or South Korea, that its output must not be used to improve other AI models, that Tencent is not affiliated with Snapr, and links to the full licence

#### Scenario: First download
- **WHEN** the user clicks Download on the translation model for the first time
- **THEN** the licence dialog opens and nothing downloads until the user clicks "I agree and download"

#### Scenario: Declined
- **WHEN** the user clicks Cancel in the licence dialog
- **THEN** no download starts and the model stays Not downloaded

#### Scenario: Already accepted
- **WHEN** the user accepted before, deleted the model and clicks Download again
- **THEN** the download starts without the dialog

### Requirement: Licence notice in About
While the translation model is downloaded, About SHALL show "Translation powered by Tencent HY", the notice "Tencent HY is licensed under the Tencent HY Community License Agreement, Copyright © 2025 Tencent. All Rights Reserved. The trademark rights of “Tencent HY” are owned by Tencent or its affiliate." and a link to the licence.

#### Scenario: Notice shown
- **WHEN** the translation model is downloaded and the user opens About
- **THEN** the Tencent HY line, the notice and the licence link are shown

#### Scenario: No model
- **WHEN** the translation model is not downloaded
- **THEN** About shows no Tencent HY notice

### Requirement: Supported target languages
Translation SHALL support exactly 38 target languages, listed in the scenario below.

#### Scenario: Target list
- **WHEN** the supported targets are listed
- **THEN** they are Chinese (`zh`), Traditional Chinese (`zh-Hant`), Cantonese (`yue`), English (`en`), French (`fr`), Portuguese (`pt`), Spanish (`es`), Japanese (`ja`), Turkish (`tr`), Russian (`ru`), Arabic (`ar`), Korean (`ko`), Thai (`th`), Italian (`it`), German (`de`), Vietnamese (`vi`), Malay (`ms`), Indonesian (`id`), Filipino (`tl`), Hindi (`hi`), Polish (`pl`), Czech (`cs`), Dutch (`nl`), Khmer (`km`), Burmese (`my`), Persian (`fa`), Gujarati (`gu`), Urdu (`ur`), Telugu (`te`), Marathi (`mr`), Hebrew (`he`), Bengali (`bn`), Tamil (`ta`), Ukrainian (`uk`), Tibetan (`bo`), Kazakh (`kk`), Mongolian (`mn`) and Uyghur (`ug`)

#### Scenario: Picker lists every target
- **WHEN** the user opens the target picker for a Vietnamese video
- **THEN** it lists the 37 targets other than Vietnamese, each by its name in the interface language

### Requirement: Default target language
The default target SHALL be English for a Vietnamese video, Vietnamese for an English video, and the interface language (Vietnamese or English) for any other source language. A target equal to the source language SHALL NOT be offered.

#### Scenario: Vietnamese source
- **WHEN** a Vietnamese video is translated with the default target
- **THEN** the result is `<video name>.en.srt`

#### Scenario: English source
- **WHEN** an English video is translated with the default target
- **THEN** the result is `<video name>.vi.srt`

#### Scenario: Other source
- **WHEN** a Japanese video is translated with the default target while the interface is in Vietnamese
- **THEN** the result is `<video name>.vi.srt`

### Requirement: Subtitle translation keeps the cues
A translation SHALL read the video's original `<video name>.srt` when it is queued and write `<video name>.<target code>.srt` with the same cues and timings, each cue's text translated and empty cues kept empty, without changing the original. It SHALL run in the background queue with progress by cue, be cancellable and replace an older file of that target only on success. It SHALL be refused, queueing nothing, without the model, without a transcript or for an unsupported target.

#### Scenario: Translate English to Vietnamese
- **WHEN** an English video with a 40-cue `a.srt` is translated to Vietnamese
- **THEN** `a.vi.srt` has 40 cues with the same timings and Vietnamese text, and `a.srt` is unchanged

#### Scenario: Edited transcript
- **WHEN** the user corrected a word in `a.srt` before translating
- **THEN** the translation is made from the corrected text

#### Scenario: No transcript
- **WHEN** the user asks to translate a video that has no `a.srt`
- **THEN** nothing is queued and an error says to create a transcript first

#### Scenario: Cancel
- **WHEN** a translation to Japanese is cancelled while `a.ja.srt` already exists from before
- **THEN** `a.ja.srt` is unchanged and no partial file is left

#### Scenario: Progress
- **WHEN** a translation of a 40-cue transcript has finished 10 cues
- **THEN** the queue shows it at 25%

### Requirement: Each cue is translated on its own
Each cue SHALL be translated by itself, without the text of other cues in the request, so a translated cue never contains content from another cue. The output SHALL be only the translation of that cue.

#### Scenario: Short reply
- **WHEN** a cue reads "Sure." after a cue asking for a report
- **THEN** its translation is a short reply such as "Chắc chắn rồi." and does not repeat the request

### Requirement: Cues are translated in batches
Translation SHALL process up to 8 cues per model run on every runtime, each cue with its own request. A cue's translation SHALL NOT contain text from another cue and SHALL match translating that cue alone, apart from rare wording differences caused by numeric precision. Progress SHALL count finished cues, and a cancel SHALL stop the translation within one generated token of the running batch.

#### Scenario: Same result as one by one
- **WHEN** a 20-cue transcript is translated to English in batches and the same cues are translated one at a time
- **THEN** at least 19 of the 20 cues have the same translation in both runs and no cue contains text from another cue

#### Scenario: Cues of different lengths
- **WHEN** a batch holds a one-word cue and a 40-word cue
- **THEN** the one-word cue's translation is short and complete, and the 40-word cue's translation is complete

#### Scenario: Progress per batch
- **WHEN** a 40-cue transcript is translated and the first batch of 8 cues finishes
- **THEN** the queue shows it at 20%

#### Scenario: Cancel during a batch
- **WHEN** the user cancels while a batch is generating
- **THEN** the translation stops promptly, no partial `.srt` is left and an existing translation file is unchanged

#### Scenario: Faster than one by one
- **WHEN** a 100-cue transcript is translated on the CPU
- **THEN** it finishes faster than translating the cues one at a time on the same computer
