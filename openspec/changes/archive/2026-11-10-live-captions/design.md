# Design

## Context

See proposal.md for motivation. Relevant current state:

- `panel.rs` owns the panel lifecycle (`Phase` closed / setup / picking / countdown / recording) and the `panel` and `camera` windows. `record_start` saves the choices and sends `Action::RecordPick(mode)`. `listen()` builds an `audio::Capture`; with `record = true` its `chunks()` gives 10 ms `Vec<i16>` chunks, 48 kHz stereo, with the microphone and system audio mixed and the microphone denoised.
- HY-MT (`translate/engine.rs`) has `Engine::load(env, dir)` and `complete_batch(requests, cancel)`, guarded by `translate::acquire()`. `onnx::target` / `init` / `session` / `gpu_for(Workload)` decide where a model runs.
- Whisper cannot stream: its encoder always processes a 30 s window, so each short chunk costs a full window.
- Moonshine Streaming (from `moonshine-ai/moonshine` `core/moonshine-streaming-model.cpp`): 16 kHz audio of any chunk length goes through a stateful `frontend` (state `sample_buffer`, `sample_len`, `conv1_buffer`, `conv2_buffer`, `frame_count`, fed back from the `*_out` outputs), then `encoder` (sliding window, left context `16 * depth` frames, lookahead `total_lookahead`), then `adapter` (`pos_offset`) giving `memory`, then `cross_kv` and a greedy `decoder_kv` with self-attention KV cache. BOS 1, EOS 2, `tokenizer.bin` BPE with U+2581 spaces, and the dimensions are read from `streaming_config.json`. The models ship as 8-bit `.ort` files in `moonshine-ai/moonshine-voice-assets`. Streaming models: English tiny/small/medium; Spanish, German and Japanese tiny/small; Vietnamese, Chinese, Arabic and Filipino tiny. All are MIT.

## Goals / Non-Goals

**Goals:**
- Sub-second latency for the current line on a mid-range CPU, without a GPU.
- Reuse the panel's audio selection, the `audio::Capture` pipeline, the HY-MT engine, the `onnx` runtime handling and the model store pattern.
- A session lifecycle separate from the recorder, so that captions during a recording can be added later without reworking it.

**Non-Goals:**
- Captions during a recording, or saving captions to an `.srt` (a later change can tee the same chunks).
- Streaming the translation token by token (only finished lines are translated).
- A hotkey slot of its own (the ring item is the shortcut), a remembered captions window position, and styling options.
- GPU execution for the caption model (Tiny/Small are fast enough on the CPU and the `.ort` files are quantized for the CPU).
- Silero VAD (line endings use an energy-based endpoint, see Decisions).

## Decisions

### Panel mode, not a new window or hotkey
Add `RecordMode::Captions` (serde `captions`). `record_start` saves the choices as today and then branches: for `Captions` it calls `captions::start(app)` instead of sending `RecordPick`. A new `Phase::Captions` drives the compact captions bar (Stop, timer, mic-lost warning, no Pause or Delete). `toggle` treats `Captions` like `Recording`, and `hotkeys::trigger` calls `captions::stop` when the phase is `Captions`, so the record hotkey stops a session.
*Alternative:* a separate captions hotkey and panel. Rejected because it duplicates device selection.

### Ring item `Wedge::Captions`
Add `Wedge::Captions` (serde `captions`, "Live captions" / "Phụ đề trực tiếp", a closed-caption glyph from Segoe Fluent Icons) to `WEDGES` and `Wedge::ALL`, and add `BUILTINS.captions` with the same glyph in `lib/ring.ts`. Because `DEFAULT_RING` derives from `BUILTINS`, a ring with `ring.items` unset gets the item automatically (9 items, under `MAX_RING_ITEMS` 12). A customised ring is untouched, and `removedBuiltins` already lists the item as addable. Its action is a new `Action::Captions`. `hotkeys::trigger` handles it on the main thread: when a session runs → `captions::stop`; when the panel is closed → `panel::open_with(app, Some(RecordMode::Captions))` (`open` becomes `open_with(app, None)`; the override changes only `PanelState.mode`, and the config is written at Start as usual); in Setup → set the mode to Captions and emit `record-panel` (the setup panel follows `state.mode` when it changes from the backend); otherwise ignore. `Snap::open_ring` filters the entries: it drops `Captions` while `State.recording` or `State.counting` is set, and drops `Record` while `State.captions` is `Some`. `Wedge::face` gets a `captioning` flag that swaps in the Stop live captions label and glyph, like Stop recording.
*Alternative:* reuse the Record item and pick the mode from the panel. Rejected because the user asked for a direct entry, and the panel would still need an extra click.

