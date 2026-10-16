# Design

## Context

See proposal.md for motivation. Relevant current state:

- Hotkeys are four fixed slots: `Hotkeys::combos() -> [String; 4]`, zipped in `apply()` with `[Fullscreen, Pick(Window), Pick(Area), Record]`. `settings::rows()` validates exactly four slots (invalid / needs_modifier / duplicate / in_use) and `save` refuses any row with an error. The frontend mirrors this with `SLOTS` in `lib/hotkeys.ts`.
- `Config` and `Hotkeys` use struct-level `#[serde(default)]`, so a missing field takes its value from `Default`.
- The area picker yields `Shot::Ready(img)`. `Snap.record_pick: bool` diverts a picked window to the recorder; everything else goes to the save worker through `Sender<Shot>`.
- The save worker (`work()`) runs on its own thread, one job at a time.
- `windows` 0.62 is already in `Cargo.lock` through `windows-capture`.
- The preview toast draws only an image and opens a file on click; it cannot show a text message.

## Goals / Non-Goals

**Goals:**
- OCR through `Windows.Media.Ocr` only, off the winit loop.
- Keep the hotkey pipeline a fixed array (five slots), not a dynamic list.

**Non-Goals:**
- Language picker, OCR on window/fullscreen shots, OCR on an existing toast image, text layout preservation beyond one line per recognized line.
- Replacing the tray tooltip for screenshot/recording status with notifications.

## Decisions

### 1. Engine: WinRT `Windows.Media.Ocr` via the `windows` crate
New `ocr.rs` with two functions:
- `available() -> bool`: `OcrEngine::TryCreateFromUserProfileLanguages()` succeeds.
- `extract(&RgbaImage) -> Result<String>`: RGBA → BGRA bytes → `CryptographicBuffer::CreateFromByteArray` → `SoftwareBitmap::Create(Bgra8)` + `CopyFromBuffer` → `engine.RecognizeAsync(&bitmap)?.join()` → lines grouped into visual rows (a line joins the row its word-box vertical center falls in), each row joined left to right with a space, rows joined with `\n`. Windows reports widely spaced items on one row as separate lines, so its line list alone is not the visual layout.

Images whose longest side exceeds `OcrEngine::MaxImageDimension()` are downscaled first with `image::imageops::resize` (already a dependency). The worker thread initializes WinRT once (`RoInitialize(RO_INIT_MULTITHREADED)`) so the blocking `.get()` is allowed.

Use the same `windows` version already in the lock (0.62) with features `Media_Ocr`, `Graphics_Imaging`, `Security_Cryptography`, `Storage_Streams`, `Foundation`, `Foundation_Collections`, `Win32_System_WinRT` so no second copy of the crate is built.

Alternatives: PaddleOCR via ONNX (rejected in exploration: 25–35 MB of runtime and models, download management); `CryptographicBuffer` vs `DataWriter` for the `IBuffer` — `CreateFromByteArray` is one call.

### 2. Five hotkey slots, text before record
`Hotkeys` gains `text: String`; order becomes `[fullscreen, window, area, text, record]` so defaults read `1..5`. `combos()` returns `[String; 5]`; an empty key yields `""` instead of `"<mods>+"`. `apply()` actions gain `Action::Text`.

The `text` field carries a field-level `#[serde(default)]` (empty string) so a file without it stays unbound instead of inheriting `Hotkeys::default().text = "4"`, which would collide with an existing `record = "4"`. Fresh configs still get `4` from `Hotkeys::default()`.

### 3. One active flag drives registration and validation
`active = cfg.text_capture && State.ocr`, where `State.ocr: AtomicBool` caches `ocr::available()`.
- `apply()` re-probes `ocr::available()`, stores it in `State.ocr`, and skips registering the text slot when inactive or empty.
- `settings::rows(app, hotkeys, text_active)`: an inactive or empty slot gets no error and `in_use = false`, and is excluded from duplicate matching. `check` takes `text: bool` from the frontend (computed there as `text_capture && ocr_available`); `save` computes it from the config being saved and `State.ocr`.
- `show_settings()` re-probes; if the result differs from `State.ocr`, it calls `apply()`. This covers "language installed while running" without polling.
- `load` returns `ocr_available` so the UI can disable the switch and the text row.

Alternative: register the text hotkey whenever enabled and check availability at press time. Rejected because the agreed behavior is that the hotkey is not registered when unavailable.

### 4. Picker mode instead of a second bool
Replace `Snap.record_pick: bool` with `mode: Mode { Shot, Record, Text }`. `Action::Text` opens `Pick::Area` with `Mode::Text`. The worker channel changes from `Sender<Shot>` to `Sender<Job>` with `Job::Save(Shot)` and `Job::Text(Shot)`. Two mutually exclusive bools would allow an invalid state.

### 5. OCR result path in the worker
`Job::Text(Shot::Ready(img))` → `ocr::extract` (no padding, no save, no toast):
- non-empty text → `arboard` `set_text`, notify "copied N lines";
- empty text → notify "no text found", clipboard untouched;
- error → notify the error.

The clipboard is written regardless of `copy_to_clipboard`, since copying is the feature itself.

### 6. Notifications via `tauri-plugin-notification`, called from Rust
The plugin is used only from the backend (`app.notification().builder()`), so no webview capability is added. The three message strings live in Rust as a small `en`/`vi` match on `cfg.language`, since `T` in `i18n.ts` is frontend-only.

Alternatives: drawing text in the softbuffer toast (needs a font rasterizer and a "no file" toast mode); tray tooltip (easy to miss).

### 7. Opening Windows language settings
New command `open_language_settings` spawning `explorer ms-settings:regionlanguage`, matching how `open_folder`/`open_config_file` spawn processes. This avoids widening the `opener` URL scope for the `ms-settings:` scheme.

### 8. Frontend
- `api.ts`: `Hotkeys.text`, `Config.text_capture`, `Loaded.ocr_available`, `check(hotkeys, text)`, `openLanguageSettings()`.
- `hotkeys.ts`: `SLOTS = ["fullscreen", "window", "area", "text", "record"]`.
- `settings/capture/text-capture-row.tsx`: `Switch` bound to `text_capture`, disabled with hint + `Button` when `!ocr_available`.
- Hotkey row: disabled and error-free for the `text` slot while inactive (used by both settings and setup).
- `i18n.ts`: labels for the switch, the hint, the action and the text hotkey (en / vi).

## Risks / Trade-offs

- [Notifications show as "PowerShell" in `tauri dev`] → Expected for unpackaged runs; verify the title in an installed `tauri build`.
- [Focus Assist / notifications disabled hides the result] → Text is still copied; acceptable.
- [OCR accuracy on small UI fonts or Vietnamese diacritics is unknown] → Verify on real screenshots during implementation; upscaling tiny selections is a possible later tweak, out of scope here.
- [Large selections exceed `MaxImageDimension`] → Downscale before recognition.
- [Default record moves from `4` to `5`] → Only affects fresh configs; existing files keep their record key.

## Migration Plan

No data migration. Existing `config.toml` files load unchanged: missing `text` → empty (unbound), missing `text_capture` → `true`. Rollback is removing the feature; extra keys in the file are ignored by the old build because of `#[serde(default)]` and lenient TOML parsing.
