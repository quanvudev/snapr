# Design

## Context

`decode::generate` starts every window with `[start, language, transcribe]`, where `transcribe` comes from `generation_config.json` `task_to_id`. The same file holds `task_to_id.translate` for every pinned model (Base, Small, Large v3 Turbo). Everything after the prompt (timestamp rules, segment cutting, seek) is task-independent, so translation is the same pipeline with a different task token.

Transcripts today are one sidecar `<stem>.srt`, found by `with_extension("srt")` in `transcript.rs`, `subtitle.rs` and `gallery.rs`. The queue identifies a task by `kind` plus `source`; the transcript pane and `transcript_save` use `Queue::has("transcribe", path)`.

## Goals / Non-Goals

**Goals:** an independent English track per video, created on demand or after recording, viewable, editable and burnable like the original.

**Non-Goals:** translating to languages other than English (Whisper cannot), translating English videos, a manual per-video language override, translating an existing `.srt` text (we re-run on the audio), a bulk-bar Translate action, a combined bilingual export, a separate model for translation.

## Decisions

### A `Track` enum, not a second module

`transcript.rs` gets `pub enum Track { Original, English }` (serde `original` / `english`, `Default = Original`) with `suffix()` (`""` / `".en"`), `kind()` (`"transcribe"` / `"translate"`) and `path(video) -> PathBuf` (`<stem><suffix>.srt`, built from the file stem, since `with_extension` on `a.en` would be wrong). Every place that now calls `with_extension(SRT_EXT)` goes through `Track::path`.

`TranscribeTask` gains `track: Track`; `kind()` returns `track.kind()`, so the queue, history, `queue-ended`, retry and the toast treat translation as its own kind with no worker change. `run` writes through `write_part_to` with stem `<stem><suffix>` and the final path `track.path(..)`, keeping today's atomic overwrite and cancel behaviour.

`decode::transcribe` takes the task token choice (`translate: bool`); `Ids` loads both `transcribe` and `translate` from `task_to_id` (translate optional: a model without it fails the task with a raw error rather than at load).

Alternative considered: a `task` field on `TaskSpec::Transcribe`. Rejected because the pane, `transcript_save` and `Queue::has` need to tell the two jobs of one video apart, and the kind is already that key.

### Video language

The language is needed before anything is queued (the menu and the pane hide the option for English), so it must be stored, not guessed per click.

Storage: `app_local_data_dir()/transcript-languages.json`, a map from the lowercased canonical video path to a Whisper language code (`"vi"`, `"en"`, `"ja"`…), behind a `Mutex` in `transcript.rs` (loaded lazily, written whole through a `.part` + rename). Nothing is written beside the video: the SRT format has no metadata, a JSON sidecar would clutter the folder and an NTFS alternate stream is dropped by OneDrive and FAT copies. A rename loses the record and the video falls back to detection; `delete_with_sidecar` removes the entry.

Writing: `decode::transcribe` returns the language it used with the segments (reverse lookup of the token id in `lang_to_id`, `<|vi|>` → `vi`). `TranscribeTask::run` for the original track records it after the SRT is written. A forced language (`vi` / `en`) is recorded as is.

Detecting: `decode::detect(model_dir, pcm)` runs the encoder and the existing one-step `Model::detect` over one 30 s window. `pcm` gets a `max_samples` limit so only the first 30 s is decoded. The async `transcript_detect(path) -> Option<String>` command (`guarded_mp4`, then `spawn_blocking`) returns the stored language, else detects when `whisper::ready` for the selected model (holding `whisper::acquire`), records it and emits `gallery-changed`; it returns `None` for no audio (`NO_AUDIO`), no model, or any error. One detection per path at a time (a `HashSet` of paths in flight).

Listing: `gallery_list` adds `language: Option<String>` to video items from the store (one lookup per video, store loaded once per listing).

Frontend rule, in one helper `canTranslate(item)` in `lib/gallery.ts`: `item.language != null && item.language !== "en"`. The pane shows the track switch when `canTranslate(item) || item.translation`. `useTranscript` calls `transcript_detect` once when the pane opens on a video with `language == null`; the resulting `gallery-changed` reload updates the item.

