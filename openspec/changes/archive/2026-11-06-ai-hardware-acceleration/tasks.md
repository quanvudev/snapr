# Tasks

## 1. Runtime spikes

- [x] 1.1 Load the `onnxruntime-directml` 1.24.4 `onnxruntime.dll` through `ort::init_from` with `ort` rc.15 (`directml` feature on) and build one Whisper Small session on the DML EP. Verify with an `#[ignore]` test (`SNAPR_ORT_DLL`) that runs the encoder. If the DLL is refused, apply the fallback in design.md (Risks) and record the outcome in design.md.
- [x] 1.2 Do the same with the `onnxruntime-gpu` 1.31.1 DLL and the CUDA EP (`cuda` feature), loading the CUDA libraries from one folder through `AddDllDirectory`. Verify the Whisper Small fp16 encoder runs on CUDA from a folder outside the Python venv.
- [x] 1.3 Walk the dependencies of `onnxruntime_providers_cuda.dll` (`dumpbin /dependents`, recursively) and write down the minimal list of DLLs and the `nvidia-*` wheels that hold them, with each wheel's URL, SHA-256 and size. Verify that the 1.2 test still passes with only those DLLs in the folder, and record the total size in design.md D1.

## 2. Batched HY-MT translation

- [x] 2.1 Add `Engine::complete_batch` in `translate/engine.rs` (left padding, masks, per-row positions, per-row repetition penalty, EOS and limit, cancel between steps) per design D4. Verify that `cargo clippy` is clean.
- [x] 2.2 Make `HymtTask::run` translate non-empty cues in batches of `BATCH = 8` in transcript order, keep empty cues empty and set progress after each batch. Verify that a 40-cue transcript reports 20% after the first batch.
- [x] 2.3 Add an `#[ignore]` engine test that translates 20 real cues one by one and in batches. Verify at least 19 of 20 are identical and none contains another cue's text. If not, implement the length-bucketing fallback from D4 and re-run.
- [x] 2.4 Time a 100-cue transcript on the CPU before and after batching in a release build. Verify the batched run is faster, and record both times in design.md.
- [x] 2.5 Update the CLAUDE.md `translate.rs + translate/` entry (batched requests, `BATCH`). Verify it no longer says one cue per request without mentioning batching.

## 3. Device detection, config and runtime store

- [x] 3.1 Add `gpu.rs` (DXGI enumeration, filters and choice per D5) and the `Win32_Graphics_Dxgi` feature. Verify on this machine that it returns the RTX 5060 Ti with about 16 GB, and skips the Radeon iGPU and the MuMu adapter.
- [x] 3.2 Add `ai_acceleration` to `config.rs` with `Config::acceleration()`. Verify a config without the key loads as Auto and an unknown value maps to Auto.
- [x] 3.3 Extend `onnx.rs` with `Runtime`, the pinned DirectML and CUDA file sets from 1.1 to 1.3, `ready`, `resolve(app)` (setting, Auto rules, detection) and download / extract / delete through `fetch::Fetch` into `runtime/<id>/`. Verify a DirectML download, a cancel (no `.part` left) and a corrupted-hash failure (nothing kept).
- [x] 3.4 Make `onnx::init` load the resolved runtime once, record it as `loaded` and add the runtime folder to the DLL search path for CUDA. Verify that with DirectML selected and downloaded, the first PaddleOCR call loads `runtime/directml/onnxruntime.dll` (Process Explorer or a log line in debug builds).
- [x] 3.5 Update CLAUDE.md (`onnx.rs`, new `gpu.rs`, config `ai_acceleration`). Verify the entries describe the runtime folders and the one-runtime rule.

## 4. Sessions on the GPU

