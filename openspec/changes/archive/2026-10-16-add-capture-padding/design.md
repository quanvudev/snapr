# Design

## Context

Every screenshot already passes through one place: `work()` in `src-tauri/src/lib.rs`, running on the save worker thread. It turns the `Shot` into an `RgbaImage`, then `capture::save` writes the PNG and copies that buffer to the clipboard. `Action::Saved(img, path)` passes the same image to the toast. Recordings use `Recorder` and never reach `work()`.

The `image` crate (0.25) is built with only the `png` feature. The settings UI already uses `@tauri-apps/plugin-dialog` `open()` in `general/folder-row.tsx`, and `dialog:default` is already granted.

## Goals / Non-Goals

**Goals:**
- A single padding step that covers fullscreen, window and area captures, the file, the clipboard and the toast.
- Changes in the Capture group take effect on the next capture through the existing autosave/`apply()` path.

**Non-Goals:**
- Solid-color fills, shadows, rounded outer canvas corners, per-side padding, or stretch/tile fit modes.
- Copying the background image into the config folder.
- Caching the decoded background between captures.
- Padding recordings.

## Decisions

### Config shape
Add `#[serde(default)] pub struct Padding { enabled: bool, size: u32, image: String }` as `Config.padding`, with defaults `false`, `32` and `""`, plus `round: bool` and `radius: u32` (defaults `false` and `12`). An empty `image` means a transparent fill, so there's no separate mode field: the UI's "clear" action sets the path back to empty. Thanks to `#[serde(default)]`, old `config.toml` files still load. The JSON contract (`api.ts`) mirrors it in snake_case.
- *Alternative:* a `mode: "transparent" | "image"` enum. Rejected because it duplicates what "is a path set" already says.

### Where padding happens
In `work()`, after the image is produced and before `capture::save`. If `cfg.padding.enabled && size > 0`, the image is replaced by its padded version, so save, clipboard and `Action::Saved` all get the padded buffer with no other changes.
- *Alternative:* pad inside `capture::save`. Rejected because the toast gets `img` from `work()`, so it would show the unpadded image.

### Padding function
Add `capture::pad(img: &RgbaImage, p: u32, bg: Option<&DynamicImage>) -> RgbaImage`:
- With a background, the canvas is `bg.resize_to_fill(w + 2p, h + 2p, FilterType::Triangle).into_rgba8()`. That call scales to cover the canvas and center-crops in one step, which is exactly the cover behavior.
- Without one, the canvas is `RgbaImage::new(w + 2p, h + 2p)`, which is all zeros, so the padding is transparent.
- `imageops::overlay(&mut canvas, img, p, p)` then draws the capture on top. It alpha-blends, so the transparent rounded corners of window captures show the background through.

### Rounded screenshot corners
When `round` is on, `work()` rounds the screenshot's corners before calling `pad`, so `imageops::overlay` blends the background or transparency into the corners. A new `capture::mask_corners(img: &mut RgbaImage, radius: f32)` uses the same signed-distance formula as `round_corners`, but multiplies each pixel's existing alpha by the mask (`a = a * mask`) and draws no outline. The radius is capped at `min(w, h) / 2`.
- *Alternative:* reuse the v1 `round_corners`. Rejected because it overwrites alpha, so a radius smaller than a window's own corners would turn those transparent pixels black, and it draws a 1px grey outline that suits window borders but not fullscreen or area captures.
- The outer canvas stays square.

### Loading the background
`work()` calls `image::open(&cfg.padding.image)` when the path is not empty. On error, it pads with `None` and keeps the error message. After `capture::save` succeeds, `refresh(app)` resets the tooltip, so the background error is written with `tooltip(app, "snapr: <error>")` **after** `refresh`. Otherwise the error would be cleared straight away. The image is decoded once per capture on the worker thread, which never blocks the overlay or hotkeys.

### JPEG support
Change `image` to `features = ["png", "jpeg"]`. It's the same crate; only its pure-Rust JPEG decoder is added.

### Settings UI
Add a `components/settings/capture/` group with one file per row, following the existing settings pattern:
- `capture.tsx`, which renders the group
- `padding-row.tsx`, with a `Switch`
- `radius-row.tsx`, with a `Switch` for `round` and a number `Input` for `radius` (1 to 128, commit on blur or Enter); the input is disabled while `round` is off
- `size-row.tsx`, with an `Input type="number"` that commits on blur or Enter and is clamped to 1 to 512, like the Preview seconds input
- `background-row.tsx`, showing the path with a browse `Button` (`open({ filters: [{ name: "Image", extensions: ["png", "jpg", "jpeg"] }] })`) and a clear `Button`

The size, radius and background rows are disabled while padding is off. The group also needs `"capture"` added to `SectionId`, an entry between General and Preview in `sections.ts` (lucide `Frame` icon), and `en`/`vi` strings in `i18n.ts`.

## Risks / Trade-offs

- [A large wallpaper (for example 6K) is decoded and resized on every capture, which can take about 100 ms] → This runs on the worker thread, and the toast appears after the save as it does today. Add a cache only if users notice the delay.
- [Windows clipboard apps may show transparent padding as black or white] → The user has accepted this as out of scope.
- [Fullscreen captures grow beyond the monitor size] → This is intended: the output is always the original plus `2p`.
