# Tasks

## 1. Backend dependencies

- [x] 1.1 In `src-tauri/Cargo.toml` add features `Win32_UI_Shell`, `Win32_System_Com`, `Win32_Graphics_Gdi` to `windows` 0.62 and `Win32_UI_Shell` to `windows-sys` 0.61 (no new crates); verify `cargo clippy` passes and `cargo tree -i windows@0.62` still shows a single copy

## 2. Gallery backend

- [x] 2.1 Create `src-tauri/src/gallery.rs` with constants `THUMB_SIZE`, `THUMB_WORKERS`, thumbnail max-age, `Item { path, date_ms, kind }`, `list(dir)` (png/mp4 only, case-insensitive, no subfolders, `date_ms = min(created, modified)`, newest first) and `guard(app, path)` (canonical parent equals canonical `Config::dir()`, extension png/mp4) per design §1; verify `cargo clippy` passes and a dev-build `gallery_list` on a folder with `a.png`, `b.MP4`, `notes.txt` and `old/c.png` returns only the two media files
- [x] 2.2 Add commands `gallery_list`, `gallery_open` (opener `open_path` from Rust), `gallery_reveal` (`explorer /select,`), `gallery_copy` (decode PNG, reuse `capture::copy`, refuse mp4), `gallery_delete` (`SHFileOperationW` with `FOF_ALLOWUNDO` and no Shell UI, per design §4), each going through `guard`; register them in `generate_handler!`; verify `cargo clippy` passes and calling any of them with `..\` or a path outside the folder returns an error without touching the file
- [x] 2.3 Register the `thumb` asynchronous URI scheme with the STA worker pool (`CoInitializeEx` once per thread, `IShellItemImageFactory::GetImage` with `SIIGBF_THUMBNAILONLY | SIIGBF_BIGGERSIZEOK`, `GetDIBits` → RGBA → PNG, `Cache-Control`, 404 on guard or Shell failure) per design §2; verify in `bun run tauri dev` that `http://thumb.localhost/<encoded png path>` and an mp4 path both return a PNG and an outside path returns 404
- [x] 2.4 Register the `media` asynchronous URI scheme serving the guarded file with `200` or `206` + `Content-Range` + `Accept-Ranges` and the extension's content type per design §3; verify a `Range: bytes=0-99` request returns 100 bytes with status 206 and an outside path returns 404
- [x] 2.5 Update `CLAUDE.md` backend modules and commands for `gallery.rs`, the `thumb`/`media` schemes and the output-folder guard; verify it matches the code

- [x] 2.6 Extend `gallery.rs` per design section 10: `Kind::Other`, extension tables for photo (`png jpg jpeg gif webp bmp`) and video (`mp4 webm`) with their MIME types, `list` returns every file (no subfolders), `guard` accepts any file directly in the folder, `media` refuses `Other`, `thumb` returns the Shell icon for `Other` and falls back to the icon when a thumbnail fails, `gallery_copy` accepts every photo (enable `gif`, `webp`, `bmp` features on `image`); verify `cargo clippy` passes
## 3. Tray and window opening

- [x] 3.1 Change `show_settings(app, section)` to emit `settings-open` with the section id; tray gets a `Gallery` item first in the menu calling it with `"gallery"`, while `Settings…` and the `open_settings` command pass `"general"`; verify `cargo clippy` passes and the tray lists Gallery, Settings…, Open folder, Stop recording, Quit
- [x] 3.2 In `Snap::user_event` emit `gallery-changed` to `main` in the `Saved` and `Recorded` arms; verify the event fires on an area capture and on stopping a recording (log or devtools listener in dev)
- [x] 3.3 Update `lib/api.ts` (`onSettingsOpen` passes the section, `onGalleryChanged`, typed wrappers for the five gallery commands, `GalleryItem` type) and `use-settings.ts` (`SectionId` gains `gallery`, `load(true, section)` sets that section); verify `bunx tsc --noEmit` passes, Settings… opens on General and Gallery opens on the Gallery group
- [x] 3.4 Update `CLAUDE.md` (tray items, `settings-open` payload, `gallery-changed`) and the settings sidebar list; verify it matches the code

## 4. Gallery UI