- [x] 4.1 Add `onnx::session(env, path, Workload)` per D2 (EP per runtime and workload, `error_on_failure`, CPU retry, `State.acceleration_error` set and cleared). Move `ocr/paddle.rs`, `transcript/decode.rs` and `translate/engine.rs` onto it. Verify that `cargo clippy` is clean and that CPU behaviour is unchanged (Paddle keeps half the cores).
- [x] 4.2 Verify the fallback: with CUDA selected but `onnxruntime_providers_cuda.dll` renamed away, a transcription finishes on the CPU and `acceleration_status` reports the error. Restoring the file and running another task clears it.
- [x] 4.3 Keep HY-MT on the CPU EP under DirectML and on the CUDA EP under CUDA. Verify with a 20-cue translation on each runtime (Task Manager GPU graph or EP log).
- [x] 4.4 Add IoBinding for the Whisper decoder and HY-MT loops on GPU EPs (D3). Measure per-token time with and without binding on DirectML and CUDA, keep the faster per EP and record the numbers in design.md.
- [x] 4.5 Run PaddleOCR text capture on DirectML for five screenshots of different sizes. Measure the first and repeated runs, and if the first run per shape is more than 3x the repeat, add the padding to a multiple of 256 from the Risks section. Verify the recognized text is unchanged against the CPU.
- [x] 4.6 Add the Whisper GPU variants to `whisper.rs` (pinned fp16 / q4f16 pairs, `encoder.gpu.onnx` / `decoder.gpu.onnx`, GPU pair on the card only with CUDA, else the CPU pair on the CPU, GPU download through the Whisper slot with a `gpu` flag, delete of the GPU pair only) per D6. Verify that with CUDA loaded and the Small variant downloaded a transcription uses the `.gpu.onnx` files on the card, and that without it the CPU files run on the CPU.
- [x] 4.7 Transcribe the same real 5-minute recording with Turbo on CPU, DirectML and CUDA. Verify all three produce usable transcripts in the same language, and record the times in design.md.
- [x] 4.8 Update CLAUDE.md (`onnx::session`, the workloads, Whisper GPU variants, the IoBinding choice). Verify `whisper.rs` and `ocr/paddle.rs` entries no longer describe their own session builders.

## 5. Commands, events and restart

- [x] 5.1 Add `acceleration_status`, `acceleration_download`, `acceleration_cancel` and `acceleration_delete` with `State.runtime_download` and the `acceleration-progress` / `acceleration-done` events in `events.rs`. Register them in `lib.rs`. Verify with the frontend dev tools that each returns or emits the documented shape.
- [ ] 5.2 Make delete refuse the loaded runtime and the downloading one. Verify both refusals return an error code and leave the files.
- [ ] 5.3 Add `acceleration_restart`: refuse `RECORDING` and `QUEUE_BUSY`, otherwise spawn with `--after <pid>` and exit. Reuse `update::handoff`. Verify a restart keeps the hotkeys working and loads the newly selected runtime, and that a restart during a recording is refused.
- [x] 5.4 Update CLAUDE.md (commands, events, restart). Verify the Settings window ↔ backend command list includes them.

## 6. Settings UI

- [x] 6.1 Add `acceleration` types and wrappers to `lib/api.ts` and `hooks/use-acceleration.ts` (status, listeners, actions, download after a save that selects a runtime that is not ready), exposed in the settings context. Verify `bunx tsc --noEmit` passes.
- [ ] 6.2 Add `settings/models/acceleration-card.tsx` first on AI models: detected card and memory, the Auto / CPU / DirectML / CUDA ToggleGroup (CUDA only for NVIDIA, GPU options disabled with no usable card), "In use" runtime, status line, GPU error, "Applies after restart" with Restart snapr and inline refusal reasons. Verify each settings-window scenario by hand in `bun run tauri dev`.
- [x] 6.3 Start the Whisper GPU variant download from `use-settings` per D6, and show it on the Whisper card through the existing status line. Verify that selecting CUDA with Small ready starts the variant download.
- [ ] 6.4 List the GPU runtimes and the Whisper GPU variants in `DownloadedModels` with sizes, Delete and the total, with Delete disabled for the loaded runtime. Verify by downloading and deleting the DirectML runtime and the Small variant.
- [x] 6.5 Add en/vi strings to `lib/i18n.ts` for the card, errors (`RECORDING`, `QUEUE_BUSY`, loaded-runtime delete) and list entries. Verify both languages render without missing keys.
- [x] 6.6 Update the CLAUDE.md AI models paragraph (acceleration card, hook, downloaded list). Verify it matches the components.

## 7. Integration checks

- [ ] 7.1 On a clean profile with no GPU runtime: transcribe, translate and use PaddleOCR. Verify everything runs on the CPU, nothing GPU-related downloads and translation is batched.
- [ ] 7.2 On this machine, select CUDA, download, restart, then transcribe with Turbo, translate 100 cues and run PaddleOCR Medium text capture. Verify each uses the GPU and is at least 3x faster than the CPU baseline from 2.4 and 4.7.
- [ ] 7.3 Switch to DirectML and restart. Verify PaddleOCR uses the GPU, Whisper and HY-MT use the CPU (with a correct transcript), and deleting CUDA works while deleting DirectML is refused.
- [x] 7.4 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`. Verify both are clean.
- [x] 7.5 Run `openspec validate ai-hardware-acceleration --strict`. Verify it passes.

## Workflow follow-up

- Archive the change after review with `/opsx:archive`.
