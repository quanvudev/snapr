# Proposal

## Why

`whisper-english-translation` gives non-English videos English subtitles, but Whisper can only translate into English: an English video cannot get Vietnamese subtitles, and no video can get any other language. HY-MT1.5-1.8B (Tencent, 33 languages plus 5 variants, published as ONNX by `onnx-community`) translates text between all of them on device, so the existing transcript can be translated into whichever language the viewer needs. Its licence is not open source and excludes some regions, so the user must see and accept it before Snapr downloads it.

## What Changes

- A new optional translation model store "HY-MT1.5 1.8B" in AI models: the `q4` ONNX build (about 1.41 GB, pinned to a commit, SHA-256 checked) plus its tokenizer, downloaded only on request, deletable, shown in Downloaded models.
- A licence confirmation before the first download: size, that the Tencent HY Community License does not apply in the European Union, the United Kingdom or South Korea, that outputs may not be used to improve other AI models, that Tencent is not affiliated with Snapr, and a link to the full licence. Download starts only after "I agree and download"; the acceptance is saved.
- About shows "Translation powered by Tencent HY" with the licence notice text while the model is downloaded.
- A subtitle translation task: translates a video's original transcript (`<video name>.srt`) cue by cue, keeping every timing, into one of 38 target languages and writes `<video name>.<code>.srt` (for example `.vi.srt`, `.ja.srt`, `.zh-Hant.srt`).
- The target defaults by source language: Vietnamese to English, English to Vietnamese, anything else to Snapr's interface language; the user can pick any other supported language that differs from the source.
- The gallery offers "Translate subtitles…" (a target picker) for an MP4 with a transcript and a known language while HY-MT is downloaded. Without HY-MT, the existing Whisper "Translate to English" stays as the fallback for non-English videos.
- The transcript pane lists every subtitle track the video has (Original plus each translation, named by language) instead of the fixed Original / English pair, and Export with subtitles chooses among the same tracks.
- Automatic translation after recording keeps its switch, now "Also translate subtitles": with HY-MT downloaded it translates the new transcript to the default target; without it, it falls back to Whisper English as today.
- Deleting a video recycles every `<video name>.<code>.srt` too, and the gallery hides them.

## Capabilities

### New Capabilities

- `subtitle-translation`: the HY-MT model store, licence confirmation and notice, the translation task, supported targets and default target rule.

### Modified Capabilities

- `gallery`: listing and delete cover every translation track; the translate action, transcript pane tracks and subtitle export track become multi-language (these requirements are introduced by `whisper-english-translation`, which must be archived first).
- `screen-recording`: automatic translation uses HY-MT and the default target when it is downloaded.
- `settings-window`: the automatic translation switch is renamed and describes both engines.

## Impact

- Backend: a new `translate.rs` store (shaped like `whisper.rs`, through `fetch.rs` and `onnx.rs`), a new `translate/` engine (prompt, tokenizer, greedy decoding with a KV cache), `transcript.rs` (`Track` gains `Translation(code)`, track discovery), `queue.rs` (`TaskSpec::Translate` gains an optional target, `QueueItem.detail`), `subtitle.rs` (track naming), `gallery.rs` (all `<stem>.<code>.srt` sidecars), `config.rs` (`hymt_license_accepted`), `snap.rs` / transcript chaining, `lib.rs`, `events.rs`.
- Dependencies: `tokenizers` (Hugging Face, with `fancy-regex` instead of the default Oniguruma) for the byte-level BPE `tokenizer.json`.
- Frontend: AI models (a translation card, the licence dialog, Downloaded models), About, the transcript pane, the subtitle dialog, the item menu, `lib/api.ts`, `lib/queue.ts`, a `lib/languages.ts` table, `lib/i18n.ts`.
- Disk and time: about 1.42 GB on disk; translating a 10-minute video on CPU is expected to take minutes (it runs in the background queue).
- Licensing: Snapr never redistributes the model (each user downloads it from Hugging Face after accepting); this change does not make a legal judgment beyond showing the terms.
