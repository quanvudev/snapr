## MODIFIED Requirements

### Requirement: Gallery lists every file in the output folder
The gallery SHALL list every file directly inside the resolved output folder and classify it by extension, case-insensitive: photos are `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`; videos are `mp4` and `webm`; every other file is an other file. Subfolders SHALL be ignored. Files are not required to follow snapr's naming. An `srt` file whose name without extension equals that of a video in the folder is that video's transcript and SHALL NOT be listed. An `srt` file named `<video name>.<code>.srt`, case-insensitive, where `<code>` is a supported translation target, for a video in the folder is that video's translation and SHALL NOT be listed either.

#### Scenario: Mixed folder
- **WHEN** the output folder holds `a.png`, `b.MP4`, `c.jpg`, `notes.txt`, `clip.mov` and a subfolder `old` containing `d.png`
- **THEN** the gallery shows `a.png` and `c.jpg` as photos, `b.MP4` as a video, `notes.txt` and `clip.mov` as other files, and nothing from `old`

#### Scenario: Empty or missing folder
- **WHEN** the output folder has no file, or does not exist
- **THEN** the gallery shows an empty state message instead of a grid

#### Scenario: Transcript beside its video
- **WHEN** the output folder holds `a.mp4`, `a.srt` and `b.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.srt` as an other file, and does not show `a.srt`

#### Scenario: Translation beside its video
- **WHEN** the output folder holds `a.mp4`, `a.en.srt` and `b.en.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.en.srt` as an other file, and does not show `a.en.srt`

#### Scenario: Several translations
- **WHEN** the output folder holds `a.mp4`, `a.srt`, `a.vi.srt`, `a.zh-Hant.srt` and `a.xx.srt`
- **THEN** the gallery shows `a.mp4` and `a.xx.srt` (not a supported code), and hides the others

### Requirement: Deleting a video removes its transcript
Deleting an MP4, from the grid, the viewer or a bulk delete, SHALL also move its `<video name>.srt` and every `<video name>.<code>.srt` translation to the Recycle Bin when they exist. A failure to remove any of them SHALL NOT fail the video's delete.

#### Scenario: Delete a transcribed video
- **WHEN** the user deletes a video that has a transcript and confirms
- **THEN** both the video and its `.srt` are in the Recycle Bin

#### Scenario: Delete a translated video
- **WHEN** the user deletes a video that has `.srt`, `.en.srt` and `.ja.srt` and confirms
- **THEN** the video and all three subtitle files are in the Recycle Bin

#### Scenario: Delete a video without transcript
- **WHEN** the user deletes a video that has no `.srt` and confirms
- **THEN** the video is in the Recycle Bin and no error is shown

### Requirement: Translate to English from the gallery
While the translation model is not downloaded, the context menu of an MP4 whose known language is not English SHALL offer Translate to English, which queues one English translation of its speech with Whisper. A finished translation SHALL be written as `<video name>.en.srt` beside the video, replacing an older one, and SHALL NOT change `<video name>.srt`. It SHALL be refused, queueing nothing, when the selected Whisper model is not downloaded or the video is known to be English. Photos, English videos and videos of unknown language SHALL NOT offer it.

#### Scenario: Translate a Vietnamese recording
- **WHEN** the translation model is not downloaded and the user right-clicks a Vietnamese MP4 that has `a.srt` and chooses Translate to English
- **THEN** a translate item appears in the queue, and when it finishes `a.en.srt` holds English text with timings while `a.srt` is unchanged

#### Scenario: Replaced by the translation model
- **WHEN** the translation model is downloaded
- **THEN** the menu offers Translate subtitles… instead of Translate to English

#### Scenario: English video
- **WHEN** the user right-clicks an MP4 whose language is English
- **THEN** the menu has no Translate to English

#### Scenario: Unknown language
- **WHEN** the user right-clicks an MP4 whose language is not known yet
- **THEN** the menu has no Translate to English

#### Scenario: Model missing
- **WHEN** the selected Whisper model is not downloaded and the user chooses Translate to English on a Vietnamese video
- **THEN** nothing is queued and an error toast says the model is not downloaded

#### Scenario: Photo
- **WHEN** the user right-clicks a PNG
- **THEN** the menu has no Translate to English

#### Scenario: Cancel keeps the old translation
- **WHEN** a video already has `a.en.srt` and the user cancels a new translation of it
- **THEN** `a.en.srt` is unchanged and no partial file is left

### Requirement: Transcript pane tracks
The transcript pane SHALL offer a track choice listing Original (`.srt`) and every translation the video has, each named by its language. Each track SHALL have its own loading, running, error and segment states. Editing, Copy all and seeking SHALL act on the shown track only. The chosen language SHALL persist across videos that have it; otherwise Original is shown. Without the translation model the English entry follows the Whisper fallback rules.

#### Scenario: English video has no switch
- **WHEN** the translation model is not downloaded and the user opens the transcript pane of an English video with only `a.srt`
- **THEN** no track choice is shown and the original transcript is listed

#### Scenario: Switch appears after detection
- **WHEN** the translation model is not downloaded, the pane of a video with unknown language is open and detection finds Vietnamese
- **THEN** the track choice appears with Original and English

