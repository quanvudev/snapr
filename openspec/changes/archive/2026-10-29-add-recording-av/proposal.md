# Proposal

## Why

Recordings are silent and show only the screen, so they are of little use for tutorials, demos or bug reports with narration. Users also cannot see or control a recording on screen beyond the tray item. A recording panel with a microphone, system audio and a draggable webcam, in the style of Loom, makes Snapr usable for those videos without a second tool.

## What Changes

- **Recording panel.** The record hotkey and the ring Record wedge first open a floating panel instead of the picker. The panel is excluded from the recording, can be dragged, and lets the user:
  - pick a microphone (or none), with a live input level;
  - pick a camera (or off);
  - turn system audio on or off;
  - start (Enter) or close (Esc, ✕).
  - Start opens the existing picker. After the pick (and the countdown when it is on) recording starts.
- **Panel during the recording.** The panel shrinks to one row:
  - a recording dot, the elapsed time and the mic level;
  - read-only icons for the devices in use, plus a camera icon that hides or shows the webcam;
  - Pause / Resume and Stop.
  - Devices cannot be changed once recording has started.
- **Webcam.** A round, always-on-top webcam window (about 180 logical px) that **is** recorded:
  - it starts in the bottom-left corner of the recorded monitor or area;
  - it can be dragged before and during the recording;
  - the panel's camera icon hides it (it then leaves the video) and shows it again in the same place.
- **Audio.** Recordings contain AAC audio (48 kHz stereo) from the selected microphone and/or the default output device's loopback, mixed. With neither selected, the file has no audio track, as today.
- **Pause / Resume** from the panel and from a new tray item. Paused time is cut from the video and the audio, so the file plays without a gap. The timer stops while paused.
- **Window pick with the camera on.** The pick records the window's rectangle as an area, so the webcam window is included.
- **Remembered choices.** The microphone, camera and system audio choices are saved in the config and reused next time. A saved device that is gone falls back to the default microphone or to no camera.
- **Lost microphone.** If the microphone disappears mid-recording, recording continues with silence and the panel shows a warning.
- **BREAKING (behaviour):** the record hotkey no longer opens the picker directly; it opens the panel, and recording needs one more step (Start or Enter).
- **Out of scope:** resizing or reshaping the webcam, background blur, echo cancellation, choosing the loopback output device, a pause hotkey.

## Capabilities

### New Capabilities

- `recording-panel`: The floating recording panel and the webcam window. It covers device selection and persistence, the setup and recording states, dragging, the webcam toggle, the Pause, Resume and Stop controls, and keeping the panel out of the video.

### Modified Capabilities

- `screen-recording`: Several requirements change:
  - the record hotkey opens the panel first;
  - the hotkey closes an open panel;
  - the tray gains Pause / Resume;
  - the visible-recording state includes paused;
  - recordings may contain audio;
  - a window pick with the camera on records the window's area.
  - New requirements cover audio capture, pause and resume, and a lost microphone.

## Impact

- **Backend:**
  - a new audio capture module (WASAPI microphone and loopback via `cpal`, resampled and mixed to 16-bit 48 kHz stereo, sent with `send_audio_buffer`);
  - `recorder.rs`: audio enabled, a pause offset applied to video and audio timestamps;
  - `lib.rs`: a new flow `Record → panel → Start → picker → countdown → record`, panel and webcam window management, a tray Pause item, `State` for the paused state;
  - `config.rs`: `record_mic`, `record_camera`, `record_system_audio` with serde defaults;
  - new commands: list microphones, start, cancel, pause / resume, stop, toggle webcam;
  - `WDA_EXCLUDEFROMCAPTURE` on the panel window.
- **Frontend:**
  - two new Tauri webview windows (`panel`, `camera`) with their own small React views;
  - the webcam is shown with `getUserMedia` in WebView2 (camera permission granted by the host);
  - capabilities for the new windows;
  - i18n strings.
- **Dependencies:** `cpal` (new).
