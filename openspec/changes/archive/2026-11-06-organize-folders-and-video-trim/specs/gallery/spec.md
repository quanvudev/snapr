## MODIFIED Requirements

### Requirement: Gallery lists every file in the output folder
The gallery SHALL list every file directly inside the resolved output folder, its `Images` subfolder and its `Videos` subfolder, and classify it by extension, case-insensitive: photos are `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`; videos are `mp4` and `webm`; every other file is an other file. Any other subfolder, and anything nested deeper inside `Images` or `Videos`, SHALL be ignored. Files are not required to follow snapr's naming, and a file is listed whichever of the three folders it is in. An `srt` file whose name without extension equals that of a video in the same folder is that video's transcript and SHALL NOT be listed. An `srt` file named `<video name>.<code>.srt`, case-insensitive, where `<code>` is a supported translation target, for a video in the same folder is that video's translation and SHALL NOT be listed either.

#### Scenario: Mixed folder
- **WHEN** the output folder holds `a.png`, `notes.txt` and subfolders `Images` containing `c.jpg`, `Videos` containing `b.MP4` and `clip.mov`, and `old` containing `d.png`
- **THEN** the gallery shows `a.png` and `c.jpg` as photos, `b.MP4` as a video, `notes.txt` and `clip.mov` as other files, and nothing from `old`

#### Scenario: Nested folder
- **WHEN** `Images` contains a subfolder `x` holding `e.png`
- **THEN** the gallery does not show `e.png`

#### Scenario: Empty or missing folder
- **WHEN** the output folder, `Images` and `Videos` have no file, or do not exist
- **THEN** the gallery shows an empty state message instead of a grid

#### Scenario: Transcript beside its video
- **WHEN** `Videos` holds `a.mp4`, `a.srt` and `b.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.srt` as an other file, and does not show `a.srt`

#### Scenario: Transcript in another folder
- **WHEN** `Videos` holds `a.mp4` and the output folder root holds `a.srt`
- **THEN** the gallery shows `a.srt` as an other file and `a.mp4` has no transcript

#### Scenario: Translation beside its video
- **WHEN** `Videos` holds `a.mp4`, `a.en.srt` and `b.en.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.en.srt` as an other file, and does not show `a.en.srt`

#### Scenario: Several translations
- **WHEN** `Videos` holds `a.mp4`, `a.srt`, `a.vi.srt`, `a.zh-Hant.srt` and `a.xx.srt`
- **THEN** the gallery shows `a.mp4` and `a.xx.srt` (not a supported code), and hides the others

### Requirement: File actions are limited to the output folder

Thumbnails, viewer content, open, reveal, copy, extract text, pin, delete and every video task (export GIF, compress, transcribe, translate, export with subtitles, trim) SHALL only act on files directly inside the resolved output folder, its `Images` subfolder or its `Videos` subfolder. Viewer content SHALL only be served for photo and video files. Pin and extract text SHALL only act on photo files. A request for any other path SHALL be refused without touching the file.

#### Scenario: Path outside the folder

- **WHEN** a request names a path outside those three folders, directly or through `..`
- **THEN** the request is refused and nothing is read, opened or deleted

#### Scenario: File in a subfolder

- **WHEN** a request names `Videos\a.mp4` or `Images\b.png` inside the output folder
- **THEN** it is accepted

#### Scenario: File in another subfolder

- **WHEN** a request names `old\d.png` or `Images\x\e.png` inside the output folder
- **THEN** the request is refused

#### Scenario: Pin outside the folder

- **WHEN** a pin request names a path outside those three folders, directly or through `..`
- **THEN** the request is refused and no pin appears

#### Scenario: Pin a non-photo file

- **WHEN** a pin request names a video or other file inside the output folder
- **THEN** the request is refused and no pin appears

#### Scenario: Extract text outside the folder

- **WHEN** an extract text request names a path outside those three folders or a video inside them
- **THEN** the request is refused and nothing is recognized

#### Scenario: Viewer content for other file

- **WHEN** viewer content is requested for `notes.txt` inside the output folder
- **THEN** the request is refused

#### Scenario: Output folder changed

- **WHEN** the user changes the output folder in General
- **THEN** the gallery lists the new folder and files of the old folder can no longer be read

### Requirement: Bulk file access is limited to the output folder

Every bulk delete, copy and pin path SHALL point to a file directly inside the resolved output folder, its `Images` subfolder or its `Videos` subfolder, and be of a type the action accepts. If any path is invalid, the whole request SHALL be refused before any file or clipboard change. Delete accepts photos, videos and other files; copy and pin accept photos only.

#### Scenario: Outside the folder

- **WHEN** a bulk request includes a path outside those three folders
- **THEN** no file is deleted, copied or pinned and the clipboard is unchanged

#### Scenario: Mixed valid and invalid delete

- **WHEN** a bulk delete has one valid path and one outside path
- **THEN** no file is deleted

#### Scenario: Files from both subfolders

- **WHEN** a bulk delete includes `Images\a.png` and `Videos\b.mp4`
- **THEN** both are accepted

#### Scenario: Non-photo copy

- **WHEN** a bulk copy request includes a video
- **THEN** the request is refused and the clipboard is unchanged

#### Scenario: Non-photo pin

- **WHEN** a bulk pin request includes an other file
- **THEN** no pins are created

#### Scenario: Video delete

- **WHEN** a bulk delete request includes a video inside the output folder
- **THEN** it is accepted