#### Scenario: Switch to English
- **WHEN** a video has `a.srt` and `a.en.srt` and the user picks English in the track choice
- **THEN** the English segments are listed, clicking one seeks the video and the active segment follows playback

#### Scenario: Edit the English track
- **WHEN** the user edits an English segment and presses Enter
- **THEN** `a.en.srt` is rewritten with the new text and `a.srt` is unchanged

#### Scenario: Several translations
- **WHEN** a video has `a.srt`, `a.en.srt` and `a.ja.srt`
- **THEN** the track choice lists Original, English and Japanese, and picking Japanese lists the Japanese segments with seeking and the active segment

#### Scenario: Translate from the pane
- **WHEN** the translation model is downloaded and the pane shows a video with a transcript and a known language
- **THEN** a Translate action opens the target picker, and a queued translation shows its progress under that language in the track choice

#### Scenario: No translation yet
- **WHEN** the translation model is not downloaded, a Vietnamese video has only `a.srt` and the user picks English
- **THEN** the pane offers Translate to English, and choosing it queues a Whisper translation and shows its progress

#### Scenario: Stays on the finished translation
- **WHEN** the pane shows the running Japanese translation and it reaches 100%
- **THEN** the pane keeps showing Japanese and lists the translated segments, without switching back to Original

#### Scenario: Follows a new translation
- **WHEN** the user picks Japanese in the target picker for the video shown in the viewer
- **THEN** the transcript pane switches to the Japanese track and shows its progress

#### Scenario: Original names its language
- **WHEN** the video's language is Vietnamese
- **THEN** the Original entry reads "Original (Vietnamese)" in the track choices of the pane and of Export with subtitles

#### Scenario: Tracks run independently
- **WHEN** a translation of a video is running and the user switches to Original
- **THEN** the original transcript is listed and editable, and switching back shows the running progress

#### Scenario: Edit a translation
- **WHEN** the user edits a Japanese segment and presses Enter
- **THEN** `a.ja.srt` is rewritten with the new text and `a.srt` is unchanged

#### Scenario: Edit refused while translating
- **WHEN** a translation of a video into a language is waiting or running
- **THEN** editing is hidden on that track, and a save of that track is refused

### Requirement: Export with subtitles chooses a track
The Export with subtitles dialog SHALL offer a Track choice among Original and every translation the video has when there is more than one, preselecting the track shown in the transcript pane when opened from it and Original otherwise. With one track the choice SHALL be hidden and that track used. The preview cue SHALL come from the chosen track. A translation export SHALL be written as `<video name>_subtitled_<code>.mp4` (`_N` when taken), an original one as `<video name>_subtitled.mp4`.

#### Scenario: Export English subtitles
- **WHEN** a video has both tracks and the user picks English and exports
- **THEN** a subtitle item is queued and the result `a_subtitled_en.mp4` shows the English cues

#### Scenario: Export Japanese subtitles
- **WHEN** a video has `a.srt` and `a.ja.srt` and the user picks Japanese and exports
- **THEN** the result is `a_subtitled_ja.mp4` with the Japanese cues

#### Scenario: Only original
- **WHEN** a video has `a.srt` and no translation and the user opens Export with subtitles
- **THEN** no Track choice is shown and the export uses `a.srt`

#### Scenario: Only English
- **WHEN** a video has `a.en.srt` and no `a.srt`
- **THEN** the context menu offers Export with subtitles, the dialog shows no Track choice and the export uses `a.en.srt`

## ADDED Requirements

### Requirement: Translate subtitles from the gallery
While the translation model is downloaded, the context menu of an MP4 that has a transcript and a known language SHALL offer Translate subtitles…, which opens a target picker preselecting the default target and queues one translation on confirm. A video without a transcript or with an unknown language SHALL NOT offer it.

#### Scenario: Translate to Japanese
- **WHEN** the user right-clicks a Vietnamese video with `a.srt`, chooses Translate subtitles…, picks Japanese and confirms
- **THEN** a translation item for Japanese is queued and later `a.ja.srt` exists

#### Scenario: Default preselected
- **WHEN** the picker opens for an English video
- **THEN** Vietnamese is preselected and English is not offered

#### Scenario: No transcript
- **WHEN** a video has no `a.srt`
- **THEN** the menu has no Translate subtitles…

### Requirement: Tiles show subtitle languages
A video tile in the gallery grid SHALL show one small badge per subtitle track at its bottom-left: the original transcript as the video's language code (or `CC` when the language is unknown), then each translation's code, upper-case, at most three badges followed by `+N` for the rest. A video without subtitles SHALL show none. The viewer thumbnail strip SHALL NOT show them.

#### Scenario: Several subtitles
- **WHEN** a Vietnamese video has `a.srt`, `a.en.srt` and `a.ja.srt`
- **THEN** its tile shows `VI`, `EN` and `JA`

#### Scenario: Unknown language
- **WHEN** a video has `a.srt` and its language is not known
- **THEN** its tile shows `CC`

#### Scenario: Many subtitles
- **WHEN** a video has the original and five translations
- **THEN** its tile shows three badges and `+3`
