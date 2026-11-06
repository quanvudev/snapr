# Tasks

## 1. Backend

- [x] 1.1 Add `CameraEffect` (`none` / `blur`) and `record_camera_effect` to `config.rs`, with a unit test that a config without the key loads as `none`; verify `cargo test --lib camera_effect` passes
- [x] 1.2 Add `effect` to `PanelState`, read it in `open`, take it in `record_select`, save it in `record_start` and pass it to the camera window as `&effect=`; verify `cargo clippy` is clean

## 2. Frontend

- [x] 2.1 Add `@mediapipe/tasks-vision` and bundle `selfie_segmenter.tflite`; `lib/background-blur.ts` (`startBlur(video, canvas)`) and `components/camera-feed.tsx` (lazy import); verify `bunx tsc --noEmit` passes and `bunx vite build` emits the WASM and model assets
- [x] 2.2 Use `CameraFeed` in the setup preview and the camera window; add the Blur background checkbox item to the camera dropdown (en "Blur background" / vi "Làm mờ nền")
- [x] 2.3 In `bun run tauri dev`, tick Blur background: the preview blurs the room and keeps the face sharp, starts blurred (no unblurred flash), and the camera window keeps the effect through countdown and recording

- [x] 2.4 Add `record_blur_strength` / `record_blur_smoothing` with clamping accessors (unit tested), carry them through `PanelState` and the camera URL into `startBlur`, and add the Camera card (`camera-blur-row.tsx`: Switch plus Strength and Edge smoothing sliders) to the Recording page
- [x] 2.5 Add a live round preview to the Camera card (Preview button, saved camera else the first, follows slider drafts, stops on window blur or hide); build `main` in `setup` through `from_config` with `panel::allow_camera` (`create: false` in `tauri.conf.json`) so the preview never prompts
- [x] 2.6 In `bun run tauri dev`, open Settings → Recording, turn blur on, press Preview and drag both sliders: the preview changes live, no permission prompt appears, and the camera light turns off when the window is hidden
- [x] 2.7 Add an Image background: `CameraEffect::Image`, `record_camera_image`, `Config::camera_effect()` (unit tested), the `camera_background(path)` command, a mirrored cover-fit background in `startBlur`, a Background radio group (None / Blur background / Image) in the panel camera dropdown and an Off / Blur / Image… ToggleGroup with Browse in the Camera card
- [x] 2.9 Soften the person's edge in Image mode: wider `IMAGE_EDGE` thresholds, a 1 px feathered mask and a light wrap of the blurred image (cached per bitmap) at 50 % inside the edge band
- [x] 2.10 Make the Image edge configurable: `record_bg_edge`, `record_bg_feather`, `record_bg_wrap`, `record_bg_wrap_radius` with the blur fields grouped as `CameraLook` (`Config::camera_look()`, clamped, unit tested), passed as `PanelState.look` and a JSON `look` URL param, and driven by a `KNOBS` slider table in the Camera card
- [x] 2.8 In `bun run tauri dev`, pick an image with text in it: the preview and the recorded bubble show it the right way round behind the person, the panel offers Image only once an image is set, and the recording keeps it

## 3. Docs and integration

- [x] 3.1 Update the `config.rs` and `panel.rs` paragraphs in `CLAUDE.md`
- [x] 3.2 End-to-end in a release build (`bun run tauri build`): record 20 s with blur on; verify the saved video shows the blurred background in the bubble, CPU use stays reasonable, and the choice is preselected next time

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
