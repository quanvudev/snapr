# Tasks

## 1. Dependencies and pinned sources

- [x] 1.1 Add `ort` (exact 2.0.0-rc pin, `default-features = false`, `load-dynamic` + `ndarray`), `ndarray`, `ureq` 3 (rustls), `sha2`, `zip` (deflate only) and `yaml-rust2` to `src-tauri/Cargo.toml`; verify `cargo clippy` in `src-tauri/` builds with no new warnings
- [x] 1.2 Look up the ONNX Runtime version the pinned `ort` rc requires and pick the matching-or-newer `onnxruntime` cp313 win_amd64 wheel on PyPI; record its URL, size and SHA-256; verify by downloading it and listing `onnxruntime/capi/onnxruntime.dll` inside
- [x] 1.3 For the six `PaddlePaddle/PP-OCRv6_{tiny,small,medium}_{det,rec}_onnx` repos, record the commit sha, and the LFS SHA-256 and byte size of each `inference.onnx` and the rec `inference.yml`, plus `thresh` / `box_thresh` / `unclip_ratio` from each det `inference.yml`; verify each pinned `resolve/<sha>/` URL returns the recorded size

## 2. Model store and download (`paddle.rs`)

- [x] 2.1 Add `PaddleSize` (tiny/small/medium) with the static file table from 1.2–1.3 and `dir(app)` = `app_local_data_dir()/paddle`; `ready(size)` checks `det.onnx`, `rec.onnx`, `dict.txt` and `onnxruntime.dll`; verify with clippy and by hand-creating/removing files under the folder and calling `ready` from a debug log
- [x] 2.2 Implement a streaming download to `<name>.part` with SHA-256 check and rename on success, `.part` removal on error or cancel, stale `.part` cleanup at start, and a cancel `AtomicBool` checked per chunk; verify by downloading Tiny and seeing only final files, and by cancelling mid-way and seeing no `.part` left
- [x] 2.3 Extract `onnxruntime.dll` from the wheel (only that entry) into `paddle/`, and convert the rec `inference.yml` `PostProcess.character_dict` into `dict.txt` (one entry per line, written last); verify `dict.txt` line count is 18708 for Small and 6904 for Tiny
- [x] 2.4 Add `State.download`, the download thread with `paddle-progress` (throttled ~10/s) and `paddle-done` events, a one-download-at-a-time guard, and `apply()` on finish; verify with `bun run tauri dev` that progress events arrive in the WebView devtools and a second start is refused
- [x] 2.5 Add commands `paddle_status`, `paddle_download`, `paddle_cancel`, `paddle_delete` (refuses a downloading size, removes the size dir, drops the cached engine for that size, removes `onnxruntime.dll` when no size is left, calls `apply()`), register them in `run`; verify each from devtools `invoke` and check the folder contents after delete

## 3. Config

- [x] 3.1 Add `OcrEngine` (`windows` default) and `PaddleSize` (`small` default) as lowercase serde enums with field-level `#[serde(default)]` on `Config.ocr_engine` / `Config.paddle_model`; verify an existing `config.toml` without them loads with Windows / Small and a saved config writes both fields

## 4. PaddleOCR engine

- [x] 4.1 Factor the row grouping out of `ocr::recognize` into `fn rows(Vec<(f32, f32, f32, String)>) -> String` used by the Windows path; verify Windows text capture output is unchanged on a two-line area and a spaced nav bar
- [x] 4.2 Implement the cached engine (`static ENGINE: Mutex<Option<(PaddleSize, Engine)>>`, `ort::init_from` with the absolute DLL path once, det + rec sessions, dict loaded with the trailing space entry); verify the first recognition loads and the second reuses the sessions (debug timing log)
- [x] 4.3 Implement detection: BGR normalize, resize to longer side ≤ 960 and multiples of 32, binarize at `thresh`, connected-component boxes, `box_thresh` filter, unclip padding, scale back; verify on a sample screenshot that box count and positions match the visible text lines
- [x] 4.4 Implement recognition: crop, resize to height 48 (width ≤ 3200), `(x/255-0.5)/0.5`, batches of 8, greedy CTC decode; feed `(x, top, bottom, text)` into `rows`; verify English, Simplified Chinese and Japanese sample screenshots give the expected text with Small
- [x] 4.5 Spike check: compare our output with the reference PaddleOCR Python pipeline (`engine="onnxruntime"`, same model) on 5 sample screenshots per size; verify line text matches for horizontal text, and note any gaps in this change's design.md

