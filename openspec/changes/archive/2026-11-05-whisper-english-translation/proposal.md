# Proposal

## Why

Snapr already transcribes recordings locally with Whisper, but a Vietnamese (or any non-English) recording can only get subtitles in its own language. Whisper can translate speech to English with the same model and decoder by swapping one task token, so English subtitles for sharing are almost free to add. Translating an English video to English is pointless, so the option must only exist for videos whose speech is not English.

## What Changes

- Snapr remembers each video's spoken language: a transcription records the language Whisper detected (or the fixed transcript language), and a video with no known language can be detected on demand from its first 30 seconds of audio with the selected Whisper model. Languages are kept in an app data file, not beside the video.
- A new queue task kind `translate` runs Whisper with the translate task and writes `<video name>.en.srt` beside the video. The original `<video name>.srt` is never touched; the two files are independent tracks.
- Translate to English is offered only for a video whose known language is not English: in the MP4 context menu, in the transcript pane (an Original / English track switch) and in Export with subtitles (a track choice). For an English video, or one whose language is not known yet, none of it is shown; opening the transcript pane on an unknown video detects its language when the model is ready.
- The gallery hides `<video name>.en.srt` like the original transcript and recycles it together with the video on delete.
- Recording settings gain "Also translate to English" under the automatic transcription switch (shown only while it is on). When on, a finished recording's transcription queues a translation when it finishes, only if the detected language is not English.
- `transcript_read` and `transcript_save` take a track; the subtitle task spec takes a track; a new `transcript_detect` command returns a video's language. Defaults keep existing callers and stored retry specs working.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gallery`: the listing hides `.en.srt` sidecars and knows each video's language, delete also recycles the translation, a Translate to English action for non-English videos, transcript pane tracks and the subtitle export track.
- `screen-recording`: automatic transcription optionally also queues an English translation for non-English recordings.
- `settings-window`: the Recording page shows the "Also translate to English" switch and a note that Large v3 Turbo translates poorly.

## Impact

- Backend: `transcript/decode.rs` (task token choice, returning the detected language, a detect-only entry), `transcript.rs` (`Track`, `.en.srt` paths, `TranscribeTask` kind and chaining, the language store, `transcript_detect`, read / save track), `queue.rs` (`TaskSpec::Translate`, `Subtitle.track`), `subtitle.rs` (reads the chosen track, output name), `gallery.rs` (sidecar listing, `language` on items, delete), `config.rs` (`record_translate`), `snap.rs` (`queue_transcript`).
- Frontend: `lib/api.ts`, `lib/queue.ts` (`TASK_KINDS.translate`, specs, `ALREADY_ENGLISH`), `hooks/use-transcript.ts`, `transcript-pane.tsx`, `subtitle-dialog.tsx`, `item-menu.tsx`, `use-gallery.ts`, `recording/auto-transcribe-row.tsx`, `lib/i18n.ts`.
- No new model, dependency or download: translation and detection use the already selected Whisper model.
