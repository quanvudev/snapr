# Design

## Context

All recognition goes through `ocr::extract(&RgbaImage) -> Result<String, String>`. It is called from two threads: the save worker (`read_text`, for the text hotkey and pin Copy text; WinRT MTA) and `gallery_text` (a `spawn_blocking` thread). `ocr::qr` runs before it in both callers. `State.ocr: AtomicBool` caches `ocr::available()`. `apply()` refreshes it, and so does `show_settings()` when the value changed. That flag gates the text hotkey, the ring Text wedge, the pin Copy text button (read at `Pin::open`), `check(text)` and the frontend's `ocr_available`. The Windows path groups OCR lines into rows with the `Row` logic in `recognize`.

PP-OCRv6 ONNX models are published by PaddlePaddle on Hugging Face as `PaddlePaddle/PP-OCRv6_{tiny,small,medium}_{det,rec}_onnx`. Each repo holds `inference.onnx` and `inference.yml`. The recognition dictionary (18,708 entries for small) is the `PostProcess.character_dict` list in the rec `inference.yml`. Decoding is `CTCLabelDecode`. Rec input is BGR, height 48. Det uses `DBPostProcess` (`thresh 0.2`, `box_thresh 0.45`, `unclip_ratio 1.4` for small) with ImageNet mean/std normalization. Sizes on disk:

| size   | det.onnx | rec.onnx |
|--------|---------:|---------:|
| tiny   |  1.8 MB  |  4.5 MB  |
| small  |  9.9 MB  | 21.2 MB  |
| medium | 62.0 MB  | 76.6 MB  |

## Goals / Non-Goals

**Goals:**
- No change in installer size or startup time for users who keep the Windows engine. No ONNX Runtime is loaded and nothing is downloaded until PaddleOCR is selected.
- One `ocr::extract` entry point, so hotkey, pin and gallery cannot drift.
- A size is either fully ready (all files verified) or absent. There is no partial state on disk that counts as ready.

**Non-Goals:**
- Vietnamese, Korean, Arabic, Cyrillic, Thai and other scripts outside PP-OCRv6. The PP-OCRv5 per-language rec models are not offered.
- Rotated or vertical text. Boxes are axis-aligned, which suits screenshots, and there is no angle classifier.
- GPU / DirectML execution. Inference is CPU only.
- Automatic model updates. A new model version means new pinned hashes in a new snapr release.

## Decisions

### ONNX Runtime through `ort` with `load-dynamic`

`ort` 2.x with `default-features = false, features = ["load-dynamic", "ndarray"]`. The runtime DLL is loaded from an absolute path with `ort::init_from(<paddle>/onnxruntime.dll)`, once, on the first PaddleOCR recognition. Loading from an absolute path matters because Windows 11 ships its own `System32\onnxruntime.dll`, and a bare name could resolve to that one.

Alternatives considered:
- `ort` with bundled or static binaries: about +19 MB in the installer for every user. This was rejected by the "default off, download on enable" goal.
- A PaddleOCR-json / RapidOCR sidecar exe: an extra process and an unsigned downloaded exe (antivirus risk), with less control over errors.
- MNN-based crates: a C++ build in the toolchain, and still linked for every user.

### Where the runtime DLL comes from

The DLL is extracted from the official `onnxruntime` CPU wheel on PyPI (`onnxruntime-<ver>-cp313-cp313-win_amd64.whl`, 14 MB). The download URL and the wheel's SHA-256 are pinned in code. Only `onnxruntime/capi/onnxruntime.dll` is extracted, with the `zip` crate. The wheel's DLL is the standard CPU build that exports the C API (`OrtGetApiBase`). The ORT version is the one the pinned `ort` release requires, or newer, since the C API is backward compatible.

Alternative: the GitHub release `onnxruntime-win-x64-<ver>.zip`. It is the official C distribution, but it is 79 MB for the same DLL. Reading just the DLL entry with HTTP Range requests would work but adds code for little gain.

### Model files, pinned and verified

`paddle.rs` holds a static table. For each size it lists the det and rec files, each with a URL pinned to a Hugging Face commit (`https://huggingface.co/PaddlePaddle/<repo>/resolve/<sha>/inference.onnx`), the SHA-256 (the LFS `oid`) and the byte size (used for progress and for the size shown in settings). The rec `inference.yml` is downloaded too and pinned by SHA-256.