Alternative considered: detect the language from the SRT text. Rejected: telling English from other Latin-script languages by text is unreliable, while Whisper's own detection is already in the decoder.

### Queue spec

`TaskSpec::Translate { path }` (kind `translate`) maps to `TranscribeTask::validated(app, path, Track::English)`, which refuses `ALREADY_ENGLISH` when the stored language is `en` (an unknown language is allowed, for a retry or a stale grid). At run time the English track also checks the language Whisper detects in its first window and, when it is English, stops with `ALREADY_ENGLISH` before writing anything. `TaskSpec::Subtitle` gains `#[serde(default)] track: Track`, so retry specs stored in this session's history and the current frontend call keep working.

### Commands

`transcript_read(path, track)` and `transcript_save(path, track, segments)`; `track` is `#[serde(default)]`-like via `Option<Track>` on the command (Tauri commands have no serde default for arguments), `None` = Original. `transcript_save` refuses `TRANSCRIBING` while `Queue::has(track.kind(), path)`, so editing the original during a translation is allowed.

### Gallery listing

`sidecar_stem` of `a.en.srt` is `a.en`. The listing builds a second set from sidecars whose stem ends with `.en` (stripped), hides those matching a video stem, and sets a new `GalleryItem.translation: bool`. A `.en.srt` whose base matches no video stays an other file. `delete_with_sidecar` recycles both tracks, ignoring errors.

### Subtitle export

`SubtitleTask::validated` reads `track.path(..)` and writes `<stem>_subtitled` or `<stem>_subtitled_en` through `write_part`. `openSubtitleFromTranscript(item)` reads the original when `item.transcript`, else the English track. `SubtitleTarget` gains `track` and `tracks` (which exist); the dialog shows the Track ToggleGroup only when both exist and re-reads the cue text for the chosen track (one `transcript_read` on switch) for the preview.

### Frontend pane

`useTranscript(path, video, track)` keys its loaded state, job and error map by `path + track` and matches queue items and `queue-ended` by `track`'s kind. `use-gallery.ts` keeps `transcriptTrack` (default Original, kept across Left/Right like `transcriptOpen`). The pane header gets a small ToggleGroup Original / English; the missing state button reads Create transcript or Translate to English. `onExport(ms, text)` passes the current track.

### Automatic translation

`config.record_translate` (`#[serde(default)]`, off). The language is only known once the transcription has run, so the translation cannot be queued up front. `Snap::queue_transcript` sets `TranscribeTask.then_translate` from `record_translate` (only on this automatic path; `validated` leaves it false); at the end of a successful original-track run with a language other than `en`, the task builds `TranscribeTask::validated(.., Track::English)` and calls `State.queue.add(app, task, None)` from the worker thread (`add` works from any thread and the worker holds no lock while running). The order is therefore transcribe, compress (queued by its probe thread meanwhile), translate; the compress replace keeps the path, so the translation reads the compressed file under the same name. `auto-transcribe-row.tsx` renders the extra Switch while `record_transcribe` is on, plus the Turbo note when `whisper_model === "turbo"`.

## Risks / Trade-offs

- Large v3 Turbo was fine-tuned without translation data and often returns the source language → shown as a note in settings, not blocked, since the user may still prefer it.
- Running both tracks doubles Whisper time per recording → only when the user opts in; tasks stay sequential in the one queue worker.
- A video named `a.en.mp4` has the original sidecar `a.en.srt`, which collides with the English track of `a.mp4` → the listing gives the original-track match priority (an exact video stem match wins), accepted as an edge case.
- Language detection on one 30 s window can be wrong for a video that starts silent or mixes languages → the track switch also appears whenever `.en.srt` exists, and a run that detects English stops with `ALREADY_ENGLISH`; there is no manual language override in this change.
- On-demand detection costs one encoder pass of the selected model (noticeable with Turbo) → it runs only when the pane opens on an unknown video, once per video, and the result is stored.
