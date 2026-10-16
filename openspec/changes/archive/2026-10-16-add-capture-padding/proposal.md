# Proposal

## Why

Screenshots shared in chats, docs and social posts look better with a border of space around them, often over a wallpaper. Today users have to add that padding in another tool after every capture.

## What Changes

- New optional padding applied to every screenshot (fullscreen, window, area) before it is saved and copied.
- The padded image is `(w + 2p) x (h + 2p)`: the original capture is never scaled and sits at `(p, p)`.
- The padding is filled with a user-selected background image (scaled to cover the canvas, center-cropped) or, when no image is selected, left transparent.
- Optional rounded corners on the screenshot inside the padding (a switch plus a radius in px). The background or transparency shows through the corners. It is only available while padding is on.
- If the selected image can't be read, the capture is still saved with transparent padding and the error appears in the tray tooltip.
- New **Capture** group in the settings sidebar with: padding on/off, padding size in px, rounded corners on/off with a radius in px, and a background image picker with a clear action.
- Background images may be PNG or JPEG.
- Recordings are not affected.

## Capabilities

### New Capabilities
- `capture-padding`: optional padding around screenshots, with a transparent or image background, and optional rounded screenshot corners, applied to the saved file, the clipboard and the preview.

### Modified Capabilities
- `settings-window`: the sidebar gains a Capture group listing the padding settings.

## Impact

- `src-tauri/src/config.rs`: new `padding` section in the TOML config (defaults keep old config files loading).
- `src-tauri/src/capture.rs` / `lib.rs`: padding step in the save worker before `capture::save`.
- `src-tauri/Cargo.toml`: enable the `jpeg` feature of the existing `image` crate (no new crate).
- Frontend: `lib/api.ts` type, `lib/i18n.ts` strings, `hooks/use-settings.ts` `SectionId`, new `components/settings/capture/` group, entry in `settings/sections.ts`. Uses the existing `@tauri-apps/plugin-dialog` for the file picker.
