# Changelog

All notable changes to Snapr. Versions follow semantic versioning.

## [1.12.0] - 2026-11-05

### Added
- Translate subtitles into 38 languages on your computer with the optional HY-MT1.5 model (about 1.4 GB, Settings → AI models). Vietnamese videos go to English and English videos to Vietnamese by default, and you can pick any other target from the gallery (right-click → Translate subtitles…) or the transcript pane. The model's licence (Tencent HY Community License) is shown and must be accepted before the first download; it does not apply in the EU, the UK or South Korea.
- Without the translation model, videos that are not in English can still get English subtitles from Whisper.
- Each translation is its own subtitle track: switch between the original and the translations in the transcript pane, edit any line, and choose the track when exporting a video with subtitles.
- The Original track now shows the video's language, and gallery tiles show a badge for each subtitle language a video has.
- Recordings can be translated automatically after they are transcribed (Settings → Recording → Also translate subtitles).
- Redact tool in Draw capture (X): draw black or pixelated boxes over anything you want to hide. Press Detect (D) to cover email addresses, phone numbers, card numbers, IP addresses and secrets such as API keys and tokens in one go, then delete any box you don't want or undo the whole batch.
- Optional automatic redaction of full-screen and window captures (Settings → Capture → Redaction): sensitive text is covered before the screenshot is saved, copied or previewed, and the original is not kept. It is off by default.
- Redaction settings: default style (black or pixelate), which kinds of information to look for, your own patterns (regular expressions), and the text recognition engine, Windows OCR (fast, the default) or PaddleOCR (more accurate, slower).

## [1.11.3] - 2026-11-05

### Fixed
- Export with subtitles no longer gets stuck on videos whose sound ends before the picture (for example a recording that was paused), and cancelling an export always stops it right away.

### Changed
- Maintenance and stability improvements.

## [1.11.2] - 2026-11-04

### Fixed
- Long file names no longer break the notification toasts in the settings window: long titles and descriptions are now cut to a few lines.

## [1.11.1] - 2026-11-03

### Fixed
- Export with subtitles no longer gets stuck partway through (for example at 4%) on longer videos. Cancel now also stops an export that is in progress.

## [1.11.0] - 2026-11-03

### Added
- Edit transcripts: in the viewer's transcript pane, hover a line and click the pencil (or double-click it) to fix its text. Enter saves it into the `.srt` file with the same timings, Escape cancels. Editing is unavailable while the video is being transcribed.
- Export with subtitles: burn a transcript into a copy of the video from the transcript pane (Captions button). Choose the position (Bottom / Top) and the size (Small / Medium / Large) with a preview. The result is saved as `_subtitled.mp4` next to the original, which is kept, the sound is kept, and the export runs in the background queue with progress and Cancel. Videos that already have a transcript also offer it when you right-click them in the gallery.
- Background tasks panel: a button next to the close button of the settings window opens a side panel with the tasks in progress (progress and Cancel) and a short message tells you when tasks are added to the queue. The queue no longer stays on screen as a toast.
- Task history: finished, failed and cancelled background tasks stay listed in the background tasks panel until you restart Snapr or clear the list. Show a result in Explorer, or retry a failed or cancelled task from there.
- AI models page: download, switch and delete the PaddleOCR and Whisper models in one place, with the total disk space they use. The Text and Recording pages show whether the model they need is ready and link to it, and each model links back to the setting that uses it.

### Changed
- Settings are split into separate pages in the sidebar (General, Capture, Text, Recording, Hotkeys, Action Ring, AI models, with About at the bottom) instead of one long scrolling page, and related options are grouped into cards.
- The Action Ring on / off switch and its hotkey are now on the Action Ring page, which is always listed in the sidebar.
- Exporting with subtitles is much faster: videos are decoded on the graphics card when possible and frames without a subtitle are no longer reprocessed. GIF exports also decode faster.

## [1.10.0] - 2026-11-03

