# Tasks

## 1. Background save worker

- [x] 1.1 In `lib.rs`, change `Action` from `Copy` to `Clone`, add `Saved(RgbaImage, PathBuf)`, and send `action.clone()` from the hotkey closure in `apply()`. Verify `cargo clippy` passes in `src-tauri/`.
- [x] 1.2 Add `enum Shot { Ready(RgbaImage), Window { hwnd, border, frame, rect } }` (in `overlay.rs`, next to `Outcome`, with `Rect` made `pub`). Change `Outcome::Save` to carry a `Shot`. Verify `cargo clippy` passes.
- [x] 1.3 In `spawn_capture`, create an `mpsc` channel and a worker thread that holds an `AppHandle` and an `EventLoopProxy<Action>` clone. For each `Shot`, the worker resolves the image (for `Window`: `capture::window`, falling back to a crop of `frame`), reads `cfg` from `State`, and calls `capture::save`. On success it restores the tooltip and sends `Saved(img, path)`; on error it sets the `snapr: {e}` tooltip. Verify `cargo clippy` passes.

## 2. Capture loop and overlay

- [x] 2.1 Store the `Sender<Shot>` in `Snap`. Replace `report()` with sending a `Shot`. Fullscreen sends `Shot::Ready(img)`, and `Outcome::Save(shot)` closes the overlay and then sends the shot. Verify `cargo clippy` passes.
- [x] 2.2 Handle `Action::Saved` in `user_event`: drop the current toast, and open `Toast::open` when `preview_seconds > 0` and `self.overlay.is_none()`. Verify `cargo clippy` passes.
- [x] 2.3 In `Overlay::event`, keep Area returning `Shot::Ready(crop)`. Make Window return `Shot::Window` with the hwnd, `edge_color(r)`, `std::mem::take(&mut self.frame)` and `r`, with no `capture::window` call on the click path. Move the crop fallback helper to where the worker can use it. Verify `cargo clippy` passes with no warnings.

## 3. Integration check

- [x] 3.1 Run `bun run tauri dev` and check every scenario in `specs/capture-output/spec.md`: the overlay closes instantly on area and window pick, fullscreen works, a hotkey pressed during a save opens a new overlay, the toast appears after the file exists and opens it on click, `preview_seconds = 0` shows no toast, two captures in one second produce `_1`, the clipboard holds the latest capture, and an unwritable folder shows the tooltip error.
- [x] 3.2 Update the Architecture section of `CLAUDE.md`: the capture loop hands shots to a background save worker that reports back with `Action::Saved`. Verify the text matches the code.
