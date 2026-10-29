# Design

## Context

Text recognition is one function today: the area picker in `Mode::Text` sends `Job::Text(Shot::Ready(img))` to the save worker, and `read_text(app, shot)` runs `ocr::extract`, writes the clipboard with `arboard` and shows the notification. The worker thread already has WinRT initialized MTA, which `Windows.Media.Ocr` needs. Pins keep their original full-resolution `RgbaImage` (used by Copy), and gallery commands validate paths with `guard(app, path)` and reach the capture thread through `State.proxy`. `State.ocr` caches `ocr::available()`.

## Goals / Non-Goals

**Goals:**
- One recognition path for every entry point, so the clipboard, QR and notification rules from the specs cannot drift between hotkey, pin and gallery.
- No OCR work on the main (Tauri) thread or the winit thread.

**Non-Goals:**
- A text index, search, or caching recognition results.
- Editing recognized text or saving it to a file.

## Decisions

### Pins feed `Job::Text`; the gallery returns the result

Pin Copy text sends `Job::Text(Shot::Ready(image.clone()))` from the capture thread (it already owns `jobs`), so pins get the hotkey's clipboard and notification behavior.

The gallery shows the result instead (revised during apply, at the user's request): `gallery_text` is an async command that runs `guard` + `Kind::Photo`, then on `tauri::async_runtime::spawn_blocking` calls `RoInitialize(RO_INIT_MULTITHREADED)`, decodes with `image::open(..).to_rgba8()`, runs `ocr::qr` and `ocr::extract` and returns `Recognized { qr, text }`. Unlike `read_text`, it returns both, since the pane can show QR codes and the caption together; an OCR error is only an error when no QR code decoded. Nothing touches the clipboard; the pane's Copy all uses `navigator.clipboard`.

Alternative (the first version): send `Action::ReadText` to the capture thread and reuse `Job::Text`. Replaced because the user wants to read and select the text in place, not get it blindly on the clipboard.

### Viewer text pane

`use-gallery` keeps `textOpen` and a `Map<path, { result?, error? }>`. The pane shows for the viewed item when `textOpen` and the item is a photo; an effect fetches a path the first time it is shown, so Left/Right and the strip follow the pane and a seen photo is not recognized again. Closing the viewer clears `textOpen`; Escape closes the pane first. The grid's Extract text menu item opens the viewer on that item with `textOpen` set. `text-pane.tsx` is a fixed-width (`w-80`) aside next to the image: header with Copy all and close, a `ScrollArea` with `select-text` paragraphs (`whitespace-pre-wrap`), QR codes first with a `Badge`. Read-only by construction: it renders text, not an input.

### QR pass lives in `read_text`, before `ocr::extract`

A new `qr::decode(&RgbaImage) -> Vec<String>` (or a function in `ocr.rs`) converts to luma, runs `rqrr::PreparedImage::prepare`, `detect_grids`, decodes each grid, drops failures, and sorts by the grid's top edge. `read_text` copies `codes.join("\n")` and notifies "QR code copied" / "Đã sao chép mã QR" (with a count when more than one) when the list is non-empty; otherwise it falls through to OCR unchanged. Because the pass sits in `read_text`, the text hotkey, pin and gallery all get it.

`rqrr` over `bardecoder` / `zxing-cpp`: pure Rust, no C toolchain, small, and only QR is in scope. Large images (a 4K gallery file) are downscaled before `prepare` so the longer side is at most ~2000 px, which keeps decode time bounded; a QR code that only survives at full resolution is an accepted miss.

QR runs even though the text hotkey is gated on OCR availability. That gate stays as is: the hotkey, pin and gallery actions all require `State.ocr`, so there is no QR-only mode.

### Availability

- Pin: `Pin::open` takes an `ocr: bool` read from `State.ocr` when the pin is created; the toolbar lays out three buttons when true and today's two otherwise. Copy text has no success flash; the notification is the feedback.
- Gallery: `load` already returns `ocr_available`; `item-menu.tsx` shows Extract text and the viewer shows the ScanText toggle for photos when it is true. The settings window re-probes OCR on open (`show_settings`), so the menu follows a newly installed language after reopening.

### Toolbar layout

The pin toolbar becomes `[Copy] [Copy text] [Close]`; the minimum window footprint grows by one button so the 16x16 case keeps every action reachable. Glyph: a text glyph from Segoe Fluent Icons (candidate `E8D2` "Font"), confirmed visually at apply time.

## Risks / Trade-offs

- [A pin created before an OCR language is installed has no Copy text until it is re-pinned] → accepted; pins are short-lived and the flag is read once per pin.
- [QR wins over text when both are present, which may surprise a user who wanted the caption] → matches the most common intent (scan the code); the spec states it explicitly. A user can select only the caption.
- [`rqrr` misses low-contrast or very small codes] → falls through to OCR, so the user still gets something; no extra failure state.
- [OCR on a large gallery file is slow] → it runs on a blocking pool thread and the pane shows a spinner; `ocr::extract` already downscales above `MaxImageDimension`.
- [Results are cached by path for the life of the hook] → a file overwritten in place under the same name shows stale text until the window reloads; snapr never overwrites captures.