## 5. Engine switch and availability

- [x] 5.1 Change `ocr::available` / `ocr::extract` to take the engine and size (`windows_available() || paddle && ready`, and paddle-when-ready-else-windows); update `apply()`, `show_settings()`, `read_text` and `gallery_text` to pass them from `State.cfg`; drop the cached engine in `apply()` when the engine is Windows; verify `cargo clippy` and that with PaddleOCR + Small ready the text hotkey, pin Copy text and gallery Extract text all return PaddleOCR results
- [x] 5.2 Add `windows_ocr` to `Loaded`, keep `ocr_available` as overall availability; verify in devtools `load` output under both engines
- [x] 5.3 Verify the fallback scenarios: during a download recognition uses Windows OCR; after deleting the size in use the text hotkey still works through Windows; with no Windows OCR language (or `available` forced false in a debug build) text capture becomes available only once the selected size finishes downloading, without restart

## 6. Settings UI

- [x] 6.1 Add `lib/api.ts` types and wrappers (`OcrEngine`, `PaddleSize`, `PaddleStatus`, `paddleStatus`, `paddleDownload`, `paddleCancel`, `paddleDelete`, `windows_ocr` on `Loaded`) and en/vi strings in `lib/i18n.ts` (engine names, size names and descriptions with download sizes, "Does not read Vietnamese", status, Download/Cancel/Retry/Delete, delete confirmation); verify `bunx tsc --noEmit`
- [x] 6.2 Add `hooks/use-paddle.ts` (status, `paddle-progress` / `paddle-done` listeners, download/cancel/delete, reload on `settings-open`) and expose it through `useSettingsContext()`; when the engine becomes PaddleOCR or the size changes, save and then call `paddleDownload` if that size is not ready; verify `bunx tsc --noEmit`
- [x] 6.3 Add `ocr-engine-row.tsx` (ToggleGroup Windows | PaddleOCR v6) and `paddle-model-row.tsx` (ToggleGroup Tiny | Small | Medium, description + size + Vietnamese note, Progress + Cancel / error + Retry / Not downloaded + Download) to the Capture group; switch the "No OCR language installed" hint in `text-capture-row.tsx` to `windows_ocr` with the switch still gated on `ocr`; verify in `bun run tauri dev` that the PaddleOCR rows are hidden under Windows and the progress bar moves during a download
- [x] 6.4 Add `paddle-models-row.tsx` (Item per size with in use / downloaded / not downloaded Badge, disk size, Delete opening an AlertDialog naming the size and its size); verify deleting a non-active size, the active size (engine stays PaddleOCR, status shows Not downloaded) and the last size (`onnxruntime.dll` gone) in the running app
- [x] 6.5 Verify the settings-window scenarios end to end: first switch to PaddleOCR downloads Small with progress, switching to an already-downloaded size is instant, hiding the window mid-download still finishes, a forced hash mismatch (edit a pinned hash in a debug build) shows the error with Retry

## 7. Docs and integration

- [x] 7.1 Update `CLAUDE.md`: the `ocr.rs` engine switch and row grouping, the new `paddle.rs` module (pinned sources, layout, readiness rule, download thread, commands, events), the new config fields, `State.download`, `Loaded.windows_ocr` and the new Capture rows; verify the described names match the code
- [x] 7.2 Run `cargo clippy`, `bunx tsc --noEmit` and `bun run tauri build`; verify the installer size did not grow by more than the new crates' code (no bundled ONNX Runtime) and the release build downloads, recognizes and deletes models

## Workflow follow-up

- Archive the change once the review is done, so the `ocr-engine` spec is created and the `text-capture` and `settings-window` deltas are merged.
