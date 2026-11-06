# Design

## Context

Background removal (`cutout.rs`, `cutout/task.rs`) already does what this change needs around the model: a pinned single-file store with `ready` / `installed` / `acquire`, a download slot in `State` with progress and done events, four commands, a `Task` kind validated at queue time through `TaskSpec`, `onnx::target` fixed at queue time, `onnx::retry_on_cpu`, `queue::write_part` into `library::images`, and three gallery entry points. Upscaling follows that shape; what is new is running a fully convolutional model on images of any size, the scale choice and the alpha channel.

The model choice comes from the A/B test in proposal.md. `realesr-general-x4v3` (SRVGGNetCompact, 1.2 M parameters) from `jonathanst29/tinier-upscale-models` at commit `899dc1e4b22bbf1955c2f1739c085edc080cb366`: file `realesr-general-x4v3.onnx`, 4,866,396 bytes, SHA-256 `924ebad6532777303582d4ce7811849b88869231a3ae7093e0f21200249df8d5`, fp32, opset 17, dynamic H/W, input `input` float32 `[1, 3, h, w]` RGB in `[0, 1]`, output `output` `[1, 3, 4h, 4w]`. Measured: 12 ms per 256×256 tile on DirectML (RTX 5060 Ti), 332 ms on 8 CPU threads.

## Goals / Non-Goals

**Goals:**
- Any photo size up to the limit, with bounded memory, on CPU, DirectML and CUDA.
- Seamless results: no visible tile borders.
- Cancel within about one tile's time.

**Non-Goals:**
- Other scale factors than ×2 and ×4, or a second model (anime and x4plus lost the A/B).
- Upscaling from Draw, pins or the capture overlay; videos.
- Denoise-strength control (the `wdn` variant of the model).

## Decisions

### Tiled inference with overlap
The image is cut into 256×256 tiles (`TILE`), each read with `PAD` 16 px of neighbouring context on every side (clipped at the image edges), run through the model, and only the centre `4 * TILE` region of the output is copied into the result. Receptive-field context of 16 px removed every seam in the A/B grid. One session is built per task and reused for every tile.
- Alternative: one whole-image run. Fastest for small crops but memory grows with the image (a 2048×1152 input needs gigabytes of activations on the card) and DirectML fails unpredictably on large inputs.
- Alternative: blending overlapping tiles. Not needed once each tile carries enough context; copying the centre is simpler and exact.

### ×2 is ×4 followed by a halving resize
×2 runs the same model and resizes its output by half with `Lanczos3`. This keeps one model, and halving the ×4 result also softens the model's slight oversharpening on UI edges.
- Alternative: the official `RealESRGAN_x2plus` (RRDB, 64 MB, slow) for ×2. Rejected: 14× the download, and RRDB reshaped diacritics in the A/B.

### Alpha enlarged separately
RGB goes through the model; alpha is resized to the output size with `Lanczos3` (the image crate's), then recombined. Pixels whose alpha is 0 still go through the model with their stored RGB; that is invisible.
- Alternative: run alpha through the model as a grey image. Triples the cost for no visible gain.

### Size limit checked at validation
`UpscaleTask::validated` reads only the PNG header (`image::image_dimensions`) and refuses `TOO_LARGE` when `max(w, h) * scale > 8192` (`MAX_SIDE`). 8192 keeps the decoded result at most 8192×8192×4 = 256 MB of RGBA and the PNG encode within a few seconds. `TOO_LARGE` joins `TASK_ERRORS` so the toast is translated.

### Output and progress
Output stem `<stem>_x2` / `<stem>_x4`, written through `queue::write_part(library::images(cfg), stem, "png", ..)`, so the `_N` rule and the `.part` cleanup come for free. Progress is 0-10 for the decode, 10-90 split evenly over the tiles, then the resize / alpha / encode. Cancel is checked before each tile and before the encode and returns `CANCELLED`.

### Placement: a new `Workload::Upscale`
`on_card` puts `Upscale` on the card for both CUDA and DirectML (a pure convolution net, like `Cutout`; DirectML was 28× faster than the CPU in the A/B), and `cpu_session` gives it the half-cores, no-spin setup used for `Ocr` and `Cutout`, so an upscale on the CPU never freezes other apps. The whole tile loop runs inside `onnx::retry_on_cpu(Workload::Upscale, ..)`, so a card failure restarts the upscale on the CPU and stores the error for AI models. The `places_workloads` test gains the new workload.

### Store: `upscale.rs`, shaped like `cutout.rs`
`app_local_data_dir()/upscale/realesr-general-x4v3/model.onnx`, `State.upscale_download`, events `upscale-progress` / `upscale-done` in `events.rs`, commands `upscale_status` / `upscale_download` / `upscale_cancel` / `upscale_delete` (refuses while downloading or `IN_USE`, then `onnx::release`). Every other store's delete command counts `upscale_download` as busy, and `onnx::release` keeps the CPU DLL while `upscale::installed`.
- Alternative: bundle the 4.6 MB model in the installer. Rejected: it still needs the downloaded ONNX Runtime, and every other model is opt-in.

### Frontend
`TaskSpec` gains `upscale { path, scale }` (`scale` 2 or 4, refused otherwise) and `lib/queue.ts` gains `upscaleSpec(path, scale)` and a `TASK_KINDS.upscale` entry. `use-gallery.ts` gets `upscaleAll(targets, scale)` beside `cutoutAll`, queueing one item per photo and showing each refusal through `taskError` without stopping the rest. The context menu uses `ContextMenuSub`; the viewer header and the bulk bar use a `DropdownMenu` (both already exist in `components/ui/`). `hooks/use-upscale.ts`, a `useUpscaleStore` in `models/stores.ts`, `models/upscale-card.tsx`, `about/upscale-notice.tsx` and `lib/upscale.ts` (name, id, licence, URL) mirror their cutout twins.

## Risks / Trade-offs

- [Model oversharpens UI edges at ×4 (slight halos, raised contrast seen in the A/B)] → ×2 halves it; the original is always kept, so the user can compare.
- [CPU-only machines: about 13 s per 1080p input at ×4] → it runs in the background queue with progress and Cancel; crops, the main use, take about 2 s.
- [The ONNX export comes from a third-party repo] → it is pinned by commit, size and SHA-256 and is BSD-3-Clause like the upstream weights; the A/B used this exact file.
- [Tiles on DirectML can still fail on low-memory cards] → `retry_on_cpu` finishes the task on the CPU and AI models shows the stored error.
- [The A/B covered one NVIDIA card and synthetic text downscaled with a box filter] → the ignored `renders_upscale` test (model, source, output, optional runtime and EP from env vars, like `renders_cutout`) lets the result be checked on other machines before release.

## Migration Plan

Additive. No config field is added, so existing configs load unchanged; the model downloads only when the user asks. Rollback is removing the kind; a stray model folder is deleted by Downloaded models or left harmlessly.
