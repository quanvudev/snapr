# Tasks

## 1. Area recording backend

- [x] 1.1 Add `Target::Area { monitor, rect }` to `recorder.rs`: start a monitor session, build the encoder at the area size, crop each sent frame with `buffer_crop` and send it with `send_frame_buffer`, and crop `keep()` the same way; verify `cargo clippy` in `src-tauri/`.
- [x] 1.2 Temporarily start an area recording with a hard-coded rect and verify the MP4 plays, has the even-rounded size, shows the right rectangle the right way up, and that window and monitor recordings are unchanged.

## 2. Record picker drag

- [x] 2.1 Let the record-mode window picker switch to an area rectangle after a 4 px drag and return `Shot::Region`, keeping click behavior for shorter movements; move the desktop interception in `Snap::window_event` from press to release; verify `cargo clippy` and that the window screenshot picker ignores drags.
- [x] 2.2 Convert the region to `Target::Area` (monitor under the drag start, clip, even size, under 16x16 falls back to a click) and call `record`; verify a drag records only the area, a drag across monitors is clipped to the first, a tiny drag records the window, and clicking the desktop still records the monitor.

## 3. Area frame

- [x] 3.1 Add the click-through, topmost, non-activating, `WDA_EXCLUDEFROMCAPTURE` layered frame window drawn 2 px outside the area; create it on area recording start and drop it on `Recorded` / `RecordFailed`; verify the frame is visible, clicks pass through it, it is absent from the saved video, and it disappears on stop and on failure.
- [x] 3.2 Verify the frame and the recorded area line up on a 100% and a 200% DPI monitor.

## 4. Countdown

- [x] 4.1 Add `record_countdown` to `Config` with `#[serde(default)]`, the field in `lib/api.ts`, a `countdown-row.tsx` Switch in `settings/recording/` and en/vi strings; verify `bunx tsc --noEmit`, `cargo clippy`, and that an existing config loads with the countdown off.
- [x] 4.2 Add `Snap.countdown` with the layered, non-activating, excluded-from-capture digit window centered on the target, ticking from `about_to_wait`, hiding pins and enabling Stop at start, and dropping the window before `record` on the last tick; verify the countdown for window, monitor and area picks and that no digit is in the first frames of the video.
- [x] 4.3 Add the cancel paths: `Action::Record` during a countdown cancels without opening the picker, the tray Stop sends `Action::CancelCountdown` when there is no recorder, and cancel/failure restore pins, tooltip and Stop; verify each path creates no file and that closing the picked window during the countdown shows the error.

## 5. Integration

- [x] 5.1 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`, then a `bun run tauri dev` pass over every scenario in this change's specs, including the ring Record wedge during a countdown and a screenshot taken during an area recording; record any gap.
- [x] 5.2 Update `CLAUDE.md` (recording: area target and crop path, frame window, countdown state and cancel paths, `record_countdown`) and verify it matches the implemented behavior.

## Workflow follow-up

- Run `openspec validate recording-plus --strict` before review.
- Archive the change after the implementation is reviewed.
