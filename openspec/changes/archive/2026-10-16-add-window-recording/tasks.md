# Tasks

## 1. Dependency and API check

- [x] 1.1 Add `windows-capture = "2"` to `src-tauri/Cargo.toml`. Done when `cargo build` in `src-tauri/` succeeds.
- [x] 1.2 Confirm the `windows-capture` API from the crate source in the cargo registry, then record the answers in a short note at the bottom of `design.md`. The questions are:
  - how to build a window item from a raw hwnd;
  - how `start_free_threaded`, `CaptureControl::stop` and `callback()` behave;
  - whether `buffer_crop` + `send_frame_buffer` expects RGBA or BGRA;
  - whether `send_frame` scales mismatched sizes.

  Done when the note answers all four and decision 5 holds or its fallback is chosen.

## 2. Shared naming

- [x] 2.1 Extract `capture::unique_path(dir, ext)` from `capture::save` and make `save` use it with `"png"`. Done when `cargo clippy` is clean and a screenshot taken twice in one second still produces `<stamp>.png` and `<stamp>_1.png`.

## 3. Config and hotkey settings

- [x] 3.1 Add `record` (default `"4"`) to `Hotkeys` in `config.rs` and make `combos()` return `[String; 4]`. Done when `cargo clippy` is clean and an old `config.toml` without `record` loads with `record = "4"`.
- [x] 3.2 Update `settings.rs::check` and any other `[_; 3]` hotkey arrays for four slots. Done when `cargo clippy` is clean and a duplicate between record and another hotkey reports `duplicate`.
- [x] 3.3 Frontend changes:
  - add `"record"` to `SLOTS` in `src/lib/hotkeys.ts`;
  - add `record` to the `Hotkeys` type in `src/lib/api.ts`;
  - add the record label in `en` and `vi` in `src/lib/i18n.ts`.

  Done when `bunx tsc --noEmit` passes and the setup page and the Hotkeys group show four rows.

## 4. Recorder

- [x] 4.1 Create `src-tauri/src/recorder.rs` with `Recorder::start(hwnd, path, proxy)` and `Recorder::stop(self)`, following design decisions 1, 2 and 5:
  - capture settings: cursor on, default border, about 30 fps cap;
  - an even fixed size with crop-or-skip frames;
  - the last frame kept as an `RgbaImage`;
  - one finalize step shared by `stop` and `on_closed` that sends `Action::Recorded` or reports the error in the tooltip.

  Done when `cargo clippy` is clean.

## 5. Wiring in lib.rs

- [x] 5.1 Add `Action::Record` and `Action::Recorded(RgbaImage, PathBuf)`, and add `recording: Mutex<Option<Recorder>>` and the stop `MenuItem` to `State`. Done when `cargo clippy` is clean.
- [x] 5.2 In `apply()`, register the record hotkey. If `State.recording` holds a recorder, take it and stop it. Otherwise send `Action::Record`. Done when pressing the record hotkey while recording stops it without opening the picker.
- [x] 5.3 In `Snap`, `Action::Record` opens `Pick::Window` with `record_pick = true`. When the outcome is a `Shot::Window` and the flag is set, call `Recorder::start` with `capture::unique_path(dir, "mp4")`, store the recorder, set the tooltip to `snapr: recording…` and enable the stop item. On error, show `snapr: <error>`. Every outcome clears the flag. Done when the record hotkey + click starts a recording with the yellow border, and Escape starts nothing.
- [x] 5.4 Handle `Action::Recorded`:
  - clear `State.recording`;
  - restore the tooltip;
  - disable the stop item;
  - open the toast like `Action::Saved`.

  Done when, after stopping, the toast shows the last frame and clicking it opens Explorer on the `.mp4`.
- [x] 5.5 Tray changes:
  - add a "Stop recording" item, disabled by default, that takes and stops the recorder;
  - make "Quit" stop any running recorder before `app.exit(0)`.

  Done when both produce a playable MP4.

## 6. End-to-end check and docs

- [x] 6.1 Run `bun run tauri dev` and confirm every scenario in `specs/screen-recording/spec.md` by hand. That includes:
  - closing the recorded window mid-recording;
  - resizing the window;
  - taking a screenshot during a recording;
  - preview 0;
  - an odd window size, checking that the video has even dimensions in file properties.
- [x] 6.2 Update `CLAUDE.md` Architecture:
  - the record action and `recorder.rs`;
  - `State.recording`;
  - the four hotkey slots.

  Done when the doc matches the code.

## Workflow follow-up

- Run `/opsx:archive` after review to merge the spec deltas into `openspec/specs/`.
