# Design

## Context

See proposal.md for motivation. Facts that shape the approach:

- Captures are written flat into `Config::dir()` as `unique_path(dir, "png" | "mp4")`. No index or metadata store exists.
- The `main` webview cannot read local files today: the `protocol-asset` feature is off and there is no file scope in `capabilities/default.json`.
- `show_settings(app)` sizes, shows and emits `settings-open` with no payload; `use-settings.ts` resets `section` to `"general"` on that event.
- Escape is handled globally in `use-hotkey-recorder.ts` (window `keydown` → `api.hide()`).
- The settings pane is a `ScrollArea` wrapping a `max-w-3xl` column in `settings.tsx`.
- Saves reach the capture thread as `Action::Saved(img, path)` and `Action::Recorded(img, path)` in `Snap::user_event`, which holds an `AppHandle`.
- Reusable pieces: `capture::copy(&RgbaImage)` (arboard), the `image` crate with PNG, the `windows` 0.62 and `windows-sys` 0.61 crates, `tauri-plugin-opener`, and the shadcn `context-menu`, `alert-dialog`, `toggle-group`, `empty` and `scroll-area` components.

## Goals / Non-Goals

**Goals:**
- One `read_dir` per list request; no per-file IPC round trips.
- Every file access from the webview passes one path guard bound to the current output folder.
- Video seeking works in the viewer.

**Non-Goals:**
- Own thumbnail cache on disk (the Windows thumbnail cache already persists them).
- Grid virtualization.
- Watching the folder for external changes (reload on open covers it).

## Decisions

### 1. Backend module `gallery.rs`

A single module owns everything that touches gallery files:

- `list(dir) -> Vec<Item>` where `Item { path, date_ms, kind: Photo | Video }`; `date_ms = min(created, modified)` from `DirEntry::metadata()`. On Windows that metadata comes from the directory enumeration itself, so listing stays one syscall batch. Sorting newest-first happens here, so the frontend only groups.
- `guard(app, path) -> Option<PathBuf>`: `canonicalize` the request and `Config::dir()`, accept only when the parent of the request equals the canonical dir and the extension is png or mp4. Every command and both protocols call it. A flat folder makes "parent equals dir" stricter and simpler than `starts_with`.
- Commands: `gallery_list`, `gallery_open` (opener `open_path`, Rust side, so JS needs no `opener:allow-open-path` scope), `gallery_reveal` (`explorer /select,<path>`, matching the existing `open_folder` style), `gallery_copy` (decode PNG with `image`, reuse `capture::copy`), `gallery_delete`.

Alternative considered: putting these in `settings.rs`. Rejected; that module is about config and autostart, and the gallery has its own Win32 surface.

### 2. Thumbnails: Windows Shell through a `thumb` URI scheme

`register_asynchronous_uri_scheme_protocol("thumb", …)`. The frontend builds URLs with `convertFileSrc(path, "thumb")` (on Windows `http://thumb.localhost/<encoded path>`), so no URL helper is written by hand.

The handler guards the path, then hands `(path, responder)` to a small pool of `THUMB_WORKERS` threads that share one `Mutex<Receiver>`. Each thread calls `CoInitializeEx(COINIT_APARTMENTTHREADED)` once, then per job:
`SHCreateItemFromParsingName` → `IShellItemImageFactory::GetImage(THUMB_SIZE, SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK)` → `GetDIBits` into a top-down 32-bit BGRA buffer → swap to RGBA into an `RgbaImage` → PNG bytes. PNG keeps the alpha of transparent padding. The response carries `Cache-Control: max-age=…` so the webview does not ask again while the window lives. Any failure returns 404 and the `<img onError>` shows a placeholder.

Why a pool rather than `spawn_blocking`: Shell thumbnail extractors need an STA COM apartment, and initializing COM per call on a shared runtime thread is wasteful and can conflict with whatever apartment that thread already has. A fixed pool also caps parallel video decodes when a screen of video thumbnails loads at once.

Alternatives considered: decoding originals in the webview (heavy with large PNGs, unreliable first frame for video), or our own resize + cache (no thumbnails for videos not recorded by snapr). Rejected per exploration.

### 3. Full media through a `media` URI scheme with Range, not the asset protocol

The viewer loads `convertFileSrc(path, "media")`. The handler guards the path, then serves the file: with no `Range` header, the whole file with `200`; with `bytes=start-end` (or `start-`), that slice with `206`, `Content-Range` and `Accept-Ranges: bytes`. Content type comes from the extension (`image/png`, `video/mp4`). Serving runs in `tauri::async_runtime::spawn_blocking`.

Why not the built-in asset protocol: its scope can only grow (`allow_directory`) or permanently forbid, so after the user changes the output folder the old folder stays readable or can never be re-allowed. Routing through the same `guard` as everything else keeps one rule, the current output folder, and needs no `tauri.conf.json` or capability change. The cost is about thirty lines of Range parsing.

### 4. Delete to Recycle Bin

`SHFileOperationW` with `FO_DELETE` and `FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI` on a double-NUL-terminated wide path (via `windows-sys`, feature `Win32_UI_Shell`). A non-zero return or `fAnyOperationsAborted` becomes an `Err(String)`. Confirmation is the frontend `AlertDialog`; the Shell's own dialog is suppressed so there is exactly one prompt.

