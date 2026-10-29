# Proposal

## Why

Recording is all-or-nothing: a whole window or a whole monitor, starting the instant the user clicks. Demos and bug reports usually need one part of the screen (a panel, a dialog, a browser viewport without its chrome), and the first second of every clip is the user moving the mouse back from the picker. Area recording and a short countdown fix both without changing the recording pipeline or file format.

## What Changes

- **Area recording**: in the picker opened by the record hotkey, dragging a rectangle records that area of the monitor where the drag started. A click keeps today's behavior (window, or the monitor on the desktop). The window screenshot picker is unchanged.
- While an area recording runs, a thin frame is drawn just outside the recorded area so the user can see it; the frame is never in the recording.
- **Countdown**: an optional 3-second countdown (3, 2, 1) shown over the target after the pick and before recording starts. Pressing the record hotkey or choosing Stop recording during the countdown cancels it and no file is created.
- A new Recording setting turns the countdown on or off, default off.
- Out of scope: audio (system or microphone), GIF export, pause/resume, an area spanning several monitors, a configurable countdown length, a separate area-record hotkey.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `screen-recording`: the record picker accepts a dragged area; area recordings have their own size rule and a visible frame; a countdown can precede any recording and can be cancelled.
- `settings-window`: the Recording group shows the countdown switch next to the frame rate.

## Impact

- Backend: `overlay.rs` record mode accepts a drag; `recorder.rs` gains `Target::Area { monitor, rect }` that crops monitor frames and feeds the encoder raw buffers; a new layered frame/countdown window (reusing the `ring` / `pin` layered helpers, excluded from capture); `lib.rs` countdown state and cancel paths; `config.rs` `record_countdown` (`#[serde(default)]`).
- Frontend: a `countdown-row.tsx` in `settings/recording/`, the config type in `lib/api.ts`, en/vi strings.
- No new dependency. Output stays H.264 MP4 named by the start timestamp; for a countdown, the timestamp is taken when recording actually starts.
