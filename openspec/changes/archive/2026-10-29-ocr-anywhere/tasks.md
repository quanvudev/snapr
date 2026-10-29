# Tasks

## 1. QR pass in text recognition

- [x] 1.1 Add `rqrr` to `src-tauri/Cargo.toml` and verify `cargo clippy` in `src-tauri/` builds with it.
- [x] 1.2 Add a QR decode function (luma conversion, downscale to ~2000 px on the longer side, detect, decode, sort by top edge, drop failures); verify `cargo clippy`.
- [x] 1.3 Call it first in `read_text`: on a non-empty result copy the joined contents and show the en/vi "QR code copied" notification, otherwise run `ocr::extract` unchanged; verify with the text hotkey on a QR code (one and two codes), a QR code with a caption, and plain text that the clipboard and notification match `specs/text-capture/spec.md`.

## 2. Copy text on pins

- [x] 2.1 Pass `State.ocr` into `Pin::open` and lay out a third toolbar button (`Button::Text`, `Outcome::Text`) between Copy and Close when it is true, growing the minimum footprint; verify `cargo clippy` and that a 16x16 pin shows all three buttons.
- [x] 2.2 Handle `pin::Outcome::Text` in `Snap` by sending `Job::Text(Shot::Ready(..))` with the pin's original image; verify Copy text on a scaled-down pin of a full screenshot copies the text, shows the notification and keeps the pin open.
- [x] 2.3 Verify with OCR unavailable (or by forcing the flag false) that pins show only Copy and Close, and that Copy text works with the text capture switch turned off.

## 3. Copy text in the gallery

- [x] 3.1 Make `gallery_text` an async command (`guard` + `Kind::Photo`, `spawn_blocking` with MTA init, decode, `ocr::qr` + `ocr::extract`) returning `{ qr, text }`, and remove `Action::ReadText`; verify `cargo clippy`.
- [x] 3.2 Add the typed `api.galleryText`, `textOpen` + per-path results and `extractText` / `toggleText` / `closeText` in `hooks/use-gallery.ts`, an Extract text item for photos in `item-menu.tsx` and a ScanText toggle in `viewer.tsx` (both only when `ocr_available`), `text-pane.tsx`, and en/vi strings; verify `bunx tsc --noEmit`.
- [x] 3.3 Manually verify Extract text from a grid tile and the viewer toggle on a PNG with text and one with a QR code plus caption: spinner, then content; text selectable but not editable; Copy all; the pane follows Left/Right, hides on videos, does not re-run on a seen photo, and Escape closes it before the viewer; MP4 and other files have no Extract text; `__TAURI_INTERNALS__.invoke("gallery_text", ...)` refuses an outside path and a video.

## 4. Integration

- [x] 4.1 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`, then a `bun run tauri dev` pass over every scenario in this change's specs; record any gap.
- [x] 4.2 Update `CLAUDE.md` (text capture: QR pass and the pin/gallery entry points; pin toolbar; gallery commands list) and verify it matches the implemented behavior.

## Workflow follow-up

- Run `openspec validate ocr-anywhere --strict` before review.
- Archive the change after the implementation is reviewed.
