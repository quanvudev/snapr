# Proposal

## Why

Text capture today depends entirely on the OCR engine built into Windows. On a machine with no OCR language pack, text capture, pin Copy text and gallery Extract text are all unavailable, and on others the result is limited to the installed languages. PP-OCRv6 (PaddleOCR, June 2026) ships official ONNX models in three sizes that read Simplified/Traditional Chinese, English, Japanese and 46 Latin-script languages offline, so snapr can offer an opt-in engine that works without any Windows language setup.

## What Changes

- New setting: OCR engine, `Windows` (default) or `PaddleOCR v6`. With PaddleOCR selected and its model ready, every recognition (text hotkey, pin Copy text, gallery Extract text) uses PaddleOCR instead of Windows OCR. QR decoding stays in front of either engine.
- New setting: PaddleOCR model size `Tiny | Small | Medium` (default Small), each with a one-line description and download size in the settings UI. Settings states that PaddleOCR does not read Vietnamese.
- Selecting PaddleOCR or a size that is not on disk downloads it once, in the background, with progress, cancel and retry in the settings window. The shared ONNX Runtime library is downloaded with the first size. Files are verified before use and stored in the local (non-roaming) app data folder.
- Downloaded sizes are kept. Settings lists each size as downloaded, in use or not downloaded, with a per-size Delete action; the runtime library is removed when no size is left.
- While the selected size is downloading, failed or missing, recognition falls back to Windows OCR. Text capture is available when Windows OCR is available or the selected PaddleOCR size is ready.
- Switching back to Windows takes effect immediately and keeps the downloaded files.

## Capabilities

### New Capabilities
- `ocr-engine`: choosing the recognition engine (Windows or PaddleOCR), PaddleOCR model sizes, their download, verification, storage, deletion and the fallback to Windows OCR.

### Modified Capabilities
- `text-capture`: recognition uses the selected engine instead of always the Windows profile languages; availability becomes "Windows OCR available or the selected PaddleOCR model ready", for the text hotkey, pins and gallery.
- `settings-window`: the Capture group gains the OCR engine choice, model size choice and the downloaded models list; the text capture switch is no longer disabled when PaddleOCR is ready.

## Impact

- Backend: `ocr.rs` gains an engine switch and a PaddleOCR pipeline (detection + recognition over ONNX Runtime); a new download/model-store module; `config.rs` gains `ocr_engine` and `paddle_model` (serde defaults, older configs load unchanged); `apply()` / `show_settings()` / `State.ocr` compute availability from both engines; `read_text` and `gallery_text` call the engine switch; new settings commands for download, cancel, delete and model status, plus a progress event to `main`.
- Frontend: Capture group rows for engine, size and models list; `lib/api.ts` wrappers and types; en/vi strings.
- Dependencies (Rust): `ort` (dynamic loading, no bundled runtime), an HTTP client, SHA-256, zip extraction for the ONNX Runtime archive, a YAML parser for the recognition dictionary.
- Network: first use of PaddleOCR downloads from Hugging Face (models) and GitHub (ONNX Runtime). Nothing is downloaded while the engine is Windows.
- Disk: about 6 MB (Tiny), 31 MB (Small) or 139 MB (Medium) per size, plus the runtime library, under `%LOCALAPPDATA%\com.quanvudev.snapr\paddle\`.
- Installer size is unchanged.
