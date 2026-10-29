# Proposal

## Why

Text capture only works on a live area of the screen at the moment the hotkey is pressed. Text that is already in a pin or in a saved screenshot has to be put back on screen and captured again, and a QR code in a selected area comes back as noise or nothing. The OCR path is already a self-contained image-to-text step, so offering it wherever snapr already holds an image is cheap and closes these gaps.

## What Changes

- Add a **Copy text** action to the pin hover toolbar that recognizes the text of the pin's full-resolution image and copies it.
- Add a **Copy text** action to the gallery item context menu (grid and viewer) for photo items that recognizes the text of the file and copies it.
- Make every text recognition (text hotkey, pin, gallery) read **QR codes** first: when the image contains one or more QR codes, their decoded contents are copied instead of the OCR text.
- All three entry points report through the existing text-capture notification, with a new "QR code copied" message.
- Copy text is offered only while Windows has an OCR language, independent of the text capture hotkey switch.
- Out of scope: searching the gallery by text, bulk Copy text from a selection, opening QR links, other barcode formats, OCR on videos.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `text-capture`: recognition also reads QR codes; text can be extracted from an existing image (pin or gallery photo) with the same result and notification rules.
- `screen-pin`: the hover toolbar gains Copy text next to Copy and Close.
- `gallery`: the item context menu gains Copy text for photo items, limited to the output folder like the other file actions.

## Impact

- Backend: `ocr.rs` / `read_text` gain a QR pass and the new message; `pin.rs` toolbar gets a third button; a new capture-thread action reads a gallery file into the text job; a `gallery_text` command in `gallery.rs`.
- Frontend: `api.galleryText`, a `copyText` action in `hooks/use-gallery.ts`, a menu item in `item-menu.tsx`, en/vi strings.
- New dependency: `rqrr` (pure Rust QR decoder).
- No config schema, hotkey or output-format change.
