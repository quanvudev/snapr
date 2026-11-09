# Proposal

## Why

Snapr can transcribe and translate a recording only after it ends. Users also want to follow speech as it happens, for example a meeting, a video in another language or their own voice, with a translation shown next to it. The recording panel already selects a microphone and system audio and runs the audio pipeline, and the app already ships HY-MT translation. Low-latency streaming speech recognition (Moonshine v2 Streaming, MIT, ONNX Runtime) now makes live captions practical on the CPU.

## What Changes

- The recording panel gets a fourth mode, **Live captions**, next to full screen, window and area. In this mode the camera picker is unavailable, the preview shows the caption languages, and the bar gets a Spoken language picker and a Translate to picker (a target language or Off).
- In Live captions mode, Start does not open a picker and records nothing. It starts a live captions session that listens to the microphone and/or system audio chosen in the panel, and it turns the panel into a compact bar with Stop and the elapsed time.
- A new **captions window** pops up above the panel: frameless, always on top, never focused and excluded from screen capture. It can be dragged. It shows the line being spoken as it is recognised, and then each finished line with its translation under it. It has Copy all.
- Recognition is local and streaming, with one Moonshine Streaming model per spoken language: English, Vietnamese, Chinese, Japanese, Spanish, German, Arabic and Filipino. Translation of finished lines is local, with the existing HY-MT model.
- A new optional Moonshine model store, downloaded on request per language from the AI models page, the same way as the other model stores. The panel says when the model for the chosen language or the translation model is missing, and links to AI models.
- The action ring gets a **Live captions** item: a new built-in in the default ring, which can be added back on the ring page by users whose ring is customised. It opens the panel already in Live captions mode, and it reads Stop live captions while a session runs.
- The chosen spoken language and target language are remembered in the config.
- Live captions and a recording cannot run at the same time in this version. The record hotkey and the panel close button stop a captions session like a recording.

## Capabilities

### New Capabilities
- `live-captions`: the live captions session (start, stop, audio sources), the ring entry point, the captions window, streaming recognition of the spoken language, translation of finished lines, the Moonshine model store and the remembered caption languages.

### Modified Capabilities
- `recording-panel`: the mode choice gains Live captions; with that mode the camera picker is unavailable and Start begins a captions session instead of opening the picker; the remembered choices include the caption languages.

## Impact

- Backend: new `captions.rs` (model store, commands, events) and `captions/` (streaming engine on `ort`, tokenizer, line segmentation, session worker and translator thread). `panel.rs` gets the captions mode, the `Captions` phase and the captions window. `audio.rs` is unchanged: the session uses `Capture::start(.., record = true, ..)` and takes `chunks()`. `config.rs` gets `RecordMode::Captions`, `captions_language` and `captions_target`. `onnx.rs` gets `Workload::Captions` (CPU only), and `release` also checks the new store. `events.rs` gets `LIVE_CAPTION`, `CAPTIONS_PROGRESS` and `CAPTIONS_DONE`. `lib.rs` registers the commands and `State.captions_download`. The tray Quit stops a session. `ring.rs` gets `Wedge::Captions` (label, vi label, glyph), and `lib.rs` gets `Action::Captions`, dispatched by `hotkeys::trigger`.
- Frontend: `BUILTINS.captions` in `lib/ring.ts` (same glyph as `WEDGES`) and its i18n name, `components/captions/` (`CaptionsApp`, `?view=captions`), the setup panel mode and language pickers, the compact captions bar, an AI models card and a Downloaded models entry, `lib/api.ts` wrappers and i18n strings.
- Dependencies: none new (`ort` loads Moonshine's `.ort` files).
- Downloads: per-language model files from Moonshine's pinned asset repository, about 30 to 130 MB each (exact sizes are pinned during implementation).
- Licence: Moonshine Streaming models are MIT. About shows a notice once a model is downloaded.
