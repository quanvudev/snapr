# Tasks

## 1. Backend

- [x] 1.1 Add `Padding { enabled, size, image }` with `#[serde(default)]` and defaults (false, 32, "") as `Config.padding` in `src-tauri/src/config.rs`; verify an old `config.toml` without `[padding]` still loads (`cargo clippy` clean)
- [x] 1.2 Enable the `jpeg` feature on the `image` dependency in `src-tauri/Cargo.toml`; verify `cargo clippy` builds
- [x] 1.3 Add `capture::pad(img, p, bg)` in `src-tauri/src/capture.rs` using `resize_to_fill` for the background (or a zeroed canvas) and `imageops::overlay` at `(p, p)`; verify `cargo clippy` is clean
- [x] 1.4 In `work()` in `src-tauri/src/lib.rs`, when padding is enabled and size > 0, load the background (if a path is set), pad the image before `capture::save`, and on a load error pad transparently and set the `snapr: <error>` tooltip after `refresh`; verify `cargo clippy` is clean
- [x] 1.5 Update the `config.rs` / save-worker notes in `CLAUDE.md` to mention padding in `work()`

- [x] 1.6 Add `round: bool` and `radius: u32` (defaults false, 12) to `Padding` in `src-tauri/src/config.rs`, add `capture::mask_corners(img, radius)` (alpha multiply, no outline, radius capped at min(w, h) / 2), and call it in `work()` before `pad` when `padding.round` is set; verify `cargo clippy` is clean

## 2. Settings UI

- [x] 2.1 Add the `padding` type to `Config` in `src/lib/api.ts` and `"capture"` to `SectionId` in `src/hooks/use-settings.ts`; verify `bunx tsc --noEmit`
- [x] 2.2 Add `en` and `vi` strings for the Capture group, padding, size, background, browse and clear to `src/lib/i18n.ts`; verify `bunx tsc --noEmit`
- [x] 2.3 Create `src/components/settings/capture/` with `capture.tsx`, `padding-row.tsx` (Switch), `size-row.tsx` (number Input, 1 to 512, commit on blur/Enter) and `background-row.tsx` (path, browse via `@tauri-apps/plugin-dialog` `open` with PNG/JPEG filters, clear), with the size and background rows disabled while padding is off; verify `bunx tsc --noEmit`
- [x] 2.4 Register the Capture group between General and Preview in `src/components/settings/sections.ts`, and update the sidebar list in `CLAUDE.md`; verify `bunx tsc --noEmit`

- [x] 2.5 Add `round` and `radius` to the `padding` type in `src/lib/api.ts`, add `en`/`vi` strings to `src/lib/i18n.ts`, and create `src/components/settings/capture/radius-row.tsx` (Switch plus number Input 1 to 128; the whole row is disabled while padding is off, and the input also while `round` is off), placed after the size row in `capture.tsx`; verify `bunx tsc --noEmit`

## 3. Integration check

- [x] 3.1 In `bun run tauri dev`: with padding 64 and no image, capture a window and verify the PNG is `(w+128) x (h+128)` with transparent padding, and that a paste from the clipboard has the same size
- [x] 3.2 Select a JPEG background, take fullscreen and area captures, and verify the background covers the canvas without distortion and the toast shows the padded image
- [x] 3.3 Delete the background file, capture, and verify the PNG is saved with transparent padding and the tray tooltip shows the error; turn padding off and verify captures are back to their original size
- [x] 3.4 Turn on rounded corners with radius 16 over a background, take area and window captures, and verify the corners are rounded with the background showing through; with radius 4 on a window capture, verify there are no dark corner pixels
