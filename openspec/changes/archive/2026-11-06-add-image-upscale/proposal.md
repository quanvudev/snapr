# Proposal

## Why

Users crop a small part of a screenshot (a dialog, a line of text, an icon) to put in a slide or a document, and the result looks blurry or blocky once it is enlarged. A local super-resolution model can enlarge it cleanly on the computer. An A/B test on screenshots (synthetic UI text with ground truth plus real screenshot crops) picked `realesr-general-x4v3`: it kept every character correct, including Vietnamese diacritics and 9 px text, where the anime model and the larger x4plus model reshaped letters; it scored 26.1 dB PSNR (Lanczos 24.5 dB) at 4.6 MB and about 0.1 s per crop on DirectML.

## What Changes

- Upscale a gallery photo by ×2 or ×4, chosen by the user each time, as a new background task kind (`upscale`). The result is a PNG `<stem>_x2.png` / `<stem>_x4.png` in the `Images` folder (`_N` when taken); the original is never changed. ×2 runs the ×4 model and scales the result down by half.
- Transparency is kept: the alpha channel is enlarged separately with a classic filter, so padded screenshots stay transparent.
- A photo whose result would be longer than 8192 px on its longer side is refused before anything is queued, with a message.
- Entry points like Remove background, each with an ×2 / ×4 submenu: the photo context menu, the viewer header (a button with a menu) and the bulk action bar (applies to every selected photo, in visible order).
- A new model store for Real-ESRGAN General x4v3 (BSD-3-Clause, about 4.6 MB plus the shared ONNX Runtime): pinned to a fixed Hugging Face commit with SHA-256 and size, downloaded only when the user asks, on a new AI models card with a "Used for" row, listed in Downloaded models with Delete, refused while in use or downloading. Upscaling without the model is refused with the usual Open AI models action.
- The model runs on the graphics card with both DirectML and CUDA and falls back to the processor when the card fails, like background removal.
- About shows the model's licence once it is downloaded, like the other models.

## Capabilities

### New Capabilities
- `image-upscale`: enlarging gallery photos ×2 or ×4 with a downloadable local model, the output file, the size limit, the model store and its download.

### Modified Capabilities
- `gallery`: the viewer, the context menu and the bulk bar offer Upscale ×2 / ×4 for photos.
- `settings-window`: AI models shows the upscale model card and lists it in Downloaded models.
- `ai-acceleration`: the upscale model runs on the card with CUDA and with DirectML.

## Impact

- Backend: new `upscale.rs` (store) and `upscale/task.rs` (`UpscaleTask`), `TaskSpec::Upscale { path, scale }` in `queue.rs`, `Workload::Upscale` in `onnx.rs` (placement and CPU thread setup), `State.upscale_download`, two events, four commands in `lib.rs`, the other stores' delete commands and `onnx::release` count the new store. No new crate: `image`, `ndarray` and `ort` are already dependencies.
- Frontend: `lib/api.ts`, `lib/queue.ts` (`upscaleSpec`, `TASK_KINDS`, `TASK_ERRORS` for the size refusal), `hooks/use-upscale.ts`, the AI models card and stores, `hooks/use-gallery.ts` and the gallery item menu, viewer and bulk bar, the About notice, `lib/i18n.ts`.
- Download: about 4.6 MB from `huggingface.co/jonathanst29/tinier-upscale-models` (a BSD-3-Clause ONNX export of the official weights, verified against PyTorch by its author).
- Docs: `CLAUDE.md`.
