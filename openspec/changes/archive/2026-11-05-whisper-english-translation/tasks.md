# Tasks

## 1. Decoder and tracks

- [x] 1.1 `transcript/decode.rs`: load `task_to_id.translate` into `Ids` (optional), add a `translate: bool` parameter to `transcribe`, use it for the third prompt token in `generate`, and return the used language code (reverse lookup in `lang_to_id`) with the segments
- [x] 1.2 `transcript/decode.rs`: `detect(model_dir, pcm)` over one 30 s window, returning the language code
- [x] 1.3 `transcript.rs`: add `Track { Original, English }` (serde snake_case, default Original) with `suffix`, `kind` and `path(video)`; replace every `with_extension(SRT_EXT)` in `transcript.rs` with `Track::path`
- [x] 1.4 `transcript.rs`: `pcm` gets a `max_samples` limit
- [x] 1.5 `TranscribeTask`: add `track` and `then_translate`, take the track in `validated` (refuse `ALREADY_ENGLISH` for English when the stored language is `en`), return `track.kind()` from `kind()`, pass `translate` to `decode::transcribe`, stop with `ALREADY_ENGLISH` when the English track detects English, write `<stem><suffix>.srt` through `write_part_to`
- [x] 1.6 `transcript_read` / `transcript_save`: add an `Option<Track>` argument (None = Original); `transcript_save` refuses `TRANSCRIBING` with `Queue::has(track.kind(), path)`

## 2. Video language

- [x] 2.1 `transcript.rs`: the language store (`app_local_data_dir()/transcript-languages.json`, lowercased canonical path → code, lazy load, `.part` + rename write) with `language(path)`, `set_language(path, code)` and `forget(path)`
- [x] 2.2 Original-track `TranscribeTask::run` records the language after writing the SRT; with `then_translate` and a language other than `en`, it queues the English task through `State.queue.add(.., None)`
- [x] 2.3 Async `transcript_detect(path)` command: stored language, else (model ready) first 30 s `pcm` + `decode::detect` under `whisper::acquire`, store it, emit `gallery-changed`; `None` on no audio, no model or error; one detection per path at a time; register in `lib.rs`

## 3. Queue, subtitle export, gallery

- [x] 3.1 `queue.rs`: add `TaskSpec::Translate { path }` and `#[serde(default)] track` on `TaskSpec::Subtitle`; map both in `into_task`
- [x] 3.2 `subtitle.rs`: read the chosen track's SRT in `validated` and name the output `<stem>_subtitled_en` for English
- [x] 3.3 `gallery.rs`: hide `<video>.en.srt` sidecars, add `translation: bool` and `language: Option<String>` to video items, recycle both sidecars and forget the language in `delete_with_sidecar`

## 4. Automatic translation

- [x] 4.1 `config.rs`: add `record_translate` (`#[serde(default)]`, off) and its default
- [x] 4.2 `snap.rs`: in `queue_transcript`, set `then_translate` from `record_translate` on the original task

## 5. Frontend

- [x] 5.1 `lib/api.ts`: `Track` type, `record_translate`, `GalleryItem.translation` and `language`, `translate` and `subtitle.track` in `TaskSpec`, track argument on `transcriptRead` / `transcriptSave`, `transcriptDetect`
- [x] 5.2 `lib/queue.ts`: `TASK_KINDS.translate`, `translateSpec(path)`, `subtitleSpec(path, preset, track)`, `ALREADY_ENGLISH` in `TASK_ERRORS`; `lib/gallery.ts`: `canTranslate(item)`
- [x] 5.3 `hooks/use-transcript.ts`: take a `track`, key loaded / job / error state by path and track, match queue items and `queue-ended` by the track's kind, `create` queues the matching spec, call `transcript_detect` once when the video's language is unknown
- [x] 5.4 `use-gallery.ts`: `transcriptTrack` state kept across viewer navigation (falls back to Original on a video without the switch); `SubtitleTarget` gets `track` and `tracks`; `openSubtitleFromTranscript` falls back to the English track
- [x] 5.5 `transcript-pane.tsx`: Original / English ToggleGroup only when `canTranslate(item) || item.translation`, Translate to English in the missing English state, export passes the track
- [x] 5.6 `subtitle-dialog.tsx`: Track ToggleGroup when both tracks exist, preview cue from the chosen track, export with the track
- [x] 5.7 `item-menu.tsx`: Translate to English only when `canTranslate(item)`; Export with subtitles when `transcript || translation`
- [x] 5.8 `recording/auto-transcribe-row.tsx`: "Also translate to English" Switch while `record_transcribe` is on, Turbo note
- [x] 5.9 `lib/i18n.ts`: en / vi strings (track labels, Translate to English, task kind label, `ALREADY_ENGLISH`, auto translate switch, Turbo note)

## 6. Docs and checks

- [x] 6.1 Update `CLAUDE.md` (tracks, `translate` kind, the language store and `transcript_detect`, `record_translate`, `.en.srt` sidecar)
- [x] 6.2 `bunx tsc --noEmit` and `cargo clippy` pass
- [x] 6.3 Manual check in `bun run tauri dev`: record Vietnamese and English with auto transcribe + translate on (only the Vietnamese one gets `.en.srt`); open the pane on an older video and see its language detected; switch tracks, edit an English segment, export English subtitles; confirm an English video has no Translate to English; delete a video and confirm both `.srt` files go to the Recycle Bin
