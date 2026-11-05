# Tasks

## 0. Prerequisite

- [x] 0.1 Finish the manual check of `whisper-english-translation` and archive it, so its gallery requirements are in the main specs

## 1. Smoke test the model

- [x] 1.1 Pin `config.json` and `generation_config.json` hashes and sizes at commit `2f11819b25de08cecd344735cdfa5136ade41a67`
- [x] 1.2 Add `tokenizers` (`default-features = false`, `fancy-regex`) and an ignored test that loads `model_q4.onnx` with the bundled ONNX Runtime 1.31.1, runs one prefill and one decode step and translates one sentence vi → en; stop and report if a contrib op is missing

## 2. Language table and tracks

- [x] 2.1 `translate::LANGUAGES` (38 codes, English and Chinese names, Whisper code mapping) and `default_target(source, ui)`
- [x] 2.2 `transcript::Track` becomes `Original | Translation(Lang)` with serde as `"original"` / code; update `path`, `stem`, `kind` and every caller (Whisper English is `Translation(en)`)
- [x] 2.3 `Task::detail`, `QueueItem` / `HistoryItem` / `queue-ended` `detail`, `Queue::has` with an optional detail; `transcript_save` uses it

## 3. Model store

- [x] 3.1 `translate.rs`: pinned `MODEL`, folder layout, `ready`, `installed`, download slot, `translate-progress` / `translate-done`, `translate_status` / `translate_download` / `translate_cancel` / `translate_delete`; `events.rs` constants; register in `lib.rs`
- [x] 3.2 `onnx::release` counts `translate::installed` and the translate download slot
- [x] 3.3 `config.hymt_license_accepted` and `LICENSE_REQUIRED` in `translate_download`

## 4. Engine and task

- [x] 4.1 `translate/prompt.rs` (templates per language pair, one cue per request, reply cleanup); the tokenizer is used directly from `engine.rs`
- [x] 4.2 `translate/engine.rs`: session load, prefill, KV-cache greedy decode with repetition penalty, EOS / max tokens, cancel
- [x] 4.3 `HymtTask` (`validated`: transcript, target, same language; `run`: per cue, progress, `write_part_to` to `<stem>.<code>.srt`); make `render_srt` `pub(crate)`
- [x] 4.4 `TaskSpec::Translate { path, target }` choosing HY-MT or Whisper; `SAME_LANGUAGE`, `MODEL_MISSING` for a non-`en` target without HY-MT
- [x] 4.5 `record_translate` chaining goes through the same path with `target: None`

## 5. Gallery and subtitles backend

- [x] 5.1 `gallery.rs`: hide every `<stem>.<code>.srt` with a supported code, `translations: Vec<String>` on video items, recycle all tracks on delete
- [x] 5.2 `subtitle.rs`: `<stem>_subtitled_<code>` for translation tracks

## 6. Frontend

- [x] 6.1 `lib/api.ts` (`Track`, `translations`, `TaskSpec.translate.target`, `QueueItem.detail`, translate store commands and events), `lib/languages.ts`, `lib/queue.ts` (`translateSpec(path, target?)`, `SAME_LANGUAGE`, `LICENSE_REQUIRED`), `lib/gallery.ts` (`canTranslate` / `hasTracks` for both engines)
- [x] 6.2 `hooks/use-translate.ts`, `models/stores.ts` `useTranslateStore`, `models/translate-card.tsx`, `models/license-dialog.tsx`, Downloaded models entry
- [x] 6.3 `about/hymt-notice.tsx` in About while the model is ready
- [x] 6.4 `gallery/translate-dialog.tsx` and the item menu entry (Translate subtitles… with the model, Translate to English without)
- [x] 6.5 `transcript-pane.tsx` track `Select` and Translate action; `use-transcript.ts` matches `kind` + `detail`; `use-gallery.ts` keeps the chosen track code
- [x] 6.6 `subtitle-dialog.tsx` track `Select` over all tracks
- [x] 6.7 `recording/auto-translate-row.tsx` new label, description and Turbo rule
- [x] 6.8 `lib/i18n.ts` en / vi strings (card, licence dialog, notice, picker, errors, language names come from `lib/languages.ts`)

- [x] 6.9 Pane keeps a finished or newly requested translation track selected (targets seen while running stay listed until the gallery reloads; picking a target in the dialog selects it), Original shows the video's language, grid tiles show up to three subtitle language badges then `+N` (`subtitleCodes`, `Thumb badges`)

## 7. Docs and checks

- [x] 7.1 Update `CLAUDE.md` (translation store, engine, tracks, licence flow)
- [x] 7.2 `cargo clippy` and `bunx tsc --noEmit` pass
- [x] 7.3 Manual check: licence dialog, download, cancel, delete; translate an English video to Vietnamese and a Vietnamese one to Japanese; edit a translated cue; export Japanese subtitles; auto translate after recording with and without the model; delete a video with several tracks
