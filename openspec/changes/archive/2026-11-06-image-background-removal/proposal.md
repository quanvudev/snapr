# Proposal

## Why

Screenshots of products, people or UI elements often need the background removed before they go into slides, docs or chats, and today that means opening another tool or a website. Snapr already has a local ONNX stack (runtime, GPU acceleration, pinned model downloads, a background task queue), so a local background remover fits naturally.

## What Changes

- A new AI model store for **BEN2 Base** (`PramaLLC/BEN2`, MIT licence, `BEN2_Base.onnx`, 223 MB), pinned to commit `e48a207` with SHA-256 and size. It was chosen over BiRefNet lite after an A/B test: equal or better on every photo, and about 13x faster on DirectML. It is downloaded on request from the AI models page and is never downloaded automatically.
- A new queue task kind `cutout`. It reads a gallery photo, predicts a foreground mask and writes `<stem>_cutout.png` with a transparent background into `Images` (`_N` when that name is taken). The original is never touched.
- In the gallery, a **Remove background** button in the viewer header and an item menu entry, both for photos only. When the model is missing they show the existing "Open AI models" error toast instead of queueing anything.
- AI models gets a Background removal card (description, size, licence, status line, "Used for" row), and Downloaded models lists it.
- The model runs on the GPU with both CUDA and DirectML, and falls back to the CPU like the other models.
- The bulk bar gains Remove background for selected photos (one task each).

Out of scope: webcam background blur or replacement (a separate change), and background removal inside the Draw overlay.

## Capabilities

### New Capabilities
- `background-removal`: the cutout task (input, output name and location, transparency, model download, deletion, licence, GPU placement and fallback).

### Modified Capabilities
- `gallery`: the viewer header button, item menu entry and bulk bar action for photos; Item context menu gains Remove background.
- `settings-window`: AI models shows the background removal card, and Downloaded models lists it.
- `ai-acceleration`: Models used on the GPU now includes BEN2 on both CUDA and DirectML.

## Impact

- Backend: new `cutout.rs` (model store, shaped like `translate.rs`) and `cutout/task.rs` (`CutoutTask`), `TaskSpec::Cutout { path }` in `queue.rs`, `Workload::Cutout` in `onnx.rs` (`gpu_for`), `onnx::release` also checks `cutout::installed`, new `State.cutout_download`, the events `cutout-progress` / `cutout-done` in `events.rs`, and commands registered in `lib.rs`.
- Frontend: `lib/api.ts`, `hooks/use-cutout.ts`, `models/stores.ts` (`useCutoutStore`), a new `models/cutout-card.tsx`, `downloaded-models.tsx`, `about/` licence notice, gallery `viewer.tsx`, `item-menu.tsx`, `bulk-bar.tsx`, `lib/queue.ts` (`TASK_KINDS.cutout`, `cutoutSpec`), `lib/i18n.ts`.
- Disk: about 223 MB once downloaded.
- `CLAUDE.md` module list.
