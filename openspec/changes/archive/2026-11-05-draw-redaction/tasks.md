# Tasks

## 1. OCR with boxes

- [x] 1.1 `ocr.rs`: add `OcrLine { text, rect, spans }` and `words(img, paddle)` preferring Windows OCR when a language is available and falling back to the ready Paddle model (text capture keeps the selected engine)
- [x] 1.2 Windows path: build each line's text from its words with byte ranges and `BoundingRect`, scaled back to image pixels when downscaled
- [x] 1.3 Paddle path: expose the line rects and texts from `Engine::extract` as `OcrLine` with empty spans

## 2. Matching

- [x] 2.1 Add the `regex` dependency to `src-tauri/Cargo.toml`
- [x] 2.2 New `redact.rs`: patterns for email, phone, card (Luhn), IPv4 (octets, version guard) and secrets (prefixes, JWT, 32+ mixed alphanumerics)
- [x] 2.3 `redact::find(lines)`: map each match to a rect (word union, proportional cut for partial words and Paddle lines), pad 2 px, clip to the image

- [x] 2.4 `redact.rs`: `Rules::from(&config::Redact)` (enabled categories, compiled custom patterns, invalid and over-limit ones dropped), `Rules::is_empty`, and `find(lines, &Rules)` applying only enabled categories plus custom patterns
- [x] 2.5 `redact.rs`: `redact_validate(pattern)` command (`EMPTY`, `TOO_LONG`, regex error), registered in `lib.rs`

## 3. Shape and rendering

- [x] 3.1 `draw.rs`: `Tool::Redact`, `RedactStyle { Black, Pixelate }`, `Style.redact`, `Shape::Redact` with hit-testing over the whole box, move and restyle
- [x] 3.2 Render Black as an opaque fill and Pixelate from `Sketch.background` with `max(16, min(w, h) / 2)` blocks, in both `view()` and `finish()`
- [x] 3.3 Add `Sketch` support for adding a batch of shapes as one undo snapshot and for checking whether a rect is already covered by a redaction box

## 4. Toolbar and overlay

- [x] 4.1 `toolbar.rs`: Redact in `MAIN_ROW` after Text; `Button::Black`, `Pixelate`, `Detect`; `options_row(Redact)`; icons, en/vi tooltips and shortcuts; busy dot on Detect; include the row in placement width
- [x] 4.2 `overlay.rs`: keys X and D, `Overlay::open(.., detectable)`, Detect disabled without OCR, with empty rules or while running, redact style seeded from the setting at open and not carried back into `Snap.style`, `Annotation.detect` / `detected`
- [x] 4.3 `overlay.rs`: return `Outcome::Detect { token, image }` and add `Overlay::redactions(token, rects)` that drops stale results, skips covered matches and shows "Nothing found"

## 5. Detection thread

- [x] 5.1 `lib.rs`: `Action::Redactions { token, rects }`
- [x] 5.2 `snap.rs`: request counter, spawn the detection thread (WinRT MTA, `paddle::selected`, `Rules::from` read from `State.cfg` at start, `ocr::words`, `redact::find`) and route the result to the open overlay; pass `State.ocr` and the redaction settings to `Overlay::open`

## 6. Settings

- [x] 6.1 `config.rs`: `Redact` section (`auto_capture`, `style`, `email`, `phone`, `card`, `ipv4`, `secrets`, `patterns`) with field-level serde defaults, in `Config` and its `Default`, plus `Config::redact_style()`
- [x] 6.2 `lib/api.ts`: the `redact` config type and `redactValidate(pattern)`
- [x] 6.3 `capture/redaction-card.tsx`: auto-redaction Switch ("Automatically redact fullscreen and window captures"), Black / Pixelate ToggleGroup, five category Switches, OCR note linking to Text when `ocr_available` is false; added to `capture.tsx` below the padding card
- [x] 6.4 `capture/redact-patterns.tsx`: list with remove buttons, `Input` + Add, validation through `redact_validate` plus duplicate and 20-pattern checks, inline error
- [x] 6.5 `lib/i18n.ts`: en / vi strings for the card, categories, pattern errors and the overlay tooltips / "Nothing found"

## 6b. Automatic redaction of captures

- [x] 6.6 Drop Draw's detect-on-switch (`Detector.auto`, `Overlay::switched`) and rename the setting to `auto_capture` (off by default)
- [x] 6.7 `jobs.rs`: `Job::Save(shot, redact)` with `true` from fullscreen and window saves, `auto_redact` before padding (skipped without OCR or rules, OCR error saves unredacted with a tooltip), `draw::redact_image`

- [x] 6.8 Redaction prefers Windows OCR; `redact.engine` (`windows` / `paddle`) with `paddle::for_redaction` and `capture/redact-engine-row.tsx` forces PaddleOCR

## 7. Docs and checks

- [x] 7.1 Update `CLAUDE.md` (Draw capture: redact tool, detection flow, `redact.rs`, `ocr::words`, the `[redact]` config and the Capture Redaction card)
- [x] 7.2 `cargo clippy` and `bunx tsc --noEmit` pass
- [x] 7.3 Manual check in `bun run tauri dev` with Windows OCR and with PaddleOCR: in Draw, press Detect on an email, phone, IP, card and an `sk-` key; delete one false positive; undo the batch; save, copy and pin and confirm the boxes are burned in; press Enter while detection runs; confirm switching to the redact tool detects nothing by itself. Turn on automatic redaction and take a fullscreen and a window capture: the file, clipboard and toast are redacted and no unredacted file exists; an area capture is not redacted; turn a category off and add a custom pattern and confirm the next capture follows them
