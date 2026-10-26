# Proposal

## Why

Captures pile up in the output folder and the only way to find one again is the tray "Open folder" item, which drops the user into Explorer with a flat list of timestamped files. snapr should let users browse their own screenshots and recordings like a phone photo library: grouped by day, previewed and played in place, with the common file actions one right-click away.

## What Changes

- New **Gallery** group, first in the settings sidebar. It shows every file in the output folder (photos: png, jpg, jpeg, gif, webp, bmp; videos: mp4, webm; anything else as a file-type icon that cannot be opened in the viewer) as a thumbnail grid grouped under day headers (or month / year headers, chosen with a Day / Month / Year switch), newest first.
- The date of each item is the earlier of the file's created and modified time, so copying a file (new created time) or editing it (new modified time) does not move it to another day. File names are not parsed.
- Filter: All / Photos / Videos.
- Each group header collapses and expands its group.
- Thumbnails come from Windows (the same thumbnails Explorer shows), for both images and videos.
- Clicking a thumbnail opens an in-app viewer: images shown fitted, videos played with controls. Left/Right moves to the previous/next item in the current filter, Escape closes the viewer. A thumbnail strip under the media shows the filtered items with the current one highlighted; clicking a thumbnail jumps to it.
- Right-click on a thumbnail (and in the viewer) offers: Open with default app, Show in Explorer, Copy image (photos only; other files get only Open, Show in Explorer and Delete), Delete. Delete asks for confirmation and moves the file to the Recycle Bin.
- The gallery updates live: a capture or recording saved while the window is open appears at the top without reopening.
- The tray menu gains a **Gallery** item above Settings…; it opens the window on the Gallery group. Settings… keeps opening on General.
- The WebView default context menu (Back / Reload / Save as / Print / Inspect) and the reload/print shortcuts (F5, Ctrl+R, Ctrl+P) are disabled in release builds, so the window cannot be reloaded or printed by accident. Development builds keep them.
- Not in this change: iPhone-style month/year cards with drill-down, albums, multi-select, zoom in the viewer, editing.

## Capabilities

### New Capabilities
- `gallery`: browsing the captures in the output folder: listing and dating files, day / month / year grouping, type filter, thumbnails, in-app viewer, per-item actions (open, reveal, copy, delete) and live updates.

### Modified Capabilities
- `settings-window`: the sidebar gains Gallery as the first group; the tray has a Gallery item that opens the window on Gallery while Settings… still opens on General; the WebView default context menu and reload/print shortcuts are disabled in release builds.

## Impact

- Backend: new `src-tauri/src/gallery.rs` (listing, Windows Shell thumbnails, full-file serving with Range, delete to Recycle Bin, open/reveal/copy commands, output-folder path guard); `lib.rs` (`thumb` and `media` URI schemes, `show_settings` takes the section to open, tray Gallery item, gallery-changed event after a save or recording, command registration).
- Config: `Cargo.toml` adds `Win32_UI_Shell`, `Win32_System_Com` and `Win32_Graphics_Gdi` features to the existing `windows`/`windows-sys` crates. No new crates, no `tauri.conf.json` or capability change.
- Frontend: new `components/settings/gallery/` group and its entry first in `sections.ts`; `SectionId` gains `gallery`; `use-settings.ts` reads the section from the `settings-open` payload; `lib/api.ts` wrappers; `lib/i18n.ts` strings; `main.tsx` disables the default context menu and shortcuts in production.
- Not affected: capture, padding, OCR and recording behavior; config file format.
