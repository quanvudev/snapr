# Proposal

## Why

Every local AI task (PaddleOCR, Whisper transcription, HY-MT subtitle translation) runs on the CPU only: `onnx.rs` downloads the CPU-only `onnxruntime` wheel, so a graphics card is never used, and HY-MT translates one cue per model run. Measured on a Ryzen 7 7800X3D + RTX 5060 Ti (benchmarks in `design.md`), a 5-minute recording takes about 47 s to transcribe with Large v3 Turbo and about 128 s to translate 100 cues, while the same work takes about 4 s and 17 s on that machine's GPU with the batched loop below. Users on fast machines see slow AI and the hardware sits idle.

## What Changes

- HY-MT translates cues in batches of up to 8 per model run instead of one by one, on every device. Each cue still gets its own prompt with no text from other cues, so translations stay the same as translating each cue alone. About 1.5x faster on CPU (100 cues: 67.7 s → 44.7 s) and the main speed-up on GPU.
- A new **Hardware acceleration** setting (Auto / CPU / DirectML / CUDA, default Auto) on the AI models page, with the detected graphics card shown. Auto uses the best runtime that is already downloaded; nothing downloads unless the user picks a GPU runtime.
- snapr detects the graphics cards (DXGI). It skips software and virtual adapters and adapters with little dedicated memory, then recommends CUDA for NVIDIA cards and DirectML for other discrete cards.
- Two optional GPU builds of ONNX Runtime, downloaded on request with progress, cancel, SHA-256 verification and delete:
  - DirectML: about 40 MB, any DirectX 12 card.
  - CUDA: about 1.3 GB on disk (1.0 GB download), NVIDIA only, and the fastest option for all three models.
- snapr loads one ONNX Runtime build per process. A change takes effect at once if no AI task has run yet in this session; otherwise the page offers **Restart snapr**, which is refused while recording, counting down or while a queue task is running.
- What runs on the GPU:
  - With CUDA, PaddleOCR, Whisper and HY-MT all run on the card.
  - With DirectML, only PaddleOCR does. Whisper produced garbage with every variant on DirectML 1.24.4 (int8, fp16, q4, q4f16), and HY-MT measured 4x slower with its q4f16 variant crashing, so both stay on the CPU.
- Whisper gets a GPU variant per model (fp16 for Base and Small, q4f16 for Turbo), used only with CUDA:
  - It downloads when CUDA is active or selected and the selected model is ready.
  - Until the variant is ready, the existing CPU files run on the CPU.
  - Downloaded models lists the GPU variants with Delete.
- If the GPU provider cannot start (old driver, missing CUDA libraries, device lost), the task runs on the CPU of the same runtime and the AI models page shows the error.

## Capabilities

### New Capabilities
- `ai-acceleration`: device detection, the hardware acceleration setting, the downloadable GPU runtimes, the one-runtime-per-process rule with restart, which models use the GPU, the Whisper GPU variants and the CPU fallback.

### Modified Capabilities
- `subtitle-translation`: adds batched translation. The existing "Each cue is translated on its own" requirement keeps holding, and a new requirement states that batching never changes a cue's translation and that progress and cancel still work.
- `settings-window`: AI models shows the hardware acceleration card, and Downloaded models lists the GPU runtimes and Whisper GPU variants.

## Impact

- Backend:
  - `onnx.rs` gains several runtimes, resolution of the active runtime, session building with an execution provider per workload, and a status.
  - New `gpu.rs` for DXGI detection.
  - `ocr/paddle.rs`, `transcript/decode.rs` and `translate/engine.rs` build sessions through the shared builder.
  - `translate/engine.rs` and `translate/task.rs` get the batched decode loop.
  - `whisper.rs` gets GPU variants.
  - Config gets `ai_acceleration`.
  - New commands `acceleration_status`, `acceleration_download`, `acceleration_cancel`, `acceleration_delete` and `acceleration_restart`, with events `acceleration-progress` and `acceleration-done`.
  - The restart reuses `update.rs`'s `--after <pid>` handoff.
- Dependencies:
  - `ort` adds the `directml` and `cuda` features (still `load-dynamic`).
  - `windows` adds `Win32_Graphics_Dxgi`.
  - New pinned downloads: the `onnxruntime-directml` 1.24.4 wheel, and the `onnxruntime-gpu` 1.31.1 wheel plus the CUDA 13 / cuDNN wheels it needs.
- Frontend:
  - `settings/models/` gets an acceleration card and a store hook (`use-acceleration.ts`), plus Downloaded models entries.
  - `lib/api.ts` types and i18n strings.
- Disk: up to about 1.3 GB for CUDA plus 0.1 to 0.6 GB per Whisper GPU variant, only after the user opts in.
- No change for users who never pick a GPU runtime, apart from batched translation.
