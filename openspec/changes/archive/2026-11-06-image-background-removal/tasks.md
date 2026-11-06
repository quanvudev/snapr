# Tasks

## 1. Model store

- [x] 1.1 Create `src-tauri/src/cutout.rs` from `translate.rs`. It needs the pinned `Remote` for `BEN2_Base.onnx` from `PramaLLC/BEN2` at commit `e48a20765fb421d19dcdb0bf3cc61e802ca5ec8f` (SHA-256 `22cea62108ff53b7ccc20f7a008bf30494228d84b1687f29ecbe76936a998101`, 222 932 053 bytes) stored as `model.onnx` in `app_local_data_dir()/cutout/ben2-base/`, plus `ready`, `installed`, `model_dir`, `IN_USE` / `acquire`, and `State.cutout_download`. The download runs `onnx::ensure` then the fetch, and removes the folder on failure or cancel. Verify with `cargo clippy` clean
- [x] 1.2 Add `CUTOUT_PROGRESS` / `CUTOUT_DONE` to `events.rs`, register `cutout_status` / `cutout_download` / `cutout_cancel` / `cutout_delete` in `lib.rs` (delete refuses the downloading model and `IN_USE`, then calls `onnx::release`), add `cutout::installed` to `onnx::release` and the cutout slot to every caller's `busy`. Verify that `cargo build` succeeds and that a dev run downloads, shows progress, and that a delete frees the folder
- [x] 1.3 Update the `CLAUDE.md` module list (`cutout.rs`, events, commands, `onnx::release`) and verify that the identifiers match the code

## 2. Inference and queue task

- [x] 2.1 Add `Workload::Cutout` to `onnx.rs`: on the CPU it uses the same thread settings as `Ocr`, and `gpu_for` places it on CUDA and DirectML. Extend `resolves_runtimes`, or add a unit test for `gpu_for` placement, and verify that `cargo test --lib onnx` passes
- [x] 2.2 Create `src-tauri/src/cutout/task.rs` with `CutoutTask::validated` (`gallery::guard`, `png` only, `MODEL_MISSING` when not ready, `onnx::target` fixed) and `run` (decode, 1024x1024 `Triangle` resize, ImageNet-normalised NCHW, session through `onnx::retry_on_cpu(Workload::Cutout, ..)`, f16 output already in [0,1] with the official min-max rescale, mask upscaled to the source size, alpha = mask × source alpha, progress 10 / 80 / 100, cancel before and after the run). The input and output names must be confirmed against the graph (`input.1`, first output f16). Add pure unit tests for the preprocessing tensor layout and the rescale and mask-to-alpha steps, and verify that `cargo test --lib cutout` passes
- [x] 2.3 Add `TaskSpec::Cutout { path }` and its `into_task` arm to `queue.rs`, writing through `queue::write_part(library::images(cfg), "<stem>_cutout", "png", ..)`, and add `png` to `PART_EXTS`. Verify that `cargo clippy` is clean
- [x] 2.4 Add an ignored test `renders_cutout` (`SNAPR_CUTOUT_MODEL`, `SNAPR_CUTOUT_SOURCE`, `SNAPR_CUTOUT_OUT`, optional `SNAPR_ORT_DLL` / `SNAPR_ORT_EP`) that writes a cutout, and verify by eye on a photo and a UI screenshot on the CPU and DirectML that the masks match. If DirectML is wrong, drop it from `gpu_for` and update the ai-acceleration delta

## 3. Frontend: AI models

- [x] 3.1 Add the `cutout_*` wrappers, the `CutoutStatus` type and `cutoutSpec` / `TASK_KINDS.cutout` to `lib/api.ts` and `lib/queue.ts`, add en/vi strings (card title, description, Used for, Remove background, task kind label) to `lib/i18n.ts`, and verify that `bunx tsc --noEmit` passes
- [x] 3.2 Create `hooks/use-cutout.ts` (status on mount and `settings-open`, progress and done listeners, download / cancel / remove), expose it as `cutout` in the settings context, add `useCutoutStore` to `models/stores.ts`, add `models/cutout-card.tsx` (`ModelCard` with the description, about 223 MB, the MIT link and a `model-usage-row` for Remove background in the gallery) to `models.tsx`, list it in `downloaded-models.tsx`, and add the model to the About notices. Verify in the app that the card downloads, shows progress and cancels, and that Downloaded models lists and deletes it with the total updated

## 4. Frontend: gallery

- [x] 4.1 Add a Remove background header button in `viewer.tsx` (photos only), an `item-menu.tsx` entry (photos only) and a `bulk-bar.tsx` action (enabled when the selection has a photo, one `queue_add` per selected photo in visible order), all through `queue_add(cutoutSpec(path))`, with refusals shown through `taskError` so `MODEL_MISSING` gets "Open AI models". Verify in the app that each entry point queues, shows "Added to queue", and that the `_cutout.png` appears in the gallery with a transparent background. With the model deleted, verify that the toast offers Open AI models and nothing is queued

## 5. Integration

- [x] 5.1 End-to-end: on a PNG in `Images` and another in the output root, run Remove background twice each and cancel one run. Verify that the outputs are in `Images` as `_cutout.png` and then `_cutout_1.png`, that the originals are unchanged, that the cancel leaves no `.part`, and that the queue sheet's history shows the kind label

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
