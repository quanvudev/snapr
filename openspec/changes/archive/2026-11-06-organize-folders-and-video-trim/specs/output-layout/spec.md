## ADDED Requirements

### Requirement: Output is organized into Images and Videos
The output folder SHALL hold two subfolders named `Images` and `Videos`. snapr SHALL write screenshots and GIF exports into `Images`, and recordings into `Videos`. A file derived from a video (a compressed, subtitled or trimmed copy, a transcript or a translation) SHALL be written into the folder of its source video. A subfolder SHALL be created when snapr first writes into it.

#### Scenario: Screenshot
- **WHEN** the user takes a screenshot
- **THEN** the PNG is saved in `<output>\Images`

#### Scenario: Recording
- **WHEN** a recording ends
- **THEN** the MP4 is saved in `<output>\Videos`

#### Scenario: GIF export
- **WHEN** the user exports a GIF from a video in `<output>\Videos`
- **THEN** the GIF is saved in `<output>\Images`

#### Scenario: Transcript beside the video
- **WHEN** a video in `<output>\Videos` is transcribed
- **THEN** its `.srt` is written in `<output>\Videos` next to it

#### Scenario: Derived copy of a root video
- **WHEN** the user compresses a video that is still directly in the output folder
- **THEN** the compressed copy is written directly in the output folder

#### Scenario: Missing subfolder
- **WHEN** `Images` does not exist and the user takes a screenshot
- **THEN** `Images` is created and the screenshot is saved in it

### Requirement: Existing files are moved once into the layout
On the first start after this change, snapr SHALL move every photo directly in the output folder into `Images` and every video directly in it into `Videos`, keeping file names and adding a `_N` suffix when the name is taken. A moved video's transcript and translations SHALL move with it, renamed to match the video's new name, and its stored language SHALL follow it. Other files SHALL stay where they are. The move SHALL run only once; files added to the root afterwards SHALL NOT be moved. A file that cannot be moved SHALL stay in the root and SHALL NOT stop the others.

#### Scenario: Mixed root
- **WHEN** the output folder holds `a.png`, `b.gif`, `c.mp4`, `c.srt`, `c.en.srt` and `notes.txt` at the first start after the update
- **THEN** `Images` holds `a.png` and `b.gif`, `Videos` holds `c.mp4`, `c.srt` and `c.en.srt`, and `notes.txt` stays in the root

#### Scenario: Name taken
- **WHEN** the root holds `c.mp4` and `c.srt` and `Videos` already holds `c.mp4`
- **THEN** they are moved to `Videos\c_1.mp4` and `Videos\c_1.srt`

#### Scenario: Language follows the video
- **WHEN** a video whose language is known is moved
- **THEN** the gallery still shows that language for it after the move

#### Scenario: Only once
- **WHEN** the user later copies `x.png` into the output folder root and restarts snapr
- **THEN** `x.png` stays in the root and is still shown in the gallery

#### Scenario: Locked file
- **WHEN** a root video is open in another app and cannot be moved
- **THEN** it stays in the root, the other files are moved, and the gallery still lists it
