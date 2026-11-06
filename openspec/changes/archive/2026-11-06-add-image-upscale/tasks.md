# Tasks

## 1. Backend: model store and placement

- [x] 1.1 Add `Workload::Upscale` to `onnx.rs`: `on_card` true for CUDA and DirectML, `cpu_session` gives it the half-cores no-spin setup like `Cutout`; extend `places_workloads` and verify `cargo test --lib onnx` passes
- [x] 1.2 Create `upscale.rs` shaped like `cutout.rs` (pinned `Remote` for `realesr-general-x4v3.onnx` at commit `899dc1e4b22bbf1955c2f1739c085edc080cb366`, 4,866,396 bytes, SHA-256 `924ebad6532777303582d4ce7811849b88869231a3ae7093e0f21200249df8d5`; `model_dir`, `installed`, `ready` (plus the runtime DLL), `acquire` / `IN_USE`, `Download`, `upscale_status` / `upscale_download` / `upscale_cancel` / `upscale_delete`), add `UPSCALE_PROGRESS` / `UPSCALE_DONE` to `events.rs`, `State.upscale_download` and the commands in `lib.rs`; verify `cargo clippy` is clean
- [x] 1.3 Count the new store everywhere a store is counted: the busy check in the paddle, whisper, translate and cutout delete commands includes `upscale_download`, `upscale_delete` counts theirs, and `onnx::release` keeps the CPU DLL while `upscale::installed`; verify by reading each call site and `cargo clippy`

## 2. Backend: the upscale task

- [x] 2.1 Create `upscale/task.rs` with `UpscaleTask::validated(app, path, scale)`: `gallery::guard`, PNG only, scale 2 or 4 else an error, `MODEL_MISSING` when not ready, `TOO_LARGE` when `max(w, h) * scale > MAX_SIDE` (8192) from `image::image_dimensions`, `onnx::target` fixed; add `TaskSpec::Upscale { path, scale }` and its `into_task` arm in `queue.rs`; verify `cargo clippy` is clean
- [x] 2.2 Implement the pure helpers and unit-test them: the tile plan (256 px tiles with 16 px clipped context, centre copy covers every output pixel exactly once, including sizes that are not multiples of 256 and images smaller than one tile), the NCHW `[0, 1]` tensor of a tile, and the ×2 / alpha recombination (output size, alpha resized with Lanczos3, transparent stays transparent); verify `cargo test --lib upscale` passes
- [x] 2.3 Implement `run`: decode, `acquire`, one session per task under `onnx::retry_on_cpu(Workload::Upscale, ..)`, tiles with cancel checked before each, progress 0-10 / 10-90 / 100, ×2 halving with Lanczos3, alpha recombined, `queue::write_part(library::images(cfg), "<stem>_x<scale>", "png", ..)`; add the ignored `renders_upscale` test (`SNAPR_UPSCALE_MODEL`, `SNAPR_UPSCALE_SOURCE`, `SNAPR_UPSCALE_OUT`, optional `SNAPR_UPSCALE_SCALE`, `SNAPR_ORT_DLL`, `SNAPR_ORT_EP`) and verify it on a screenshot crop with the A/B model on DirectML and on the CPU: correct output size, no visible tile seams, timing printed
- [x] 2.4 Document the store, the task, `Workload::Upscale` and `TaskSpec::Upscale` in `CLAUDE.md` (module list, onnx placement rule, queue kinds) and verify the identifiers named there exist

## 3. Frontend: model card and About

- [x] 3.1 Add the API wrappers and types in `lib/api.ts` (`upscaleStatus`, `upscaleDownload`, `upscaleCancel`, `upscaleDelete`, the two event listeners), `lib/upscale.ts` (name, id, licence, URL) and `hooks/use-upscale.ts` mirroring `use-cutout.ts`, exposed as `upscale` in the settings context; verify `bunx tsc --noEmit` passes
- [x] 3.2 Add `models/upscale-card.tsx` (description, about 4.6 MB, BSD-3-Clause link, status line, "Used for: Upscale in the gallery") to the AI models page, a `useUpscaleStore` in `models/stores.ts` listed by `downloaded-models.tsx`, `about/upscale-notice.tsx`, and the en / vi strings; verify in `bun run tauri dev` that Download, Cancel, Retry, the Downloaded models entry, Delete and the About notice behave as the settings-window spec says

## 4. Frontend: gallery entry points

- [x] 4.1 Add `upscaleSpec(path, scale)`, `TASK_KINDS.upscale` and `TOO_LARGE` in `TASK_ERRORS` to `lib/queue.ts`, and `upscaleAll(targets, scale)` with `upscale(item, scale)` / `upscaleSelected(scale)` actions in `use-gallery.ts` (one `queue_add` per photo in visible order, each refusal shown through `taskError` without stopping the rest); verify `bunx tsc --noEmit` passes
- [x] 4.2 Add the Upscale `ContextMenuSub` (×2, ×4) to `item-menu.tsx` for photos, an Upscale `DropdownMenu` button to the viewer header for photos and to `bulk-bar.tsx` (disabled without a photo), with en / vi strings; verify in `bun run tauri dev` each scenario of the gallery spec: context menu ×2, viewer ×4, bulk with an MP4, bulk with a photo that is too large, nothing offered on an MP4
- [x] 4.3 Update the gallery and frontend paragraphs of `CLAUDE.md` (entry points, `upscaleAll`, the card, the store hook) and verify they name the real identifiers

## 5. Integration

- [x] 5.1 End-to-end in a release build (`bun run tauri build`): download the model from AI models, upscale a padded screenshot crop by ×2 and by ×4 from the gallery, and verify the files are `<stem>_x2.png` / `<stem>_x4.png` in `Images` at the right size, text and Vietnamese accents are unchanged, the padding stays transparent, a 2560×1440 photo at ×4 is refused, Cancel leaves no file, and with DirectML the queue item finishes in about a second

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
