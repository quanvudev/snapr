# Proposal

## Why

The webcam bubble shows whatever is behind the user: a messy room, a whiteboard with private notes, people walking past. Other recorders and meeting apps offer background blur, and users expect it. The camera bubble is a WebView2 window that WGC records like any other window on screen, so the effect can run entirely in the webview with a tiny segmentation model and needs no change to the recorder.

## What Changes

- A "Blur background" checkbox item in the camera dropdown of the recording panel, under the camera list, disabled while the camera is off.
- With it on, the setup preview and the camera window draw the webcam through a `<canvas>`: MediaPipe `ImageSegmenter` (`@mediapipe/tasks-vision`, Apache-2.0) with the selfie segmenter model (about 250 KB, float16) runs on a 256 px wide copy of each frame (GPU delegate, CPU fallback), the confidence mask is smoothed over time, the frame is blurred and the person is drawn sharp on top. The WASM runtime and the model are bundled with the app, so nothing is downloaded and nothing leaves the computer.
- Until the first mask is ready, the whole frame is shown blurred, so the background is never visible unblurred while blur is on.
- A Camera card on Settings → Recording with the same Blur background switch and, while it is on, Strength (1-10, default 4) and Edge smoothing (0-90 %, default 50) sliders, stored as `record_blur_strength` / `record_blur_smoothing` (clamped through `Config::blur_strength()` / `blur_smoothing()`) and read when the panel opens. A round preview beside the sliders opens the camera only after its Preview button is pressed, shows the slider values live, and stops when the window loses focus or hides. The `main` window is now built in `setup` from its config (`create: false`) so it can grant camera access like the panel windows.
- An Image background: `CameraEffect` gains `image` and `record_camera_image` holds a user-picked photo (Settings → Recording, Off / Blur / Image… like the padding background). The panel's camera menu becomes a Background radio group (None / Blur background / Image, Image disabled until a photo is set). The `camera_background(path)` command returns the file's bytes (image extensions only); the photo is drawn to cover the frame, pre-mirrored because the feed is shown mirrored, and the frame stays blurred until the person mask and the photo are ready.
- New config field `record_camera_effect` (`none` default / `blur`, field-level default), saved with the other panel choices when a recording starts and read when the panel opens. The choice is locked while recording, like the devices.
- The MediaPipe code is a lazily imported chunk, so the settings window never loads it.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `recording-panel`: device selection gains the blur choice; blur strength and edge smoothing are configurable in Settings; the webcam window and preview apply it; remembered choices include it; it is locked while recording.

## Impact

- `package.json`: new dependency `@mediapipe/tasks-vision`. The bundle grows by about 15 MB (the vision WASM binary), loaded only by the panel and camera windows.
- `src/assets/selfie_segmenter.tflite` (SHA-256 `191ac9529ae506ee0beefa6b2c945a172dab9d07d1e802a290a4e4038226658b`).
- `src-tauri/src/config.rs`: `CameraEffect`, `record_camera_effect`.
- `src-tauri/tauri.conf.json` (`create: false` on `main`), `src-tauri/src/lib.rs` (builds `main` in `setup`).
- `src-tauri/src/panel.rs`: `allow_camera`, `PanelState.effect`, `record_select` (new argument), `record_start` (saves it), `open` (reads it), the camera window URL (`&effect=`).
- Frontend: `settings/recording/camera-blur-row.tsx`, `lib/background-blur.ts`, `components/camera-feed.tsx` (shared by `panel/setup-panel.tsx` and `camera/camera-app.tsx`), `lib/api.ts`, `lib/i18n.ts`.
