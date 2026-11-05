# Proposal

## Why

Screenshots for bug reports and chats often carry emails, phone numbers, IPs, API keys or card numbers, and today the only way to hide them in a Draw capture is drawing filled rectangles by hand, which is slow and easy to miss. Snapr already has OCR on the capture side (Windows OCR and PaddleOCR), so it can find that text itself and cover it before the image ever leaves the overlay.

## What Changes

- A new Redact tool in the Draw toolbar's main row (after Text, key X). Dragging with it draws a redaction box; boxes are ordinary shapes: selectable, movable, deletable, undoable.
- A redaction box has a style: Black (solid, default) or Pixelate. The options row shows the two styles and a Detect button. Pixelate always uses blocks large enough that text cannot be read back.
- Detect runs OCR on the current selection in the background and adds one redaction box per match of: email address, phone number, payment card number (Luhn checked), IPv4 address, and secrets (well-known API key / token prefixes, JWTs, long high-entropy strings). The added boxes are one undo step; the user removes a wrong one by selecting it and pressing Delete.
- Draw capture never detects on its own: Detect runs only when the user presses it (or D), so the person drawing stays in control of what gets covered. Pressing Detect again rescans and skips matches already covered.
- While detection runs, the overlay stays fully usable and a small busy marker shows on the Detect button. A result that arrives after the overlay closed or the selection changed is dropped.
- Detect needs OCR (Windows OCR language or a ready PaddleOCR model); without it the button is dimmed and redaction stays manual.
- Saved, copied and pinned images contain the redactions burned in; the original pixels under a box are not recoverable from the output.
- Fullscreen and window captures can be redacted automatically: with "Automatically redact fullscreen and window captures" on (off by default), the save worker runs OCR on the shot, covers every match in the default style and only then saves, copies and previews it. The unredacted original is never written. Area captures and Draw captures are not redacted automatically.
- Redaction is configurable in Settings → Capture, in a Redaction card: automatic redaction of fullscreen and window captures (off by default), the default style (Black by default), one switch per detected category (all on by default) and a list of custom regex patterns (empty by default, validated before saving). The categories, patterns and style apply to both Draw's Detect and automatic redaction. A config without these settings loads with the defaults.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `draw-capture`: the main toolbar row gains the Redact tool, the options row gains redaction options, plus new requirements for redaction boxes, automatic detection, the detected categories and custom patterns, all following the redaction settings.
- `capture-output`: fullscreen and window captures are redacted before they are saved when automatic redaction is on.
- `settings-window`: the Capture page shows the Redaction settings card.

## Impact

- Backend: `draw.rs` (`Tool::Redact`, `Shape::Redact`, pixelate / black rendering from the frozen background), `toolbar.rs` (main row slot, options row, icons, tooltips en/vi, busy marker), `overlay.rs` (key X, detection request and result handling), `snap.rs` (detection thread and the result `Action`), `ocr.rs` / `ocr/paddle.rs` (a box-returning recognition next to the text one), a new `redact.rs` (pattern matching to boxes), `lib.rs` (`Action::Redactions`), `jobs.rs` (`Job::Save(shot, redact)` and `auto_redact`).
- Config: a new `[redact]` section in `config.rs` (`auto_capture`, `style`, `email`, `phone`, `card`, `ipv4`, `secrets`, `patterns`), with a `redact_validate(pattern)` command.
- Frontend: `components/settings/capture/` (a Redaction card), `lib/api.ts`, `lib/i18n.ts`.
- Dependencies: `regex`.
