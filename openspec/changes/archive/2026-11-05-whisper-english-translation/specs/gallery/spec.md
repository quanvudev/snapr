## MODIFIED Requirements

### Requirement: Gallery lists every file in the output folder
The gallery SHALL list every file directly inside the resolved output folder and classify it by extension, case-insensitive: photos are `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`; videos are `mp4` and `webm`; every other file is an other file. Subfolders SHALL be ignored. Files are not required to follow snapr's naming. An `srt` file whose name without extension equals that of a video in the folder is that video's transcript and SHALL NOT be listed. An `srt` file named `<video name>.en.srt`, case-insensitive, for a video in the folder is that video's English translation and SHALL NOT be listed either.

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

### Requirement: Deleting a video removes its transcript
Deleting an MP4, from the grid, the viewer or a bulk delete, SHALL also move its `<video name>.srt` and its `<video name>.en.srt` to the Recycle Bin when they exist. A failure to remove either SHALL NOT fail the video's delete.

#### Scenario: Delete a transcribed video
- **WHEN** the user deletes a video that has a transcript and confirms
- **THEN** both the video and its `.srt` are in the Recycle Bin

#### Scenario: Delete a translated video
- **WHEN** the user deletes a video that has both `.srt` and `.en.srt` and confirms
- **THEN** the video and both subtitle files are in the Recycle Bin

#### Scenario: Delete a video without transcript
- **WHEN** the user deletes a video that has no `.srt` and confirms
- **THEN** the video is in the Recycle Bin and no error is shown

## ADDED Requirements

### Requirement: Video language is known
Snapr SHALL remember the spoken language of each MP4: a finished transcription records the language Whisper detected, or the fixed transcript language when one is set. The record SHALL survive a restart and SHALL be dropped when the video is deleted.

#### Scenario: Recorded by transcription
- **WHEN** transcript language is Auto-detect and a Vietnamese video finishes transcribing
- **THEN** the video's language is Vietnamese, also after snapr restarts

### Requirement: Video language is detected on demand
For an MP4 with no recorded language, opening the transcript pane SHALL detect it from the first 30 seconds of audio with the selected Whisper model when that model is downloaded, without blocking the viewer, and record it. A video with no audio track or no downloaded model SHALL stay unknown.

#### Scenario: Detected on demand
- **WHEN** a video was transcribed by an older snapr, the Small model is downloaded and the user opens its transcript pane
- **THEN** after a moment the video's language is known and the pane and context menu reflect it

#### Scenario: Model missing
- **WHEN** no Whisper model is downloaded and the user opens the transcript pane of a video with no recorded language
- **THEN** the language stays unknown and no error is shown

### Requirement: Translate to English from the gallery
The context menu of an MP4 whose known language is not English SHALL offer Translate to English, which queues one English translation of its speech. A finished translation SHALL be written as `<video name>.en.srt` beside the video, replacing an older one, and SHALL NOT change `<video name>.srt`. It SHALL be refused, queueing nothing, when the selected Whisper model is not downloaded or the video is known to be English. Photos, English videos and videos of unknown language SHALL NOT offer it.

#### Scenario: Translate a Vietnamese recording
- **WHEN** the user right-clicks a Vietnamese MP4 that has `a.srt` and chooses Translate to English
- **THEN** a translate item appears in the queue, and when it finishes `a.en.srt` holds English text with timings while `a.srt` is unchanged

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
The transcript pane SHALL show an Original (`.srt`) / English (`.en.srt`) switch only when the video's known language is not English or its `.en.srt` exists. Each track SHALL have its own loading, missing, running, error and segment states; a missing English track offers Translate to English instead of Create transcript. Editing, Copy all and seeking SHALL act on the shown track only. The chosen track SHALL persist across videos that have the switch.

#### Scenario: English video has no switch
- **WHEN** the user opens the transcript pane of an English video that has no `a.en.srt`
- **THEN** no Original / English switch is shown and the original transcript is listed

#### Scenario: Switch appears after detection
- **WHEN** the pane of a video with unknown language is open and detection finds Vietnamese
- **THEN** the Original / English switch appears

#### Scenario: Switch to English
- **WHEN** a video has `a.srt` and `a.en.srt` and the user switches the pane to English
- **THEN** the English segments are listed, clicking one seeks the video and the active segment follows playback

#### Scenario: No translation yet
- **WHEN** a video has only `a.srt` and the user switches to English
- **THEN** the pane offers Translate to English, and choosing it queues a translation and shows its progress in the pane

#### Scenario: Tracks run independently
- **WHEN** a translation of a video is running and the user switches to Original
- **THEN** the original transcript is listed and editable, and switching back to English shows the running progress

#### Scenario: Edit the English track
- **WHEN** the user edits an English segment and presses Enter
- **THEN** `a.en.srt` is rewritten with the new text and `a.srt` is unchanged

#### Scenario: Edit refused while translating
- **WHEN** a translation of a video is waiting or running
- **THEN** editing is hidden on the English track, and a save of that track is refused

### Requirement: Export with subtitles chooses a track
The Export with subtitles dialog SHALL offer a Track choice Original / English when the video has both subtitle files, preselecting the track shown in the transcript pane when opened from it and Original otherwise. When only one exists the choice SHALL be hidden and that track used. The preview cue SHALL come from the chosen track. An English export SHALL be written as `<video name>_subtitled_en.mp4` (`_N` when taken), an original one as `<video name>_subtitled.mp4`.

#### Scenario: Export English subtitles
- **WHEN** a video has both tracks and the user picks English and exports
- **THEN** a subtitle item is queued and the result `a_subtitled_en.mp4` shows the English cues

#### Scenario: Only original
- **WHEN** a video has `a.srt` and no `a.en.srt` and the user opens Export with subtitles
- **THEN** no Track choice is shown and the export uses `a.srt`

#### Scenario: Only English
- **WHEN** a video has `a.en.srt` and no `a.srt`
- **THEN** the context menu offers Export with subtitles, the dialog shows no Track choice and the export uses `a.en.srt`
