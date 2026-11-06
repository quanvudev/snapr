# Design

## Context

- `onnx.rs` downloads one CPU-only `onnxruntime` 1.31.1 wheel. It extracts `onnxruntime.dll` to `paddle/onnxruntime.dll` and calls `ort::init_from` once per process (`OnceLock`). The `ort` crate (`=2.0.0-rc.15`, `load-dynamic`) cannot load a second runtime DLL in the same process.
- Sessions are built in three places, each with its own settings:
  - `ocr/paddle.rs`: half the logical cores, spinning off.
  - `transcript/decode.rs::session`: defaults.
  - `translate/engine.rs::Engine::load`: defaults.
- HY-MT (`translate/task.rs`) calls `Engine::complete` once per cue: a prefill, then one decoder run per generated token, batch 1.

Benchmarks were run on a Ryzen 7 7800X3D and an RTX 5060 Ti 16 GB with Python `onnxruntime` 1.31.1 (CPU and CUDA 13) and `onnxruntime-directml` 1.24.4. The inputs were random, and the KV cache stayed on the device through IoBinding. A Whisper window is the encoder run plus 120 tokens.

| Workload | CPU (today) | DirectML | CUDA |
|---|---|---|---|
| Whisper Small, one 30 s window | 2.5 s (int8) | 1.46 s (fp16), 1.81 s (int8) | 0.70 s (fp16), 2.16 s (int8) |
| Whisper Turbo, one 30 s window | 4.66 s (q4) | 0.84 s (q4f16), 0.97 s (q4) | 0.42 s (q4f16), 0.54 s (q4) |
| PaddleOCR Medium, detect 1080p + one 8-line rec batch | 709 + 308 ms | 67 + 31 ms | 66 + 20 ms |
| HY-MT q4, per token, batch 1 | 40 ms | 222 ms | 21 ms |
| HY-MT q4, per cue at batch 8 (35-token prompt, 25 tokens out) | 0.66 s | — | 0.17 s |
| HY-MT q4f16 | — | process crash `0xC0000409` | 40 ms per token (slower than q4) |

Batching HY-MT helps on every device, because decoding is limited by memory bandwidth and per-run overhead rather than compute. On the CPU a cue takes 1.28 s alone, 0.66 s at batch 8 and 0.50 s at batch 16. On CUDA it takes 0.60 s alone, 0.17 s at batch 8 and 0.12 s at batch 32. The int8 Whisper files are slow on CUDA, because the int8 operators fall back to slow kernels, so CUDA needs fp16 or q4f16 files.

## Goals / Non-Goals

**Goals:**
- One place (`onnx.rs`) decides the runtime, the execution provider for each workload and the CPU fallback. The model modules only say which workload they are.
- Batched HY-MT decoding that does not change which tokens a cue gets.
- No download and no behaviour change until the user picks a GPU runtime, except batching.

**Non-Goals:**
- TensorRT, OpenVINO, Windows ML or onnxruntime-genai. These are possible later; DirectML and CUDA cover NVIDIA, AMD and Intel today.
- Running several runtimes in one process, or switching the runtime without a restart once one is loaded.
- GPU builds of PaddleOCR or HY-MT models. Their existing files are used as they are.
- Changing which OCR engine redaction prefers. Windows OCR stays first.

## Decisions

### D1. Three runtime builds, one loaded per process
- `Runtime { Cpu, DirectMl, Cuda }`, each with a pinned file set and a folder under `app_local_data_dir()`:
  - Cpu stays at `paddle/onnxruntime.dll`, so existing installs keep working.
  - DirectML goes in `runtime/directml/`: `onnxruntime.dll`, `onnxruntime_providers_shared.dll` and `DirectML.dll` from `onnxruntime_directml-1.24.4-cp313-cp313-win_amd64.whl` (25.1 MB download, 39.7 MB on disk). `ort` rc.15 without `api-*` features requires C API 17, so the 1.24.4 DLL loads (task 1.1: Whisper Small encoder 57 ms on the DML EP).
  - CUDA goes in `runtime/cuda/`, from four pinned wheels (about 1.0 GB download, 1.28 GB on disk):
    - `onnxruntime_gpu-1.31.1-cp313-cp313-win_amd64.whl` (168 MB): `onnxruntime.dll`, `onnxruntime_providers_shared.dll`, `onnxruntime_providers_cuda.dll`. TensorRT is left out.
    - `nvidia_cublas-13.10.0.20-py3-none-win_amd64.whl` (427 MB): `cublas64_13.dll`, `cublasLt64_13.dll`.
    - `nvidia_cuda_runtime-13.5.37-py3-none-win_amd64.whl` (2.8 MB): `cudart64_13.dll`.
    - `nvidia_cudnn_cu13-9.28.0.42-py3-none-win_amd64.whl` (406 MB): `cudnn64_9.dll`, `cudnn_graph64_9.dll`, `cudnn_ops64_9.dll`, `cudnn_cnn64_9.dll`, `cudnn_heuristic64_9.dll`, `cudnn_engines_precompiled64_9.dll`, `cudnn_engines_runtime_compiled64_9.dll`, `cudnn_engines_tensor_ir64_9.dll`.