Layout under `app_local_data_dir()/paddle/`:

```
paddle/
  onnxruntime.dll
  tiny/   det.onnx  rec.onnx  dict.txt
  small/  det.onnx  rec.onnx  dict.txt
  medium/ det.onnx  rec.onnx  dict.txt
```

Each file is streamed to `<name>.part` while a `sha2::Sha256` is updated. On a match the file is renamed to its final name. On a mismatch, or on any error, the `.part` file is removed. `dict.txt` is written last, after the YAML `character_dict` has been parsed into one entry per line. The YAML is parsed with `yaml-rust2`, and only the dictionary is kept, not the YAML. **A size is ready when `det.onnx`, `rec.onnx` and `dict.txt` all exist and `onnxruntime.dll` exists.** Because the files appear only after verification and `dict.txt` comes last, this file check is enough. No manifest is needed, and leftover `.part` files are deleted at the start of the next download.

HTTP: `ureq` 3 (blocking, rustls). Downloads run on a dedicated `std::thread`, never on the main or capture thread.

### Download control and progress

`State.download: Mutex<Option<Download>>`, where `Download` holds `{ size, cancel: Arc<AtomicBool> }`. Only one download runs at a time: starting a second one while one runs is refused. The thread emits a `paddle-progress` event to `main` with `{ size, done, total }`, throttled to about 10 per second. It finishes by emitting `paddle-done { size, error: Option<String> }`, then calls `apply()` so `State.ocr` and the text hotkey update.

New commands (in `settings.rs` or a new `paddle.rs`):
- `paddle_status` returns `{ ready: [sizes], bytes: {size: n}, downloading: Option<{size, done, total}>, runtime: bool }`.
- `paddle_download(size)`, `paddle_cancel()` and `paddle_delete(size)`.

`paddle_delete` refuses a size that is downloading. It runs `remove_dir_all(paddle/<size>)`, drops the cached engine if it was that size, removes `onnxruntime.dll` once no size remains, and calls `apply()`. Deletion is a direct removal and does not go through the Recycle Bin, because these are app data and not user files.

Starting a download is driven by the frontend. When the engine is set to PaddleOCR, or the size changes, `use-settings` saves the config and then calls `paddle_download` if the size is not ready. The backend `save` stays a pure save. Startup never downloads anything. If the user quits mid-download, the next start shows the size as not downloaded, and the Download action resumes from scratch.

### Engine selection and caching

```
ocr::extract(img, cfg)
   |
   +-- cfg.ocr_engine == paddle && paddle::ready(size) --> paddle::extract(img, size)
   |
   +-- otherwise --------------------------------------------> windows recognize(img)

ocr::available(cfg) = windows_available() || (cfg.ocr_engine == paddle && paddle::ready(size))
```

`extract` and `available` take the engine and size, read from `State.cfg` by the callers, so `ocr.rs` does not reach into `State`. The loaded PaddleOCR engine is a process-wide `static ENGINE: Mutex<Option<(Size, Engine)>>`, with `Engine { det: Session, rec: Session, dict: Vec<String> }`. It is created on the first use of a size and replaced when the size changes. A `Mutex` is used because `ort` sessions run with `&mut self` and the gallery and the worker are different threads. The mutex also keeps memory to a single loaded size. The engine is never loaded while the engine setting is Windows.

### PaddleOCR pipeline

