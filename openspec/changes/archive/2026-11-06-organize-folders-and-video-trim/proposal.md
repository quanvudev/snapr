# Proposal

## Why

Everything snapr makes lands in one flat output folder: screenshots, recordings, GIFs, compressed and subtitled copies and every `.srt` track. After a few weeks the folder is hard to browse in Explorer. Videos also cannot be shortened: the GIF dialog can pick a range, but an MP4 can only be exported whole, so cutting the start and end of a recording needs another tool.

## What Changes

- The output folder gets two fixed subfolders: `Images/` (screenshots and GIF exports) and `Videos/` (recordings with their `.srt` / `.<code>.srt` tracks beside them, so external players still pick up subtitles).
- Screenshots save into `Images/`, recordings into `Videos/`, and GIF exports into `Images/` whatever folder the source video is in. Compressed, subtitled, trimmed copies and transcripts stay in the source video's folder.
- The gallery lists the output folder root, `Images/` and `Videos/`; other subfolders are still ignored. File actions accept files directly inside any of those three folders.
- **BREAKING (file layout)**: on the first start after the update, photos and videos snapr can show that sit directly in the output folder are moved once into `Images/` / `Videos/`, along with each video's subtitle tracks and its stored language; a taken name gets a `_N` suffix. Other files stay in the root.
- New **Trim video**: a dialog for an MP4, opened from the viewer header and the context menu, with a preview and a two-thumb range slider like Export GIF. It queues a `trim` background task that writes `<stem>_trimmed.mp4` (with `_N` when taken) next to the source, keeps the audio, never touches the original, and writes every subtitle track of the source cut to the range and shifted to start at zero.

## Capabilities

### New Capabilities

- `output-layout`: where each kind of snapr output is written inside the output folder, and the one-time move of existing files into that layout.
- `video-trim`: cutting an MP4 to a range from the gallery as a background task, including its subtitle tracks.

### Modified Capabilities

- `gallery`: listing covers the root, `Images/` and `Videos/`; single and bulk file actions accept files in those three folders; transcripts and translations are matched within the video's own folder.
- `capture-output`: captures are saved into `Images/` instead of the output folder root.
- `screen-recording`: recordings are saved into `Videos/` instead of the output folder root.

## Impact

- Backend: a new `library.rs` (folder resolution and migration), `capture.rs`, `recording.rs`, `gif.rs` (output folder), `gallery.rs` (listing, `guard`), `queue.rs` (`clean_parts` over three folders, new `TaskSpec::Trim`), `transcript/languages.rs` (rename a key), `config.rs` (`folders_migrated` flag), a new `trim.rs` task using WinRT `Windows.Media.Editing` (`MediaComposition`, `MediaClip`), which adds the `Media_Editing` feature to the `windows` dependency.
- Frontend: `gallery/trim-dialog.tsx`, viewer header and item menu entries, `use-gallery.ts` (`trimTarget`), `lib/queue.ts` (`trimSpec`, `TASK_KINDS.trim`), `lib/api.ts`, i18n en/vi.
- Users' existing files move on disk once; shortcuts or scripts pointing at old root paths will no longer find them.