### Session = audio capture + recognizer thread + translator thread
`captions::start` builds `audio::Capture` through the existing `listen(app, mic, speaker, record = true, denoise)` and takes `chunks()`. It spawns:
1. **Recognizer thread**: downmixes and resamples 48 kHz stereo i16 to 16 kHz mono f32 (a 3:1 decimation with a short low-pass FIR), and accumulates audio. Every 160 ms (`STEP`) of new audio it runs `frontend` → `encoder` → `adapter` on the new audio only (incremental state), then a greedy decode over the current line's `memory` to get the partial text, and emits `live-caption { id, text, final: false }`. When the endpoint triggers, it emits `final: true`, sends `(id, text)` to the translator and resets the line state (decoder caches and adapter memory; the frontend and encoder keep their streaming state).
2. **Translator thread** (only when a target is set, it differs from the spoken language and HY-MT is ready): holds one `Engine` loaded at start under `translate::acquire()` and `onnx::gpu_for(Workload::Translate)`. It drains all pending finished lines and runs them in one `complete_batch` (so a backlog catches up in a single run), then emits `live-caption { id, translation }`.
`State.captions: Mutex<Option<Session>>` holds the capture, a stop flag and the join handles; `stop` sets the flag, drops the capture, joins both threads and closes the windows. The `Capture` drop already releases the devices.
*Alternative:* a queue task. Rejected because the queue is FIFO and one item at a time, so it would block GIF and transcription work for the whole session.

### Endpointing by energy and length
A line is finished when the 16 kHz RMS stays below an adaptive floor (noise estimate × 2, with a minimum) for `PAUSE` 1.2 s after speech was seen (700 ms cut sentences at natural pauses), or when the line reaches `MAX_LINE` 10 s. Silence before any speech is skipped without decoding. This avoids shipping Silero VAD, and the denoised microphone already keeps the floor low. The thresholds are constants and get tuned during implementation. Translation works on sentences, not lines: finished lines join under one id until a line ends with sentence punctuation, 4 lines or 240 characters, or 0.5 s more silence, and only then go to HY-MT, since translating line fragments gave wrong translations. Lines are attributed to the microphone or system audio by comparing the two sources' energy (sent with each mixed chunk), one model over the mix rather than one per source; a sustained louder other source ends the line, and the window renders the two sides as chat bubbles. Overlapping speech goes to the louder side.

### Port the streaming graph to Rust in `captions/stream.rs`
Mirror `moonshine-streaming-model.cpp`: load `frontend`, `encoder`, `adapter`, `cross_kv` and `decoder_kv` through `onnx::session(env, path, Workload::Captions)`, read `streaming_config.json` with serde, and keep the state tensors as owned `ndarray`s fed back each step. Cap decoding at `max_tokens_per_second` × the line duration (13 for non-Latin scripts, as upstream advises) and at `max_seq_len`. `captions/tokens.rs` decodes `tokenizer.bin` (format confirmed in task 1.1). `Workload::Captions` is CPU only in `on_card` and gets the same half-cores, no-spin setup as `Ocr`. The pinned `quantized_26_11_07` encoders take two more inputs than the C++ code feeds, `cache` [depth, 1, 16, encoder_dim] and `cache_valid` [16]: the port feeds both as zeros, so the graph behaves like the windowed path of the C++ code (checked against TTS English and recorded Vietnamese speech: the final text was exact for English and off by one diacritic for Vietnamese, with the slowest 160 ms step at 122 ms for English Small and 23 ms for Vietnamese Tiny in a release build). Byte-fallback pieces (`<0xNN>`) in the Tiny tokenizers are decoded to their byte instead of being skipped as upstream does.
*Alternative:* bundle the Moonshine C++ library. Rejected because it needs a native build and a second ONNX Runtime, while `ort` is already loaded dynamically.