1. **Detection:** RGBA to BGR f32, resized so that the longer side is at most 960 and both sides are multiples of 32, normalized with ImageNet mean/std, then `det.run`. The probability map is binarized at `thresh`. Connected components (4-neighbour flood fill over the bitmap) become axis-aligned boxes. Boxes whose mean probability is below `box_thresh` are dropped. Each box is expanded by the DB unclip distance `area * unclip_ratio / perimeter` and scaled back to image coordinates. Thresholds come from a small per-size constant table copied from each det `inference.yml`.
2. **Recognition:** each box is cropped from the original image and resized to height 48, keeping the aspect ratio (width capped at 3200). It is normalized `(x/255 - 0.5) / 0.5` in BGR. Crops are batched (up to 8, padded to the batch's widest crop) through `rec.run`. Decoding is greedy CTC: argmax per step, repeats collapsed, blank index 0 dropped, index `i` mapped to `dict[i-1]`, and a trailing space entry is appended to the dictionary as `CTCLabelDecode` does with `use_space_char`.
3. **Layout:** each recognized box becomes `(x, top, bottom, text)` and goes through the same row grouping as the Windows path. That logic is factored out of `recognize` into `fn rows(lines) -> String`, so both engines produce identical row and line behavior.

### Availability refresh

`apply()` computes `State.ocr` with the new `available(cfg)`. `show_settings()` already re-applies when availability changes. The download thread and `paddle_delete` call `apply()`. Saving a config that changes the engine or size already goes through `apply()`. `Loaded.ocr_available` keeps its meaning, "text capture can work now", so the existing gating in `text-capture-row.tsx`, `check(text)`, the ring and pins needs no change. `Loaded` also gains `windows_ocr: bool`, so the "No OCR language installed" hint and language-settings action can still be shown correctly under the Windows engine.

### Config

`Config` gains `ocr_engine: OcrEngine` (`windows` | `paddle`, default `windows`) and `paddle_model: PaddleSize` (`tiny` | `small` | `medium`, default `small`). Both are serde enums in lowercase with field-level `#[serde(default)]`, so older configs load unchanged.

### Settings UI

The rows go in `components/settings/capture/` after `text-capture-row.tsx`. Every row is built from existing shadcn components.

- `ocr-engine-row.tsx`: a `ToggleGroup` with Windows | PaddleOCR v6.
- `paddle-model-row.tsx`, shown only for PaddleOCR: a `ToggleGroup` with Tiny | Small | Medium, plus an `ItemDescription` with the size's description and download size and the "does not read Vietnamese" note. Under it is the status: a `Progress` with the percentage and a Cancel `Button` while downloading, the error with Retry, or "Not downloaded" with Download.
- `paddle-models-row.tsx`: an `Item` list of the three sizes, with a `Badge` showing in use, downloaded or not downloaded, the disk size, and a Delete button that opens an `AlertDialog`.

`hooks/use-paddle.ts` owns `paddle_status`, listens to `paddle-progress` / `paddle-done`, and exposes download, cancel and delete. Its state goes through `useSettingsContext()` the same way as `ocr`. The en/vi strings go in `lib/i18n.ts`. Download failures are shown inline in the row, because the row has a Retry action, and not as a toast.

## Risks / Trade-offs

- [`ort` 2.x is still a release candidate and its API changes between rcs] → pin the exact rc in `Cargo.toml`, keep all `ort` calls inside `paddle.rs`.
- [Hugging Face or PyPI moves or removes a file] → URLs are pinned to a commit / exact file, hash mismatch or 404 shows an error with Retry, Windows OCR keeps working as fallback.
- [Medium is slow on weak CPUs (≈ hundreds of ms to seconds per capture)] → the description says so, the work runs off the UI threads, Small is the default.
- [Our DB post-process is simpler than Paddle's (no polygon unclip, no rotation)] → acceptable for horizontal screenshot text; boxes are padded by the unclip distance; a spike task compares output with the reference Python pipeline on sample screenshots.
- [Users expect Vietnamese because snapr defaults to vi] → the UI states it plainly next to the size choice, and the Windows engine stays the default.
- [Antivirus flags a DLL downloaded into AppData] → it is Microsoft-signed `onnxruntime.dll`, extracted unmodified, hash-pinned. A load failure is reported as an error, and recognition falls back to Windows.
- [Memory: Medium holds ~140 MB of weights while loaded] → only one size is cached; it is dropped on size change, on delete, and when the engine is switched to Windows.

## Migration Plan

There is no data migration. Older configs load with `ocr_engine = windows`, so behavior is identical until the user opts in. Rollback is a matter of selecting Windows. Deleting the sizes in settings removes every downloaded file. Uninstalling snapr does not remove `%LOCALAPPDATA%\com.quanvudev.snapr\paddle\`, the same as the existing app data.

## Spike results

Measured on a Ryzen 7 7800X3D (8 cores, CPU only) with `ort` 2.0.0-rc.15 (`load-dynamic`) and the `onnxruntime.dll` from the onnxruntime 1.31.1 cp313 wheel. The reference was PaddleOCR 3.7.0 / PaddleX 3.7.2 with `engine="onnxruntime"` and the same ONNX files. Its output was grouped with the same `rows` logic before comparing. Twelve images were used: six rendered with Windows fonts (two English lines, a spaced nav bar, code, Simplified Chinese, Japanese, mixed Chinese and English), three crops of a real terminal screenshot, two real screenshots (1984x1096 and 1214x622) and a 2560x1440 image with 12 px Latin and 13 px Chinese text.

- **Detection resize: deviation from this design.** Limiting the longer side to 960 loses small text on large captures. On the 2560x1440 image, similarity with the reference dropped to 0.05 (Tiny) and 0.13 (Small), and a 1820 px wide terminal crop lost words. The reference pipeline resizes with `limit_type: min`, `limit_side_len: 64` and `max_side_limit: 4000`. The implementation now does the same: it scales up only when the shorter side is under 64, scales down only when the longer side is over 4000, and rounds both sides to multiples of 32. This costs little, because recognition dominates the time.
- **Text match (Small).** Output is identical to the reference on all six rendered samples, including Japanese and Chinese. Character similarity on the real captures is 0.93 to 1.00, and line counts match. The remaining differences are spaces between CJK and Latin runs, the odd character in dense 14 px terminal text, and Vietnamese diacritics, which neither engine reads.
- **Text match (Tiny).** Output is identical to the reference on English, Chinese and the nav bar, and similarity on the real captures is 0.90 to 0.97. Tiny cannot read Japanese kana: it returns lookalike CJK and Latin glyphs, and so does the reference. This matches the "without Japanese" description.
- **Thresholds.** The det `inference.yml` values are used: `thresh 0.2`, `unclip_ratio 1.4` for all sizes, `box_thresh` 0.4 for Tiny and 0.45 for Small and Medium. The reference pipeline overrides them with 0.3 / 0.6 / 1.5 from `OCR.yaml`, but the outputs still agree. The per-size table can keep the model values.
- **Mean/std.** Paddle applies the ImageNet mean/std in index order to BGR channels, so the B channel gets 0.485/0.229. The implementation does the same for parity.
- **Rec width.** Crops are padded to at least 320 px, like `RecResizeImg [3, 48, 320]`. Without this padding, short words decode worse.
- **Engine cache.** The first call loads the runtime and both sessions, taking about 100 ms (Tiny) or 140 ms (Small) more than later calls. Later calls reuse the cached sessions.
- **Not handled (as planned).** There is no rotated or vertical text and no angle classifier. Boxes are axis-aligned bounding boxes of 4-connected components, unclipped by `area * ratio / perimeter`.

Per-image time in ms, after warm-up (ours, then the reference):

| image | px | Tiny | Small |
|---|---|---:|---:|
| two English lines | 345x84 | 13 / 15 | 42 / 51 |
| nav bar | 330x55 | 11 / 12 | 45 / 36 |
| code, 4 lines | 304x130 | 20 / 20 | 59 / 65 |
| Chinese, 2 lines | 296x86 | 10 / 11 | 33 / 33 |
| Japanese, 2 lines | 232x86 | 10 / 9 | 27 / 23 |
| terminal paragraph crop | 1820x255 | 230 / 221 | 880 / 992 |
| terminal tabs crop | 390x70 | 19 / 20 | 65 / 86 |
| terminal footer crop | 1820x100 | 75 / 65 | 182 / 200 |
| gallery screenshot | 1214x622 | 50 / 43 | 90 / 92 |
| full terminal screenshot | 1984x1096 | 730 / 662 | 2380 / 2540 |
| 2560x1440, 60 short lines | 2560x1440 | 410 / 441 | 1110 / 1241 |

Recognition takes most of the time on text-heavy captures: on the full terminal screenshot with Small, detection takes 283 ms of 2.4 s. A typical area capture of a few lines takes 10 to 60 ms with Tiny or Small.
