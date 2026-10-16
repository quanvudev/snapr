# Proposal

## Why

snapr only takes still screenshots. Users who want to show a flow or a bug in one app have to switch to a separate recorder. A fourth hotkey that records a picked window to MP4 covers the most common case without leaving snapr.

## What Changes

- New **record** hotkey (default `4`, with the configured modifiers). It opens the existing window picker. Clicking a window starts recording that window to an MP4 file.
- Pressing the record hotkey again, or choosing **Stop recording** in the tray menu, stops the recording and finalizes the file.
- While recording, Windows draws its capture border around the window and the tray tooltip reads `snapr: recording…`.
- When the recording ends (stopped, window closed or app quit), the file is saved in the output folder with the same timestamp naming as screenshots and a `.mp4` extension, and the existing preview toast shows the last frame. Clicking the toast opens Explorer on the video.
- Video only: H.264, about 30 fps, cursor included. No audio and no clipboard copy.
- The setup page and the Hotkeys settings group show four action hotkeys instead of three. An existing `config.toml` without the new key gets the default.
- New dependency: `windows-capture` (Windows Graphics Capture + Media Foundation encoder).

Out of scope: audio, recording a monitor or an area, pause/resume, a recording timer, configurable fps or quality.

## Capabilities

### New Capabilities
- `screen-recording`: Recording screen content to a video file: starting, stopping, the recording indicator, the output file and its preview.

### Modified Capabilities
- `app-startup`: The setup page shows four action hotkeys (adds Record).
- `settings-window`: The Hotkeys group shows four action hotkeys (adds Record).

## Impact

- Backend (`src-tauri/src`):
  - new `recorder.rs`;
  - `lib.rs` gets the record action, overlay routing, tray item and stop on quit;
  - `config.rs` gets the `record` hotkey;
  - `settings.rs` validates four hotkeys;
  - `capture.rs` gets a small extraction of the unique-name logic.
  - `overlay.rs` and `toast.rs` are unchanged.
- Frontend: `lib/hotkeys.ts` (`SLOTS`), `lib/api.ts` (`Hotkeys` type), `lib/i18n.ts` (labels). The hotkey rows render from `SLOTS`, so no new components are needed.
- `Cargo.toml`: adds `windows-capture`.
- Config: the new `hotkeys.record` field. This is backward compatible through serde defaults.
