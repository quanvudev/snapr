# Tasks

## 1. Dependencies

- [x] 1.1 Add `windows = "0.62"` (features `Media_Ocr`, `Graphics_Imaging`, `Security_Cryptography`, `Storage_Streams`, `Foundation`, `Foundation_Collections`, `Win32_System_WinRT`) and `tauri-plugin-notification` to `src-tauri/Cargo.toml`, register the notification plugin in the builder; verify `cargo tree -i windows@0.62` shows a single 0.62 copy and `cargo clippy` passes

## 2. OCR module

- [x] 2.1 Create `src-tauri/src/ocr.rs` with `available()` and `extract(&RgbaImage) -> Result<String, String>` (BGRA buffer, `SoftwareBitmap`, downscale above `MaxImageDimension`, lines joined with `\n`) per design §1; verify `cargo clippy` passes

## 3. Config and hotkey slots

- [x] 3.1 In `config.rs` add `Hotkeys.text` (field-level `#[serde(default)]`), `Config.text_capture` (default `true`), defaults text `4` / record `5`, and `combos() -> [String; 5]` in order fullscreen, window, area, text, record with empty key → `""`; verify a `config.toml` with `record = "4"` and no `text` loads with text empty and record `4`
- [x] 3.2 In `settings.rs` make `rows` five slots taking `text_active`; inactive or empty text slot has no error, `in_use = false`, and is excluded from duplicate matching; `check` takes `text: bool`, `save` derives it from the saved config and `State.ocr`; `load` returns `ocr_available`; add `open_language_settings` (spawns `explorer ms-settings:regionlanguage`) and register it; verify `cargo clippy` passes

## 4. Capture flow

- [x] 4.1 In `lib.rs` add `State.ocr: AtomicBool`, `Action::Text`, and in `apply()` re-probe `ocr::available()` and skip registering the text slot when inactive or empty; in `show_settings()` re-probe and call `apply()` when availability changed; verify with no OCR language the text hotkey is not registered
- [x] 4.2 Replace `record_pick` with `Mode { Shot, Record, Text }` and the worker channel with `Job { Save(Shot), Text(Shot) }`; `Action::Text` opens the area picker in `Mode::Text`; verify fullscreen, window, area and record behave as before
- [x] 4.3 In the worker call `RoInitialize(RO_INIT_MULTITHREADED)` once and handle `Job::Text`: extract, copy with `arboard` `set_text` when non-empty, and send a notification (copied N lines / no text found / error) with en/vi strings from `cfg.language`; verify extracting an area copies its text, an empty area leaves the clipboard unchanged and shows "no text found", and no PNG is written

## 5. Settings UI

- [x] 5.1 Update `lib/api.ts` (`Hotkeys.text`, `Config.text_capture`, `Loaded.ocr_available`, `check(hotkeys, text)`, `openLanguageSettings`) and `SLOTS` in `lib/hotkeys.ts`; pass `text_capture && ocr_available` to `check` in `use-settings.ts`; verify `bunx tsc --noEmit` passes
- [x] 5.2 Add `components/settings/capture/text-capture-row.tsx` (shadcn `Switch`, disabled with hint and a `Button` calling `openLanguageSettings` when unavailable) and include it in `capture.tsx`; add en/vi strings to `lib/i18n.ts`; verify the switch toggles and saves, and is disabled with the hint when OCR is unavailable
- [x] 5.3 Make the hotkey row render the `text` slot disabled and error-free while inactive, in both settings and setup; verify five rows show with defaults 1–5 on setup and turning text capture off clears a duplicate between text and another slot

## 6. Docs and integration

- [x] 6.1 Update `CLAUDE.md` (five hotkey slots with text before record, `ocr.rs`, `Mode`/`Job`, notifications, new command) and verify it matches the code
- [x] 6.2 Run `bun run tauri dev` and check end to end: extract text from an area, cancel the picker, extract during a recording, turn the switch off and on, and launch with an old config containing `record = "4"`; verify each matches the scenarios in `specs/`