### Model store `captions.rs`, shaped like `cutout.rs` but per language
`CaptionLanguage { En, Vi, Zh, Ja, Es, De, Ar, Tl }`. Each language has a pinned model: English, Spanish, German and Japanese use Small, the others Tiny (the largest streaming size that exists up to Small). Each model is a list of `fetch::Remote` files (URL at a fixed commit of `moonshine-voice-assets`, SHA-256, size) in `app_local_data_dir()/captions/<lang>/`. It has `ready` (files plus the runtime DLL), `installed`, one download at a time in `State.captions_download`, `captions-progress { language, done, total }` / `captions-done { language, error }`, the commands `captions_status` / `captions_download(language)` / `captions_cancel` / `captions_delete(language)` (refused while downloading or `IN_USE`, then `onnx::release`), and an `IN_USE` language list with `acquire`. `onnx::release` and every other store's delete count this store as installed or busy.

### Captions window
`captions` label, `index.html?view=captions`, built by `panel::build` with `focused(false)`, 640×200 logical, placed centred just above the panel. It is built from a spawned thread: `record_start` is a sync command, and building a webview inside its IPC callback on the main thread deadlocks (found while testing). It gets `WDA_EXCLUDEFROMCAPTURE` and is made non-activating with `WS_EX_NOACTIVATE` so clicks (scrolling, Copy all, dragging through `data-tauri-drag-region`) never take focus from the foreground app. `CaptionsApp` keeps lines keyed by `id` (`{ text, final, translation? }`), renders the partial line dimmed, a finished line in normal weight and its translation bold below it, and follows the bottom unless the user scrolled up (the same follow rule as the transcript pane). Copy all uses `navigator.clipboard.writeText`. It is added to `capabilities/default.json`.

### Config
`captions_language` (default `en`) and `captions_target` (default `""` = use `default_target(language, ui)`; `off` = no translation; else a code), both `#[serde(default)]`. They are written by `record_start` and passed to the panel in `PanelState` (`captions_language`, `captions_target`, plus `captions_ready` and `translate_ready` for the Start and notice logic). `record_select` gets the two language arguments.

### Panel UI
`setup-panel.tsx`: add the Captions entry (a `Captions` icon) to `modes`. When it is selected: hide the `CameraFeed` (release the stream) and show a placeholder with "EN → VI"; disable the camera dropdown; add two `DropdownMenu`s (Spoken language with 8 entries, Translate to with Off plus `lib/languages.ts`). Start is disabled with a tooltip when there is no audio source or the caption model is missing, and a small notice row with an "Open AI models" button (`show_settings(app, "models")` through a new `open_models` command) shows when a model is missing. `recording-panel.tsx` renders a captions variant for `phase === "captions"`.

## Risks / Trade-offs

- [The `.ort` files may not load in the `ort` crate's dynamically loaded runtime, for example because of a version or format mismatch] → Task 1.1 loads each graph with the pinned CPU runtime first. Fallback: the unconverted `.onnx` sources that the upstream docs say exist in the same bucket.
- [The streaming graph I/O is only partly documented] → port it from the C++ source at a pinned commit and check it against upstream with an ignored test on a WAV file (`SNAPR_CAPTIONS_MODEL`, `SNAPR_CAPTIONS_WAV`) that compares the text.
- [Energy endpointing splits lines badly in noisy rooms or music] → adaptive floor plus the 10 s cap. Silero VAD can replace the endpoint function later without changing the specs.
- [HY-MT on the CPU (~0.45 s per line) plus recognition compete for cores] → recognition uses half the cores with no spinning, and translation batches the backlog. On CUDA, HY-MT moves to the card through `gpu_for`.
- [Tiny models (Vietnamese ~9.4% WER, English Tiny 12%) give rough captions] → Small where it exists. The model choice is per language and can be raised when upstream publishes larger streaming models.
- [A session left running holds the microphone] → the compact bar is always visible and the tray Quit stops it.

## Migration Plan

Additive. Older configs load without the new fields. A config with `record_mode = "captions"` read by an older snapr fails to parse that field. That only happens on a downgrade, which the app does not support.

## Open Questions

- Using the encoder `cache` incrementally (only the new frames per step) could cut the encoder cost further; not needed at the measured speeds.
