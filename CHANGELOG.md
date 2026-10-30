# Changelog

All notable changes to Snapr. Versions follow semantic versioning.

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