### Added
- Export GIF: turn a recording into an animated GIF from the gallery (right-click, the viewer or the bulk bar). Trim it with a preview, then pick the width (480 / 640 / 800 / Original) and the frame rate (10 / 15 / 20). The dialog shows an estimated file size.
- Compress video: shrink a recording to Low, Medium or High quality at 1080p, 720p or its original size. The dialog shows the estimated file size, and the original is kept.
- Transcripts: create subtitles for a recording's speech entirely on your computer with Whisper (Base, Small or Turbo, downloaded from Settings). Transcripts are saved as an `.srt` file next to the video. In the viewer, a transcript pane (Captions button or `T`) follows playback, jumps to a line when you click it and copies everything at once. The language can be Auto-detect, Vietnamese or English.
- Background tasks: GIF exports, compressions and transcripts run one at a time in a queue, with progress and Cancel in a toast. You get a notification when one finishes while the window is closed. Several videos can be queued at once from the gallery selection.
- Recording quality setting (Low / Medium / High) to trade file size against sharpness.
- Optional automatic compression after recording (to 1080p or 720p, replacing the original, which goes to the Recycle Bin) and automatic transcription of recordings that have sound.

### Changed
- Recordings now pick their bitrate from their size, frame rate and quality instead of a fixed 15 Mbps, so most files are smaller at the default Medium quality.
- Deleting a video from the gallery also moves its transcript to the Recycle Bin, and transcripts no longer show up as separate gallery items.

## [1.9.1] - 2026-10-30

### Fixed
- The action ring now closes when you click outside it or switch to another window, including when it was opened with a mouse button and did not get focus.

## [1.9.0] - 2026-10-30

### Added
- Install with Scoop: `scoop bucket add snapr https://github.com/quanvudev/snapr-releases`, then `scoop install snapr/snapr`. Scoop copies get the same in-app updates, and Start with Windows keeps working after `scoop update`. Uninstalling with Scoop removes the Start with Windows entry and keeps your settings and captures.

### Changed
- Updates install faster and without an installer window: Snapr downloads the new version, checks its signature, swaps itself in place and restarts in the tray. Settings, captures, OCR models and Start with Windows are kept.

## [1.8.0] - 2026-10-29

### Added
- Recording panel: the record hotkey (or the ring's Record) now opens a small floating panel first, with a round webcam preview and a toolbar to choose what to record before you press Start.
- Recording modes: Full screen (starts at once on the monitor under the cursor), Window (click a window) or Area (drag a rectangle), picked from the panel and remembered.
- Microphone and system audio: pick a microphone and the system audio source (all system audio, or one output device) in the panel; both are mixed into the video's sound track. The microphone level shows as a glowing ring around the audio button.
- Webcam: a round webcam bubble appears in the recording and can be dragged anywhere, even while recording. The first camera is selected automatically when one is found.
- While recording, a compact bar under the webcam shows the timer with Stop, Pause / Resume and Delete (click twice to discard the recording). Pause cuts the paused time out of the video.
- Tray: a Pause recording / Resume recording item.

### Changed
- Recordings now keep a true 60 frames per second instead of repeating frames.
- The recording panel and bar never appear in the video.

### Fixed
- Area recordings are no longer saved upside down.

## [1.7.0] - 2026-10-29

### Added
- Draw capture: the selected area stays editable. Drag the handles on its corners and edges to resize it, or hold Space and drag inside it to move it. Shapes you already drew stay where they are on screen.
- Draw capture: the area's size in pixels is shown next to it while you edit.
- Draw capture: Redo (Ctrl+Y or Ctrl+Shift+Z) and Undo / Redo buttons on the toolbar, dimmed when there is nothing to undo or redo.
- Draw capture: Copy (Ctrl+C) puts the annotated image on the clipboard and Pin (Ctrl+P) pins it on screen, both without saving a file.

### Changed
- Draw capture: the toolbar is now two rows centred under the area. The main row (tools, Undo / Redo, Pin / Copy, Done / Cancel) never moves; the second row holds the options for the current tool.
- Draw capture: outline and fill colours have their own buttons (a ring for the outline, a square for the fill), and the colour palette opens in place of the options row instead of a separate popup.

## [1.6.0] - 2026-10-29

### Breaking
- The installer is now a per-user NSIS setup only; the MSI is no longer published. If you installed the MSI, uninstall it and install `Snapr_1.6.0_x64-setup.exe` once. Later versions update in place.

### Added
- In-app updates: Snapr checks for a new version at launch and every 6 hours, shows a notification and an "Update to x.y.z" tray item, and installs it when you choose Update. Updates wait while a recording or countdown is running.
- Settings, General: "Check for updates automatically" (on by default).
- Settings, About: the update status with Check now, download progress and Retry.