Alternative: the `trash` crate. Rejected; it is a new dependency for one Win32 call.

### 5. Opening a section from the tray

`show_settings(app, section: &str)` emits `settings-open` with the section id as payload. Tray `gallery` → `"gallery"`, tray `settings` and the `open_settings` command → `"general"`. `api.onSettingsOpen` passes the payload to `load(true, section)`, which calls `setSection(section)` instead of the hard-coded `"general"`. First run still switches to the setup view, so the section does not matter there.

### 6. Live update

In `Snap::user_event`, the `Saved` and `Recorded` arms emit `gallery-changed` to `main` before showing the toast. The gallery hook listens and re-runs `gallery_list`. Reloading the list instead of inserting the path keeps one code path for ordering and dating, and one `read_dir` is cheap. The hook also reloads on `settings-open`.

### 7. Frontend structure

- `lib/gallery.ts`: pure functions `groupBy(items, period)` and `matches(item, filter)`. A `PERIODS` mapping gives each period (day, month, year) its local key (`YYYY-MM-DD`, `YYYY-MM`, `YYYY`) and its `Intl.DateTimeFormat` options, so a new period is one entry instead of a branch. Header text uses the app language.
- `hooks/use-gallery.ts`: items, filter, viewer index, delete and copy actions, the `gallery-changed` and `settings-open` listeners. The components stay thin.
- `components/settings/gallery/`: `gallery.tsx` (group root: filter `ToggleGroup`, `Empty` state, day sections, or the viewer), `gallery-item.tsx` (thumbnail tile wrapped in `ContextMenu`), `item-menu.tsx` (the shared menu items, used by tile and viewer), `viewer.tsx`, `delete-dialog.tsx` (`AlertDialog`).
- The viewer replaces the grid inside the pane rather than opening a `Dialog`, so the title bar, drag strip and sidebar stay usable.
- `SECTIONS` gets a `wide` flag; `settings.tsx` drops `max-w-3xl` for wide sections so the grid uses the pane width.
- Keys: the viewer registers its `keydown` listener in the capture phase and calls `preventDefault` for Left, Right and Escape. `use-hotkey-recorder.ts` returns early when `e.defaultPrevented`, so Escape in the viewer closes the viewer and not the window.

### 8. Default context menu and reload keys

In `main.tsx`, under `import.meta.env.PROD`, add a document `contextmenu` listener that calls `preventDefault`, and a `keydown` listener that cancels F5, Ctrl+R and Ctrl+P. The listeners run in the bubbling phase on `document`, after the base-ui `ContextMenu` trigger has handled the event on the element, so snapr's own menus still open.

Alternative: disabling WebView2 browser accelerator keys and the default menu through native settings. Rejected for now; it needs reaching into the WebView2 controller from `setup`, and the JS listeners cover the user-visible behavior.

### 9. Constants

`THUMB_SIZE` (256 px, physical), `THUMB_WORKERS` (3) and the thumbnail `Cache-Control` max-age live as named constants at the top of `gallery.rs`.

### 10. File kinds beyond snapr's own output

`Kind` gains `Other`. Two const tables map extensions to kinds and MIME types: photos `png jpg jpeg gif webp bmp`, videos `mp4 webm` (what WebView2 renders and plays natively). Everything else is `Other`, including `mov`, `mkv` and `avi`, which WebView2 cannot reliably play and the default app handles better.

- `list` returns every regular file directly in the folder; `guard` drops the extension check and keeps the "parent equals output folder" check, so open, reveal and delete work for any file there.
- `media` refuses `Other`, so only renderable files are streamed into the webview.
- `thumb` asks the Shell for `SIIGBF_ICONONLY` for `Other`, and retries with `SIIGBF_ICONONLY` when a photo or video thumbnail fails, so the frontend placeholder is only a last resort.
- `gallery_copy` accepts every photo; the existing `image` crate gets its `gif`, `webp` and `bmp` features (no new crate).
- Frontend: the viewable list (photos and videos of the current filter) drives viewer navigation, the strip and delete-in-viewer; `Other` tiles show the name and ignore clicks.

## Risks / Trade-offs

- [`SIIGBF_THUMBNAILONLY` returns nothing for a file Windows has not thumbnailed and cannot (codec missing)] → 404 → placeholder tile; the item still works.
- [Thousands of items render thousands of DOM tiles] → lazy `<img loading="lazy">` keeps decoding bounded; virtualization is the upgrade if scrolling becomes slow.
- [Range handling bugs break seeking] → test seeking near the start, middle and end of a long recording; `curl -H "Range: bytes=0-99"` against the scheme URL in dev.
- [The `defaultPrevented` check in the hotkey recorder could mask Escape elsewhere] → only the viewer calls `preventDefault` on Escape; recorder behavior is unchanged otherwise.
- [Base-ui `ContextMenu` might rely on the event not being default-prevented] → the global listener runs after it in bubbling order; verify in a release build (task 6.2).
- [Canonicalizing a missing file fails] → guard returns `None` → 404 or command error; the list reloads on the next change or open.

## Migration Plan

None. No config or file format change; rollback is reverting the change.
