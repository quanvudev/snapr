# Design

## Context

This change builds on `whisper-english-translation` (implemented, not yet archived; it must be archived before this one so the gallery requirements it adds exist in the main specs). That change introduced `transcript::Track { Original, English }`, the `translate` queue kind (Whisper's `<|translate|>` task), `<stem>.en.srt`, the video language store (`transcript/languages.rs`, `transcript_detect`), `canTranslate` / `hasTracks` on the frontend and `record_translate` chaining from the original transcription.

Model facts, checked against `onnx-community/HY-MT1.5-1.8B-ONNX` at commit `2f11819b25de08cecd344735cdfa5136ade41a67`:
- Files used: `onnx/model_q4.onnx` (448,829 B, SHA-256 `9ffed3b4d2321e42e4c79b970ee73ff80bd10fd4f89208a2b6ee6a5ef59b800c`), `onnx/model_q4.onnx_data` (1,405,788,224 B, SHA-256 `ab975a558df7dbb1ed863d983e8d530e85eeb16de6f7028542b9c0af9a2222eb`), `tokenizer.json` (8,672,322 B, SHA-256 `1b119fa76913d752f1003222865691e56f69fbff53b257acc10f8fd183d4f70b`), plus `config.json` (1,639 B, SHA-256 `391307b3bab6318ea6987071ccb82e1502aa83bc19f52a93428f7efb9466ecaa`) and `generation_config.json` (255 B, SHA-256 `d350fa7971cdf384def814c0f21ee35b89ae9ca8608f7b7a1203e3a6111d4f5c`). `q4` (MatMulNBits, fp32 activations) is chosen over `q4f16` because Snapr ships the CPU ONNX Runtime 1.31.1, where fp16 activations are slow or unsupported.
- Graph: decoder-only `HunYuanDenseV1ForCausalLM`, 32 layers, 16 heads, 4 KV heads, head dim 128, vocab 120,818, `com.microsoft` `GroupQueryAttention`, `RotaryEmbedding`, `MatMulNBits`. Inputs `input_ids`, `attention_mask`, `position_ids`, `past_key_values.{0..31}.{key,value}`; outputs `logits`, `present.{0..31}.{key,value}`.
- Tokenizer: byte-level BPE (120,000 vocab, 119,758 merges) with a regex pre-tokenizer using a lookahead (`\s+(?!\S)`). Special ids: BOS `<｜hy_begin▁of▁sentence｜>` 120000, `<｜hy_User｜>` 120006, `<｜hy_Assistant｜>` 120007, EOS `<｜hy_place▁holder▁no▁2｜>` 120020, pad 120002.
- Chat template (no system prompt): `BOS <｜hy_User｜>{content}<｜hy_Assistant｜>`.
- Prompts (from the model card): XX⇄XX `Translate the following segment into {target_language}, without additional explanation.\n\n{source_text}`; ZH⇄XX uses the Chinese template; contextual: `{context}\n参考上面的信息，把下面的文本翻译成{target_language}，注意不需要翻译上文，也不要额外解释：\n{source_text}\n`.
- Licence: Tencent HY Community License, territory excludes EU / UK / South Korea; distributions to third parties need the Notice text; "Powered by Tencent HY" marking is encouraged, not required; output must not improve other AI models; end users must be told Tencent is not affiliated.

## Goals / Non-Goals

**Goals:** on-device transcript translation into 38 targets, clear licence consent, reuse of the queue, track and model-store machinery.

**Non-Goals:** the 7B model, GPU execution providers, terminology lists or glossary prompts, translating photos or OCR text, bilingual subtitle burn-in, region detection or blocking (the dialog states the territory; Snapr does not geolocate), translating directly from audio with HY-MT.

## Decisions

### Tracks become open-ended

`Track` changes to `Original | Translation(Lang)` where `Lang` is a validated code from one table `translate::LANGUAGES` (`code`, English name, Chinese name, Whisper code when different). The Whisper English track is `Translation(en)`, so `.en.srt` keeps working with no migration. Serde: `"original"` or the code string (`"en"`, `"zh-Hant"`); the frontend `Track` type becomes `"original" | LangCode`. `Track::path` is `<stem>.srt` / `<stem>.<code>.srt`; `kind()` is `transcribe` / `translate` for every translation.

The queue key `(kind, source)` is no longer enough to tell two translations of one video apart, so `Task` gains `fn detail(&self) -> Option<String>` (default `None`), `QueueItem` / `HistoryItem` / `queue-ended` carry `detail` (the target code), and `Queue::has` gains an optional detail match. `transcript_save` refuses only while that exact target runs.

`gallery_list` reports `translations: Vec<String>` (the codes of existing `<stem>.<code>.srt` with a supported code) instead of `translation: bool`; `transcript: bool` stays. `delete_with_sidecar` recycles the original and every listed code.

### Two engines behind one task kind

`TaskSpec::Translate { path, target: Option<String> }`. `into_task`:
- translation model ready → `HymtTask::validated(path, target.unwrap_or(default))`: needs `<stem>.srt` (`NO_TRANSCRIPT`), a supported target different from the known source (`SAME_LANGUAGE`), reads the segments at queue time like `SubtitleTask` (later edits never reach a queued task).
- not ready → today's Whisper `TranscribeTask(Track::Translation(en))`; a `target` other than `en` is refused with `MODEL_MISSING`.

The default target (`translate::default_target(source, ui)`) is `vi → en`, `en → vi`, else the interface language from `config.language`; an unknown source uses the interface language too, but the UI only offers translation when the language is known.

`record_translate` chaining (`TranscribeTask::queue_translation`) calls the same `into_task` path with `target: None`, so the automatic flow picks HY-MT when ready and Whisper English otherwise (still skipped for English without HY-MT).

### Model store (`translate.rs`)

Shaped like `whisper.rs`: `MODEL` (one entry, the five pinned `fetch::Remote`s at the commit above), files in `app_local_data_dir()/translate/hymt-1.8b/` (`model.onnx`, `model.onnx_data` — the external data file must keep the name the graph references, so it is stored as `model_q4.onnx_data` beside `model_q4.onnx`; `tokenizer.json`, `config.json`, `generation_config.json`), `ready`, `installed`, `onnx::ensure` for the shared runtime, one download slot `State.translate_download`, events `translate-progress` / `translate-done`, commands `translate_status`, `translate_download`, `translate_cancel`, `translate_delete` (refuses while in use, then `onnx::release` with both other stores' busy flags). `onnx::release` gains `translate::installed`.

Licence consent: `config.hymt_license_accepted: bool` (`#[serde(default)]`). `translate_download` refuses `LICENSE_REQUIRED` while it is false; the frontend opens the AlertDialog, and "I agree and download" saves the flag and then downloads. The licence URL is the model's `License.txt` on Hugging Face (`tencent/HY-MT1.5-1.8B`).

### Engine (`translate/`)

- `tokenizer.rs`: `tokenizers::Tokenizer::from_file`, with the crate built `default-features = false, features = ["fancy-regex"]` (Oniguruma needs a C build; the lookahead in the pre-tokenizer needs fancy-regex). Special tokens are inserted as ids, never through text.
- `engine.rs`: one `ort` session (`commit_from_file` on `model_q4.onnx`, external data resolved beside it), loaded per task and dropped after, like Whisper. Prefill the prompt once, then greedy decode one token per step feeding `present.*` back as `past_key_values.*`, `attention_mask` all ones and `position_ids` counting on; empty past tensors `[1, 4, 0, 128]` for the first step. Greedy with the card's `repetition_penalty` 1.05 (no sampling, so a re-run gives the same subtitle); stop at EOS 120020 or `MAX_OUTPUT_TOKENS` (4× the source tokens, at least 64). Cancel is checked between steps.
- `prompt.rs`: picks the template (Chinese for any `zh` / `zh-Hant` / `yue` side, XX otherwise) and the target name (English or Chinese name from `LANGUAGES`). Each cue is sent alone. The contextual template was measured on the real model and rejected: with the previous cues in the prompt it added content from them in 2 of 8 cases (Chinese template) and 5 of 8 (an English variant), for example "Sure." becoming "Of course. Can you send me the report before Friday?", while the plain template never did; the cost is that a very short cue can be read loosely ("Rồi." → "Then."). The decoded reply is trimmed; a reply wrapped in quotes or starting with the target name and a colon is cleaned.
- `HymtTask::run`: one cue at a time (empty cues copied as empty), progress `done / cues`, writes `<stem>.<code>.srt` through `queue::write_part_to` with `render_srt` (made `pub(crate)`), so cancel and failure keep the old file.

Expected speed on a mid-range CPU is a few seconds per cue; a 10-minute video (about 150 cues) takes several minutes. This is acceptable as a background queue item and is stated in the card's description, not promised.

### Frontend

- `lib/languages.ts`: the 38 codes with en / vi display names (one table, mirrored by `translate::LANGUAGES`; change both together) and `defaultTarget(source, ui)`.
- `models/translate-card.tsx` built from `ModelCard` + `useTranslateStore` (`stores.ts`, over a new `hooks/use-translate.ts`), the licence `AlertDialog` (`models/license-dialog.tsx`), Downloaded models and About (`about/hymt-notice.tsx`, shown while ready).
- `gallery/translate-dialog.tsx`: a `Dialog` with a `Select` of targets (source excluded, default preselected) and Translate; opened from the item menu and from the transcript pane.
- `transcript-pane.tsx`: the Original / English ToggleGroup becomes a `Select` of tracks (Original, each `translations` code, plus a running target not yet on disk) and a Translate button when the model is ready; with no model the Whisper English rules from the previous change apply unchanged. `useTranscript(item, track, video)` matches queue items by `kind` and `detail`.
- `subtitle-dialog.tsx`: the Track ToggleGroup becomes a `Select` when there are more than two tracks.
- `recording/auto-translate-row.tsx`: new label and description; the Turbo note only without the translation model.

## Risks / Trade-offs

- Licence and territory: Snapr cannot enforce the territory → the dialog states it before download and the notice is shown in About; the user downloads from Hugging Face directly, Snapr never bundles the weights.
- Size (1.42 GB) and RAM (the q4 weights plus a KV cache of 32 × 2 × 4 × 128 floats per token, about 128 KB per token) → download only on request; one cue at a time keeps the cache small.
- CPU speed → background queue, cancel between tokens, progress per cue.
- Contrib ops (`GroupQueryAttention`, `RotaryEmbedding`, `MatMulNBits`) must exist in the bundled ONNX Runtime 1.31.1 CPU build → verified first in tasks (a load-and-one-step smoke run) before the rest is built; if missing, stop and revisit the runtime pin.
- `tokenizers` adds compile time and binary size → no default features; it is only loaded by the translation task.
- Mismatch between the Whisper language codes and the HY-MT table (`zh` vs `zh-Hant` / `yue`, `tl`) → the language table maps Whisper codes to targets; a source code with no mapping still allows every target.