- [x] 4.1 Add `lib/gallery.ts` with `groupByDay` (local day key, days and items newest first, count per day) and `matches(item, filter)`; verify `bunx tsc --noEmit` passes and grouping two items from today plus one from another day yields two groups in the right order (quick check in devtools console)
- [x] 4.2 Add `hooks/use-gallery.ts` (items, filter, viewer index, reload on mount, `settings-open` and `gallery-changed`, copy and delete actions; delete removes the item and moves the viewer to next, else previous, else closes); verify `bunx tsc --noEmit` passes
- [x] 4.3 Add `components/settings/gallery/` (`gallery.tsx`, `gallery-item.tsx`, `item-menu.tsx`, `delete-dialog.tsx`) using shadcn `ToggleGroup`, `Empty`, `ContextMenu`, `AlertDialog`; tiles use `convertFileSrc(path, "thumb")` with `loading="lazy"`, a video marker and a placeholder on error; add the entry first in `sections.ts` with a `wide` flag honored by `settings.tsx`; add en/vi strings to `lib/i18n.ts`; verify the grid shows day headers with counts, the filter hides empty days, the menu shows Copy image only on PNG items, and Delete asks, then moves the file to the Recycle Bin
- [x] 4.4 Add `viewer.tsx` (image fitted, `<video controls>` for mp4, both via `convertFileSrc(path, "media")`, capture-phase keydown for Left/Right/Escape with `preventDefault`, same context menu) and make `use-hotkey-recorder.ts` skip events with `defaultPrevented`; verify Left/Right navigate within the filter, Escape closes the viewer but not the window, a video can be seeked, and deleting in the viewer shows the next item
- [x] 4.5 Update `CLAUDE.md` frontend section (gallery group, `lib/gallery.ts`, `use-gallery.ts`, `wide` sections, viewer Escape rule); verify it matches the code

- [x] 4.6 Generalize `groupByDay` into `groupBy(items, period)` driven by a `PERIODS` mapping (day / month / year: key and `Intl.DateTimeFormat` options), add `period` state to `use-gallery.ts` and a Day / Month / Year `ToggleGroup` in `gallery.tsx`, en/vi strings, and mention it in `CLAUDE.md`; verify `bunx tsc --noEmit` passes and Month shows one header per month with counts, combined with the type filter
- [x] 4.7 Add a thumbnail strip under the viewer media (`viewer-strip.tsx`, shadcn horizontal `ScrollArea`, tiles via `api.thumbSrc` with `loading="lazy"`, active tile highlighted and `scrollIntoView({ inline: "center" })` on change, click selects); `use-gallery.ts` passes the filtered items and a select action; shrink the media height to fit; mention it in `CLAUDE.md`; verify `bunx tsc --noEmit` passes and arrows, clicks and the filter keep the strip in sync
- [x] 4.8 Handle `kind: "other"` in the frontend: tile shows `Thumb` plus the file name and clicking does nothing, filter Photos/Videos excludes it, viewer navigation, strip and delete-in-viewer use only photos and videos, menu shows Copy image for every photo; update `CLAUDE.md`; verify `bunx tsc --noEmit` passes and a `.txt` file shows an icon tile that does not open the viewer
- [x] 4.9 Make each group collapsible with the shadcn `Collapsible` (header with chevron and count as the trigger, items as the content), collapsed keys held in `use-gallery.ts` state, groups expanded by default; update `CLAUDE.md`; verify `bunx tsc --noEmit` passes and collapsing one group hides only its items while its count stays
## 5. Browser defaults

- [x] 5.1 In `src/main.tsx`, under `import.meta.env.PROD`, cancel `contextmenu` on `document` and cancel F5, Ctrl+R and Ctrl+P on `keydown`; verify `bunx tsc --noEmit` passes and in `bun run tauri dev` the default menu still appears

## 6. Integration

- [x] 6.1 Run `bun run tauri dev` and walk every scenario in `specs/gallery/spec.md` and the gallery-related scenarios in `specs/settings-window/spec.md`: mixed folder, copied/edited/renamed dates, filter, video thumbnail, viewer navigation and seeking, each menu action, delete confirm/cancel/locked file, capture while open, change output folder, tray Gallery on first run
- [x] 6.2 Run `bun run tauri build`, launch the release build and verify right-click on an empty area shows no menu, F5/Ctrl+R do nothing, and the gallery item menu still opens
- [x] 6.3 Run `bunx tsc --noEmit` and `cargo clippy` in `src-tauri/`; verify both pass with no new warnings
