# Design

## Context

The Draw overlay lives on the winit capture thread (`overlay.rs`, `Annotation { sketch, toolbar, .. }`). Shapes are a closed enum in `draw.rs` rendered with tiny-skia over `Sketch.background` (the unannotated crop of the frozen frame), with `view()` for the live preview and `finish()` for the output. `Sketch::reselect` re-crops the background and translates shapes when the selection changes.

OCR exists only as text: `ocr::extract(img, paddle) -> String`. Windows OCR (`recognize`) already walks `Lines()` / `Words()` with `BoundingRect` but throws the rects away after row grouping; PaddleOCR (`Engine::extract`) returns axis-aligned line boxes (`Line`) with their text. WinRT OCR needs an MTA thread; the save worker (`jobs.rs`) initialises one but is serial with saves.

## Goals / Non-Goals

**Goals:** a redaction shape, one-click detection of common sensitive text inside the Draw overlay, nothing leaving the process.

**Non-Goals:** face or image detection, automatic redaction of area captures, redaction of gallery images or recordings, recordings), a setting to auto-redact every screenshot, importing or sharing pattern lists, blur (Gaussian blur of text can be partly reversed and costs more; Pixelate with big blocks is the soft option).

## Decisions

### OCR with boxes

Add `ocr::words(img, paddle) -> Result<Vec<OcrLine>, String>` beside `extract`, with `OcrLine { text, rect, spans: Vec<(Range<usize>, Rect)> }` in image pixels:
- Windows: each line's words are joined with single spaces; each word records its byte range and `BoundingRect`, scaled back when the image was downscaled for `MaxImageDimension`.
- Paddle: `Engine::extract` already yields line rects and text; spans are empty and the line rect is split by character position.

Engine: Windows OCR first whenever a Windows OCR language exists, whatever `ocr_engine` says; the ready Paddle model only when Windows OCR is unavailable or errors. Measured on a 1920x1080 screen in a release build: Windows OCR 0.06 s, Paddle Small 0.75 s, Paddle Medium 3.3 s, while matching and painting take under 5 ms, so following `ocr_engine` made every automatically redacted capture wait seconds. Windows OCR also gives exact word boxes, and the redacted categories are Latin script, which any Windows OCR language reads. Text capture and the gallery keep `extract` with the selected engine. For users who want PaddleOCR's accuracy anyway, `redact.engine` (`windows` default / `paddle`) forces the ready Paddle model of the selected size through `paddle::for_redaction`, independent of `ocr_engine`, with Windows OCR as the fallback on error; the setting row says it adds seconds per capture. The `rows` grouping is not used: matching works per OCR line.

### Matching (`redact.rs`)

`redact::find(lines) -> Vec<Rect>` runs a fixed list of `regex` patterns per line (new dependency `regex`), with post-filters: Luhn for cards, octet range for IPv4, letter-and-digit mix for the 32+ generic secret, a version guard (`v` prefix or more than four dot groups) for IPv4. For each match, the box is the union of the word rects whose ranges overlap the match; a word only partly covered (e.g. `KEY=sk-…`) is cut horizontally in proportion to character offsets inside it. Paddle lines are cut the same way. Boxes get 2 px padding, are clipped to the image and translated to overlay coordinates by the caller.

Keeping the patterns in one module with plain unit-testable functions makes the categories easy to extend later (there are no tests in the repo today; this module is the natural first one but tests are not required by this change).

### Settings

`config.rs` gets `Config.redact: Redact` with `#[serde(default)]` on the field and on each member, so a missing section or a missing key loads its default: `auto_capture: false`, `style: String` (`"black"` / `"pixelate"`, read only through `Config::redact_style() -> RedactStyle`, which maps an unknown value to Black, like `quality()`), `email`, `phone`, `card`, `ipv4`, `secrets: true`, `patterns: Vec<String>` (empty).

`redact::find(lines, &Rules)` takes the enabled categories and the custom patterns. `Rules::from(&Redact)` compiles the custom patterns with `regex::Regex::new`, silently dropping invalid ones and anything past the 20 / 200 limits (the config can be hand-edited). Custom matches use the same span-to-rect mapping as built-in ones. `Rules::is_empty()` (no category on and no compiled pattern) dims Detect.

Read timing: `Snap::pick` reads the style and `Rules::is_empty()` once per overlay (they shape the toolbar); the detection thread and the save worker read `State.cfg` again when they start, so a pattern added while the overlay is open applies to the next Detect or capture. The default style seeds the overlay's redact style at each open, unlike the other `Snap.style` fields, which last until quit.

The `redact_validate(pattern) -> Result<(), String>` command (in `redact.rs`) checks empty, length, and `Regex::new`, returning `EMPTY`, `TOO_LONG` or the regex error text; the frontend maps the first two to i18n and shows the third as is. Duplicates and the 20 limit are checked on the frontend, which owns the list; `save` stores whatever list it gets and `Rules` re-enforces the limits.

