# Tasks

## 1. Moonshine model and streaming engine

- [x] 1.1 Pin the streaming models: for each of en/es/de/ja (Small) and vi/zh/ar/tl (Tiny), record the files, sizes and SHA-256 from `moonshine-voice-assets` at a fixed commit, and confirm the `tokenizer.bin` format. Verify by loading every `.ort` graph with `onnx::session` on the pinned CPU runtime in an ignored test (fall back to the `.onnx` sources if `.ort` fails, and update design.md).
- [x] 1.2 Add `Workload::Captions` to `onnx.rs` (CPU only in `on_card`, half cores, no spin) and extend `places_workloads`; verify with `cargo test --lib onnx`.
- [x] 1.3 Add `captions/tokens.rs` (decode `tokenizer.bin` ids, U+2581 to spaces) with unit tests on known ids; verify with `cargo test --lib captions::tokens`.
- [x] 1.4 Add `captions/stream.rs`: the `streaming_config.json` serde struct, loading of the five sessions, incremental `feed(samples)` through frontend/encoder/adapter with fed-back state, `decode()` greedy with the KV caches and token caps, and `reset_line()`. Verify with an ignored test `transcribes_wav` (`SNAPR_CAPTIONS_MODEL`, `SNAPR_CAPTIONS_WAV`) that feeds a 16 kHz WAV in 160 ms steps and prints the text matching the speech.
- [x] 1.5 Add the 48 kHz stereo i16 → 16 kHz mono f32 resampler and the energy endpoint (`PAUSE` 700 ms, `MAX_LINE` 10 s, adaptive floor) in `captions/segment.rs`, with unit tests for a 1 kHz tone level, silence skipping, pause end and the length cap; verify with `cargo test --lib captions::segment`.

## 2. Model store

- [ ] 2.1 Add `captions.rs` shaped like `cutout.rs`: `CaptionLanguage`, the pinned `MODELS`, `model_dir`, `ready`, `installed`, `IN_USE` with `acquire`, `State.captions_download`, the download thread through `fetch::Fetch` and `onnx::ensure`, `captions-progress` / `captions-done` (constants in `events.rs`), and the commands `captions_status`, `captions_download`, `captions_cancel`, `captions_delete`. Register them in `lib.rs`. Verify by downloading, cancelling and deleting a language from the browser devtools with `invoke`.
- [x] 2.2 Make `onnx::release` and every other store's delete count the captions store as installed or busy; verify that deleting the last other model keeps `onnxruntime.dll` while a captions model is installed.
- [ ] 2.3 Add `lib/api.ts` wrappers, `hooks/use-captions.ts` (status, progress and done listeners, download / cancel / remove), `useCaptionsStore` in `models/stores.ts`, a `models/captions-card.tsx` (language ToggleGroup or Select, size, status line), the entries in `downloaded-models.tsx`, an About MIT notice once a model is ready, and the en/vi i18n strings. Verify with `bunx tsc --noEmit` and by downloading Vietnamese from the AI models page.

## 3. Session

- [x] 3.1 Add `captions_language` / `captions_target` to `config.rs` with serde defaults and `Config::captions_target(ui)` (`off`, else a valid code, else `default_target`); add `RecordMode::Captions`. Verify that an older config loads with English and the default target (`cargo test` on a config without the fields).
- [x] 3.2 Add the session in `captions/session.rs`: `start(app)` (audio through `panel::listen` with `record = true`, the recognizer thread emitting `live-caption`, and the translator thread with HY-MT loaded once and the backlog batched through `complete_batch` only when a distinct target is set and `translate::ready`), `stop(app)` (flag, drop capture, join, close windows), and `State.captions`. Verify with `SNAPR_CAPTIONS_LOG=1` in a debug build that the partial and final lines and the translations are printed while a YouTube video plays.
- [ ] 3.3 Wire the mic-lost callback to the panel warning, `hotkeys::trigger` (record hotkey stops a session), `panel::toggle` for `Phase::Captions`, and the tray Quit (stop before `app.exit`). Verify that each path ends the session and the microphone privacy indicator turns off.

## 4. Panel and captions window

- [x] 4.1 In `panel.rs`: add `Phase::Captions`; add `captions_language`, `captions_target`, `captions_ready` and `translate_ready` to `PanelState`; extend `record_select`; branch `record_start` on `RecordMode::Captions` (save the choices, `captions::start`, no picker); add the `open_models` command (`show_settings(app, "models")`). Verify with `cargo clippy`.
- [x] 4.2 Build the `captions` window (`view=captions`, 640×200, `focused(false)`, `WDA_EXCLUDEFROMCAPTURE`, `WS_EX_NOACTIVATE`) above the panel, and add it to `capabilities/default.json`. Verify that it does not take focus from Notepad while typing and is missing from a snapr screenshot.
- [ ] 4.3 Add `components/captions/` (`CaptionsApp`: lines by id, dimmed partial, translation below, follow-bottom unless scrolled up, Copy all, drag region) and route `?view=captions` in `main.tsx`. Verify with `bunx tsc --noEmit` and a live session.
- [x] 4.4 In `setup-panel.tsx`: the Captions mode entry; release the camera and show the language placeholder; disable the camera picker; Spoken language and Translate to dropdowns; Start disabled without an audio source or caption model; the missing-model notice with Open AI models; en/vi strings. In `recording-panel.tsx`: the captions bar (Stop, timer, mic-lost warning). Verify with `bunx tsc --noEmit` and by checking each spec scenario of the recording-panel delta.

## 5. Action ring

- [x] 5.1 Add `Wedge::Captions` to `ring.rs` (`WEDGES`, `ALL`, `action` → `Action::Captions`, a Stop live captions face while captioning) and `Action::Captions` to `lib.rs`. Handle it in `hotkeys::trigger` (stop a session, else `panel::open_with(.., Some(RecordMode::Captions))`, else switch the setup panel's mode), and filter `Snap::open_ring` (no Captions during a recording or countdown, no Record during a session). Verify with `cargo clippy` and by picking the item in each of the three states.
- [ ] 5.2 Add `BUILTINS.captions` (same glyph) to `lib/ring.ts`, the `BuiltinId` type in `lib/api.ts`, the en/vi `ringCaptions` strings, and make `setup-panel.tsx` follow a backend change of `state.mode`. Verify with `bunx tsc --noEmit`, check that a default ring shows 9 items, and check that a customised ring offers Live captions under the items to add.

## 6. Integration

- [ ] 6.1 Walk through every scenario in `specs/live-captions/spec.md` on a release build (`bun run tauri build`), including offline, screen share exclusion and Copy all, and note the measured line latency (target: under 1 s for the partial line on the CPU).
- [x] 6.2 Update CLAUDE.md (architecture: the ring item, the captions session, `captions.rs` / `captions/`, the panel mode, the window, the events and the commands) and verify that `openspec validate live-captions --strict` passes.