- How the CUDA list was fixed (task 1.3):
  - `onnxruntime_providers_cuda.dll` imports only cuBLAS / cuBLASLt statically and loads cuDNN by name at run time. cuFFT, cuRAND, nvrtc, nvJitLink, `cudnn_adv` and `cudnn_ext` are not needed for these models.
  - Without `cudnn_engines_tensor_ir` or `cudnn_engines_runtime_compiled`, Conv silently runs about 50x slower (Turbo encoder 4970 ms instead of 89 ms). Without `cudnn_engines_precompiled` it fails with `CUDNN_STATUS_INTERNAL_ERROR`.
  - With the list above, the Turbo q4f16 encoder took 89 ms, PaddleOCR Medium detection 66 ms and the HY-MT token 23 ms, the same as the full venv.
- Every wheel is a pinned `fetch::Remote` (URL, SHA-256, size). Only the listed entries are extracted, through `.part` files and a rename. The URLs and digests live in `onnx.rs`.
- `onnx::init` loads from the runtime resolved at the first AI task (`onnx::resolve(app)`: the setting, Auto rules, and what is downloaded and detected). It records which one was loaded in a `OnceLock<Runtime>`.
- Before `init_from` of a GPU runtime, `SetDllDirectoryW(runtime folder)` puts that folder in the standard DLL search order:
  - `DirectML.dll` is then taken from the folder instead of the older `System32` copy.
  - cuDNN, which ORT loads by bare name, is found too.
  - One process loads one runtime, so a process-wide setting is fine. It replaced the `AddDllDirectory` plan because ORT's own `LoadLibrary` calls do not pass the user-dirs flag.
- Alternative considered: loading GPU EPs as plugin libraries next to the CPU build. ORT 1.31's EP plugin API is not supported by `ort` rc.15 and DirectML has no plugin build, so this was rejected.

### D2. Workload-aware session builder
- `onnx::session(env, path, Workload) -> Result<Session, String>`, with `Workload { Ocr, WhisperEncoder, WhisperDecoder, Translate }`, replaces the three builders.
- CPU: today's settings. Ocr keeps half the cores and no spinning; the others keep the defaults.
- DirectML: mem pattern off, sequential execution, DML EP on the DXGI adapter index from detection, for Ocr only. Whisper and Translate get the CPU EP of the DirectML build (`gpu_for(workload)` is the one rule):
  - Whisper gives garbage on DirectML 1.24.4 with every variant, measured on the RTX 5060 Ti. The int8 encoder differs from the CPU by up to 26 even with graph optimisations off. The fp16 files fail to initialise in Python (`GetIndexFromName`) and give garbage in Rust. The Turbo q4 / q4f16 transcripts were garbage end to end. Only the fp32 encoder matched (0.002), but fp32 is about 1 GB per model and was not pursued.
  - PaddleOCR Medium on DirectML matches the CPU (max diff 1.6e-7 detection, 1.2e-5 recognition).
  - HY-MT measured 4x slower per token on DirectML, and q4f16 crashed.
- CUDA: CUDA EP on device 0 (the CUDA ordinal of the detected NVIDIA card; with one NVIDIA card it is 0), every workload.
- The EP is registered with `error_on_failure()`. If registering or committing the session fails, the builder retries on the CPU EP of the same runtime, stores the error in `State.acceleration_error` and returns the CPU session. A success on the GPU clears the error.
- Alternative considered: `ort`'s silent CPU fallback. Rejected, because the user would never learn why the GPU is idle.