Frontend: `capture/redaction-card.tsx` (a `SettingsCard` with the "Automatically redact fullscreen and window captures" Switch for `auto_capture`, a Black / Pixelate ToggleGroup, five Switch rows and `capture/redact-patterns.tsx`: the list with remove Buttons, an `Input` and Add, the inline error under the input). It is always shown on the Capture page, below the padding card, whether or not OCR is available (a note says detection needs OCR and links to the Text page when `ocr_available` is false).

### Background detection

`Annotation` gets `detect: Option<u64>` (the running request) and `detected: Option<(i32, i32, u32, u32)>` (the selection last scanned). `Overlay` asks for a detection by returning a new `Outcome::Detect { token, image }` handled in `snap.rs`, which spawns a short-lived thread (WinRT MTA init, `paddle::selected` read from `State.cfg`, `ocr::words`, `redact::find`) and sends `Action::Redactions { token, rects }` through the proxy. `Snap` passes it to `Overlay::redactions(token, rects)`, which ignores it unless `token` equals the running request and the selection is unchanged since the request (`Annotation.detected`); otherwise it adds the boxes as one `snapshot()`. The token is a counter in `Snap`, so a result for a closed overlay can never match.

A dedicated thread per request (not the save worker) keeps detection from waiting behind a PNG encode and vice versa; requests are at most one at a time per overlay. Paddle's `ENGINE` mutex already serialises engine use with the save worker's text capture.

Availability: `Overlay::open` gets `detectable: bool` (`State.ocr` and non-empty rules); when false Detect is disabled (`overlay::enabled`). Draw never runs Detect on its own: an early version detected on the first switch to the redact tool, which was dropped because the person drawing can add boxes themselves and automation fits captures with no editing step.

### Automatic redaction of fullscreen and window captures

`Config.redact.auto_capture` (off by default) turns it on. `Job::Save` becomes `Save(Shot, bool)`: `Action::Fullscreen` and a `Shot::Window` save pass `true`, area, Draw and record picks `false`, so the choice is made where the capture kind is known rather than inferred from the `Shot` variant (fullscreen and area are both `Shot::Ready`). The save worker (already WinRT MTA) calls `jobs::auto_redact` after the window capture and before padding, so padding backgrounds are never scanned and the file, clipboard and toast get the same redacted image: it skips while `State.ocr` is false or the rules are empty, runs `ocr::words` with the selected engine and `redact::find`, then `draw::redact_image(img, rects, style)` burns the boxes in with the same Black / Pixelate rendering as Draw. An OCR error saves the shot unredacted and puts `Snapr: redaction failed: …` in the tooltip: losing the screenshot is worse, and the user is told. The worker is serial, so a capture waits for its OCR (a fraction of a second on Windows OCR, more on the first Paddle run) and later captures queue behind it as they already do.

### Shape and rendering

`Tool::Redact`, `Shape::Redact(Point, Point, RedactStyle)` with `RedactStyle { Black, Pixelate }` (shared with `config.rs`) stored in `draw::Style.redact`, seeded from the setting at each overlay open (see Settings). Rendering:
- Black: an opaque black rect fill.
- Pixelate: computed from `Sketch.background` (not from shapes under it), block size `max(16, min(box_w, box_h) / 2)` physical px (so a one-line box is at most two blocks tall), each block filled with its average colour; blocks are aligned to the box origin.

Because both styles paint only from the background and are opaque, the output never contains the original pixels. Hit-testing treats the whole box as inside (like filled shapes), so a press inside selects and moves it.

### Toolbar

`MAIN_ROW` gets `Slot::Control(Button::Tool(Tool::Redact))` after Text; the placement code's width computation includes the new options row. `options_row(Redact, ..)` is `[Button::Black, Button::Pixelate, Divider, Button::Detect]`. Icons from Segoe Fluent Icons (a filled block for Black, a grid for Pixelate, a search/scan glyph for Detect), chosen at implementation; the Detect button draws a small busy dot while `detect` is `Some`. "Nothing found" uses `toolbar::tag` under the Detect button for 1.5 s.

Keys: X selects the tool, D runs Detect (both skipped while a text box is open).

## Risks / Trade-offs

- OCR misses text (small fonts, low contrast, unsupported language) → detection is an aid, not a guarantee; manual boxes stay first-class and the spec never promises completeness.
- False positives (long numbers that are not phones, hashes as secrets) → one Delete per box, one Ctrl+Z for the whole batch, and the category can be turned off in Settings.
- A custom pattern can be slow (huge alternations) → Rust `regex` guarantees linear-time matching, and the 200 character / 20 pattern limits keep compile cost small.
- PaddleOCR line boxes make partial matches approximate → 2 px padding plus proportional cuts; Windows OCR word rects are exact.
- PaddleOCR saturating the CPU → ONNX Runtime defaults to one busy-spinning intra-op thread per core, so back-to-back redacted captures with Paddle Medium froze other apps; the Paddle sessions use half the logical cores with spinning off, measured at the same speed (Small 0.80 s, Medium 3.38 s on 16 cores).
- First Paddle detection loads the model (about a second) → busy marker; later detections reuse the cached engine.
- Pixelation with small blocks can be reversed → the block size (half the box's shorter side, at least 16 px) makes text unreadable; Black stays the default.
