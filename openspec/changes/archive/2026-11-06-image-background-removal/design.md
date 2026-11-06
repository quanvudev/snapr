# Design

## Context

Snapr has three ONNX model stores (`paddle.rs`, `whisper.rs`, `translate.rs`). They share `fetch.rs` (pinned `Remote` with SHA-256 and size, `.part` then rename), `onnx.rs` (runtime DLL, `session(env, path, Workload)`, `gpu_for`, `retry_on_cpu`, `release`) and the queue (`Task` trait, `TaskSpec`, `write_part`). `translate.rs` is the closest template: one model, one download slot in `State`, `*-progress` / `*-done` events, `*_status` / `*_download` / `*_cancel` / `*_delete` commands, and an `IN_USE` counter that blocks delete while a task runs. The gallery already routes model refusals through `needsModels` (`MODEL_MISSING`) to an "Open AI models" toast action.

Model facts, checked on Hugging Face and by running it: `PramaLLC/BEN2` at commit `e48a20765fb421d19dcdb0bf3cc61e802ca5ec8f`, MIT licence, `BEN2_Base.onnx`, 222 932 053 bytes, SHA-256 `22cea62108ff53b7ccc20f7a008bf30494228d84b1687f29ecbe76936a998101`. Input `input.1` f32 [1,3,1024,1024]: a 1024x1024 resize, /255 and ImageNet mean/std, as in the official PyTorch `BEN2.py`. The repo's `onnx_run.py` skips the normalisation; that is wrong and breaks UI screenshots. The first output is f16 [1,1,1024,1024], already sigmoided, and gets the official min-max rescale.

A/B against BiRefNet lite (MIT, 224 MB) on five photos and two UI screenshots: BEN2 was equal or better on every photo (it kept the whole body of a long-haired cat that BiRefNet cut off, and gave crisper edges on a person). Neither model is usable on UI screenshots. Inference per image: BiRefNet lite 4.1 s on the CPU and 4.4 s on DirectML (it gains nothing on the card), BEN2 5.8 s on the CPU and 0.33 s on DirectML.

## Goals / Non-Goals

**Goals:**
- Reuse the existing store, queue, ONNX and gallery patterns, adding no new abstractions.
- Produce good edges on both UI screenshots and photos, with alpha at the source resolution.

**Non-Goals:**
- Webcam segmentation (a separate change, done in the webview).
- Background removal in the Draw overlay or the toast.
- Replacing the background with a colour or image, and manual mask refinement.
- Supporting more than one model size.

## Decisions

- **BEN2 Base over BiRefNet lite** (see the A/B above). It is a single file that runs on the CPU, DirectML and CUDA; it casts to fp16 internally, so `ort` gets the `half` feature to read the f16 output. Alternatives that are better on paper were rejected: RMBG-2.0 (non-commercial and gated, so anonymous downloads get 401), ISNet / imgly (AGPL), and full BiRefNet (490 MB fp16, about 3-4x slower; a possible "high quality" option later).
- **New module `cutout.rs`, a near copy of `translate.rs`.** It holds `STORE_DIR` `cutout`, `MODEL_DIR` `ben2-base`, one `Remote`, `ready` / `installed` / `model_dir`, `State.cutout_download`, `IN_USE` + `acquire`, the events `cutout-progress` / `cutout-done` (constants in `events.rs`) and the commands `cutout_status` / `cutout_download` / `cutout_cancel` / `cutout_delete`. There is no licence gate, because MIT needs no acceptance dialog. The download also runs `onnx::ensure` for the CPU runtime. `onnx::release` adds `cutout::installed`, and its callers pass the cutout download slot in `busy`. The alternative, a generic store abstraction, was rejected under "no speculative abstractions".
- **`cutout/task.rs`: `CutoutTask { path, target }`, kind `cutout`, `TaskSpec::Cutout { path }`.** `validated` runs `gallery::guard`, requires a `png` extension (the gallery lists png and mp4 only), refuses `MODEL_MISSING` when the model is not ready, and fixes `onnx::target(app)` at queue time like the other tasks. `run`:
  1. `acquire()`, `onnx::init(&target)`, decode the PNG to RGBA (progress 10).
  2. Resize to 1024x1024 with `Triangle`, ignoring the aspect ratio as the official code does, and build the ImageNet-normalised NCHW tensor for `input.1`. Alpha is ignored, because screenshots are opaque.
  3. `onnx::retry_on_cpu(Workload::Cutout, ..)` runs the session (progress 80). Cancel is checked before and after, since a single run cannot be interrupted.
  4. Convert the f16 output to f32, apply a min-max rescale to [0,1], resize the 1024x1024 mask to the source size with `Triangle`, then set alpha to `mask * source alpha`.
  5. Write through `queue::write_part(library::images(cfg), "<stem>_cutout", "png", ..)`, which gives the `_N` suffix, the `.part` and cleanup on error or cancel. `PART_EXTS` (`gif`, `mp4`, `srt` today) gains `png`, so a leftover `.png.part` is swept at startup.

  The session is dropped after each task, as Whisper's is, so 223 MB of weights do not stay resident.
- **`Workload::Cutout` on the GPU for both CUDA and DirectML.** BEN2 runs well on DirectML (0.33 s inference against 5.8 s on the CPU), and its mask matches the CPU one within 3/255. `gpu_for` gets `(DirectMl, Cutout)`. If DirectML gives a visibly wrong mask in the verification task, fall back to CUDA only, which changes only one line and the ai-acceleration scenario. CPU threads follow the `Ocr` workload's half-cores, no-spin setup so a cutout never freezes the desktop.
- **Frontend.**
  - `hooks/use-cutout.ts` mirrors `use-translate.ts` (`cutout` in the settings context).
  - `models/stores.ts` gains `useCutoutStore`.
  - `models/cutout-card.tsx` is a `ModelCard` with no options, plus a `model-usage-row` and the MIT link.
  - `downloaded-models.tsx` lists it.
  - `lib/queue.ts` gains `TASK_KINDS.cutout` and `cutoutSpec(path)`.
  - The gallery gets a viewer header button (lucide `Eraser`, or the closest existing icon), an `item-menu.tsx` entry and a `bulk-bar.tsx` action. All three call `queue_add`, and refusals go through `taskError`.
  - The `about/` notices list the MIT model.
  - `TASK_ERRORS` already maps `MODEL_MISSING`.

## Risks / Trade-offs

- [The 1024x1024 stretch blurs thin details on very wide screenshots (for example 3840x1080)] → The mask is upscaled with soft edges, which is acceptable for a first version. Tiling can come later.
- [The model does not work on UI screenshots, which are most of Snapr's photos] → The card description says it suits photos of people, pets and objects. Screenshots are not blocked, because detecting them is not worth the complexity.
- [CPU inference takes about 6 to 8 s and uses a lot of memory (around 1.5 GB peak)] → It runs in the queue with progress and cancel, and the session is freed after each task.
- [DirectML output may differ from the CPU] → A verification task compares the masks, and the fallback plan is above.
- [223 MB download] → It happens only when the user clicks Download, and the size is shown on the card.

## Migration Plan

This is additive. No config field is needed, because there are no settings beyond the model being present. Removing the feature means deleting the store folder through Delete.