### D3. KV cache stays on the device for Whisper only
- `onnx::Runner` (from `onnx::runner(env, path, workload)`) wraps a session.
  - On the CPU, and for HY-MT, `run(feed, host)` is a plain `session.run`.
  - For Whisper on a GPU session, it uses an `IoBinding`. Inputs are bound as given (`Feed::Owned` / `Feed::Ref`), outputs named in `host` (`logits`) go to CPU memory, and every other output (`present.*`, the encoder's `last_hidden_state`) stays in device memory. The encoder hidden state and the cross-attention cache are then fed back without copying.
- Measured on the 8-minute `tst.mp4` (482 s of Vietnamese speech) in release builds, task 4.4:

  | Whisper | CPU (int8 / q4) | CUDA without binding | CUDA with binding |
  |---|---|---|---|
  | Small (fp16 on CUDA) | 96–110 s | 76.7 s | 28.3 s |
  | Large v3 Turbo (q4f16 on CUDA) | 121.3 s | 42.7 s | 16.1 s |

  - Without binding, each token copied the 74 MB Small cross-attention cache and the hidden state host to device.
  - Segments and text were the same with and without binding (129 / 199 segments).
- HY-MT on CUDA was slower with binding: 100 cues took 8.6 s against 6.5 s without. Its KV cache is small (4 KV heads) and the per-run binding cost dominates, so `runner` binds only for `Workload::Whisper`.
- DirectML runs only PaddleOCR, which uses plain sessions, so DirectML never binds.

### D4. Batched HY-MT
- `Engine::complete_batch(requests: &[(&str, usize)], cancel) -> Vec<String>` replaces `complete`; a single cue is a batch of one.
- `engine::batches(tokens, BATCH)` sorts the non-empty cues by source token count and cuts them into groups of `BATCH = 8`. `HymtTask::run` translates each group and writes each reply back to its cue's index. Empty cues stay empty without a model run, and progress counts finished non-empty cues.
- Sorting matters because a batch runs until its longest row ends. Without sorting, 20 real cues took 13.5 s at batch 1 and 13.5 s at batch 8. Sorted, they took 11.1 s, and 100 cues went from 67.7 s to 44.7 s (1.5x) on the CPU in a release build. The order never changes a result, since each row is independent.
- Padding is on the **right**, because ORT's `GroupQueryAttention` derives `seqlens_k` from the sum of `attention_mask` and keeps each row's valid tokens at the start of `present`:
  - Prefill: each `BOS USER … ASSISTANT` prompt is padded to the longest with the pad id 120002. `attention_mask` is 1 on real tokens and 0 on padding. Positions are `0..len` with the padding repeating `len - 1`. The logits of each row are taken at `len - 1`.
  - Decode: one token per row. The mask is `length + 1` ones then zeros over the cache width plus one, and the position is that row's real length.
  - Left padding was measured to break GQA (garbage translations).
- Each row keeps its own repetition-penalty `seen` set and output limit. A row that emitted EOS or hit its limit is fed EOS and its tokens are ignored. The batch ends when every row is done.
- Correctness (task 2.3): right padding gave the same greedy text as batch 1 on CPU and CUDA for 6 cues in Python, and in Rust for 20 of 20 real cues (`translate::engine::tests::batches_match_single_cues`).
- Batch size is 8 on every runtime. 16 is about 25% faster per cue on the CPU and 32 about 30% faster on CUDA, but larger batches hold more cues back from progress and use more memory with long cues. One constant keeps it simple and can be raised later without a spec change.
- The prefill logits are `[rows, width, vocab]` (about 230 MB at 8 × 60 tokens). Only one row slice per row is copied out.

### D5. Device detection
- New `gpu.rs`: `CreateDXGIFactory1` → `EnumAdapters1`, skipping `DXGI_ADAPTER_FLAG_SOFTWARE` and adapters with `DedicatedVideoMemory` < 2 GiB. A virtual display adapter reports 0 and is skipped by that rule.
- It returns `Adapter { index, name, vendor, memory }`. The chosen adapter is the one with the most memory. `vendor == 0x10DE` makes it NVIDIA.
- The result is cached in `State` at first use and refreshed when the AI models page opens (`acceleration_status`).
- Alternative considered: NVML or `nvidia-smi` for NVIDIA detection. Rejected because DXGI already covers it.

### D6. Whisper GPU variants
- `MODELS` gains a second file pair per model, pinned at the same commits: Base and Small `encoder_model_fp16.onnx` and `decoder_model_merged_fp16.onnx`, Turbo the `_q4f16` pair. They are stored as `encoder.gpu.onnx` and `decoder.gpu.onnx` in the same model folder, so delete-model removes both and the JSON files are shared.
- `decode::Model::load` uses the GPU pair on the card when `onnx::gpu_for(Whisper)` is set (only with CUDA) and the pair exists. Otherwise it loads the CPU pair on the CPU EP (`onnx::on_cpu`), because the int8 files are no faster on CUDA (Small encoder 394 ms against 484 ms on the CPU) and were not checked for correctness there.
- The onnx-community fp16 and q4f16 files keep float32 inputs and outputs, including `past_key_values`, so the decoder code is unchanged.
- The variant download reuses `State.whisper_download` with `Download.gpu = true` and the existing `whisper-progress` and `whisper-done` events, with a `gpu` flag. Delete can remove the GPU pair only.
- The frontend starts the variant download after a save that makes CUDA the resolved runtime (selected and downloaded) while the selected model is ready, after the CUDA runtime download finishes, or after a model change while CUDA is resolved and the model is ready. This is the same rule `use-settings` already applies to model downloads.

### D7. Config, commands, events
- Config: `ai_acceleration: "auto" | "cpu" | "directml" | "cuda"`, `#[serde(default)]` = auto, read only through `Config::acceleration()` (unknown → Auto).
- Commands:
  - `acceleration_status` returns `{ adapter, recommended, selected, resolved, loaded, restart_needed, runtimes: [{ id, ready, bytes, downloading }], progress, error }`.
  - `acceleration_download(runtime)`, `acceleration_cancel` and `acceleration_delete(runtime)`. Delete refuses the loaded runtime and the downloading one.
  - `acceleration_restart` refuses `RECORDING` while `State.recording` or `State.counting`, and `QUEUE_BUSY` while the queue has waiting or running items. It then spawns the exe with `--after <pid>`, the same handoff `update::handoff` uses, then `cleanup_before_exit` and `exit`.
- Download slot: `State.runtime_download` (`Download { runtime, cancel, done, total }`), independent of the Paddle, Whisper and HY-MT slots.
- Events: `acceleration-progress { runtime, done, total }` (throttled 100 ms) and `acceleration-done { runtime, error }`, added to `events.rs`.
- `onnx::release` keeps its current rule for the CPU DLL. GPU runtimes are removed only by `acceleration_delete`.

### D8. Frontend
- `hooks/use-acceleration.ts` holds the status, the listeners and the actions, exposed as `acceleration` in the settings context. `persist` starts the runtime download after a save that selects DirectML or CUDA while it is not ready.
- `settings/models/acceleration-card.tsx` is first on the page, built from `SettingsCard`, `ToggleGroup`, `Progress`, `Button` and `Badge`, and implements the settings-window delta.
- `DownloadedModels` takes extra entries for the runtimes and the GPU variants.
- `lib/api.ts` gets the types and wrappers, and `lib/i18n.ts` the en/vi strings.

### Measured end to end (task 4.7)
Large v3 Turbo on the 8-minute `tst.mp4`, Vietnamese detected on every runtime:

| Runtime | Time | Segments | Where Whisper ran |
|---|---|---|---|
| CPU build | 121.3 s | 200 | CPU, q4 files |
| DirectML build | 115.6 s | 200 | CPU, q4 files, by design |
| CUDA build | 16.1 s | 199 | RTX 5060 Ti, q4f16 GPU variant, bound KV cache |

The CUDA transcript differs only in small wordings (for example "dịch từng từ" in both, with the cue split at 3160 ms).

## Risks / Trade-offs

- [`ort` rc.15 may require a newer C API than the DirectML 1.24.4 DLL provides] → Task 1.1 loads it through `ort::init_from` first. If it is refused, use the `Microsoft.ML.OnnxRuntime.DirectML` NuGet package of the newest version that loads, or pin `ort` to the newest release candidate that accepts both 1.24 and 1.31. If neither works, ship CUDA first and DirectML later; the spec allows a runtime to be missing from the choice.
- [The CUDA download is large (about 1.0 GB, 1.3 GB on disk) and needs a recent driver (CUDA 13 needs R580 or later)] → It is opt-in, sized in the UI and resumable through Retry. A driver that is too old hits the CPU fallback with the error shown (D2).
- [DirectML recompiles for every new input shape, and PaddleOCR detection gets a different size per screenshot] → Measured in task 4.5 on 8 real screenshots and crops of 407x269 to 1984x1144 with PaddleOCR Medium. No padding was needed:
  - Only the session's very first run was slow (1.9 s, then 129 ms). A new shape later cost at most 2.4x its repeat (259 against 109 ms).
  - Every image gave the same lines and text as the CPU, 6 to 7x faster (1984x1144: 4.4 s on the CPU, 0.63 s on DirectML).
- [Batched outputs can differ from batch 1 through float rounding] → The spec allows rare wording differences. Task 2.4 compares 20 real cues and requires at least 19 identical.
- [Restart while the settings window is open] → The new process opens with the window hidden, as on a normal start. The page tells the user snapr will restart.
- [The GPU runtime is deleted from disk while not loaded, but the setting still points to it] → `resolve` checks readiness at the first task, so Auto and explicit choices fall back to the CPU, and the card shows Download.
- [VRAM use when Turbo, PaddleOCR Medium and HY-MT are loaded together on an 8 GB card] → Sessions are already dropped after each task for Whisper and HY-MT, and PaddleOCR keeps only one size (`unload_others`). An out-of-memory error hits the CPU fallback.

## Migration Plan

- A config without `ai_acceleration` loads as Auto, and with no GPU runtime downloaded that means the CPU, as today. Existing CPU DLL and model paths do not move.
- Rollback: pick CPU and restart, or delete the GPU runtimes. An older snapr ignores the new config key and the extra files (`runtime/`, `*.gpu.onnx`).

## Open Questions

- Whether batch 16 on the CPU (0.50 s per cue against 0.66 s at 8) is worth the larger progress steps. It can be tuned after release, since the spec only says "up to 8".
