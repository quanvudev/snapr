# Tasks

## 1. Output layout

- [x] 1.1 Add `src-tauri/src/library.rs` with `images(cfg)`, `videos(cfg)` and `folders(cfg)` (root, `Images`, `Videos`) and register it in `lib.rs`; verify `cargo clippy` is clean
- [x] 1.2 Save screenshots through `library::images` in `capture::write` and recordings through `library::videos` in `recording::record` (both `create_dir_all`); verify in `bun run tauri dev` that a fullscreen shot lands in `<output>\Images` and a recording in `<output>\Videos`, each created when missing
- [x] 1.3 Set `GifTask::validated`'s `out_dir` from `library::images` instead of the source's parent; verify a GIF export of a video in `Videos` is written to `Images`
- [x] 1.4 Run `queue::clean_parts` over `library::folders`; verify a leftover `Videos\x.mp4.part` is deleted on start
- [x] 1.5 Update CLAUDE.md (new `library.rs` entry, capture / recording / GIF output folders) and verify it names `Images` and `Videos`

## 2. Gallery over three folders

- [x] 2.1 Make `gallery::guard` accept a canonical parent equal to any existing folder of `library::folders`; verify with `bun run tauri dev` that open / reveal / pin / delete work on `Images\` and `Videos\` files and that `old\d.png` and `Images\x\e.png` paths are refused
- [x] 2.2 Make `gallery_list` list each of the three folders with the existing per-folder transcript and translation hiding and merge them before sorting; verify the "Mixed folder", "Nested folder" and "Transcript in another folder" scenarios of the gallery delta
- [x] 2.3 Verify the thumbnail and media URI schemes, `gallery_text`, `transcript_read` / `transcript_save`, Export GIF, Compress, Transcribe, Translate and Export with subtitles all work on a video in `Videos` and write their outputs beside it (GIF in `Images`)
- [x] 2.4 Update the gallery paragraph of CLAUDE.md (listing and `guard` over three folders) and verify it matches the code

## 3. One-time migration

- [x] 3.1 Add `folders_migrated` to `Config` (`#[serde(default)]` so an old config loads `false`, `true` in `Config::default()`); verify an old config without the field loads `false` and a fresh install's first save writes `true`
- [x] 3.2 Add `languages::rename(old, new)` in `transcript/languages.rs` (move the key, one save); verify `cargo clippy` is clean
- [x] 3.3 Implement `library::migrate`: photos to `Images`, videos to `Videos` via `rename` to `capture::stem_path`, then each existing `Track::all()` file renamed to the new stem and `languages::rename`; per-file errors skipped, flag set and config saved at the end; call it in `setup` before `queue::spawn`. Verify with a copy of a real output folder: the "Mixed root", "Name taken", "Language follows the video", "Only once" and "Locked file" scenarios of `output-layout`
- [x] 3.4 Document the migration and `folders_migrated` in CLAUDE.md (config section and `library.rs`) and verify it matches the code

## 4. Trim task (backend)

- [x] 4.1 Add the `Media_Editing` feature to the `windows` dependency in `src-tauri/Cargo.toml`; verify `cargo build` succeeds
- [x] 4.2 Add `src-tauri/src/trim.rs` with `TrimTask::validated(app, path, start_ms, end_ms)` (`video::guarded_mp4`, `gif::probe`, range checks with the GIF slack) and `TaskSpec::Trim { path, start_ms, end_ms }` in `queue.rs` / `into_task`; verify `queue_add` refuses an empty range and an end past the duration
- [x] 4.3 Implement `Task::run` with `MediaComposition` + `MediaClip` trims, `RenderToFileWithProfileAsync` (Precise, profile from the source with `video::bitrate(.., Quality::High)`), progress handler, 100 ms status poll and `Cancel()`, writing through `queue::write_part` to `<stem>_trimmed.mp4`; verify a 0:05–0:40 trim of a recording lasts about 35 s with matching audio, a silent source gives no audio track, a second trim is `_trimmed_1.mp4`, and a cancel leaves no `.part`
- [x] 4.4 After the video is written, cut every existing source track (overlapping cues, clipped, shifted, renumbered, empty file when none left) to `track.path(output)` through `write_part_to` and copy the source language with `languages::set`; add a unit test for the cue cut (overlap, clip, shift, out-of-range dropped) and verify `cargo test --lib trim` passes
- [x] 4.5 Add `trim` to the queue kinds in CLAUDE.md (`queue.rs` paragraph and a `trim.rs` entry) and verify it matches the code

## 5. Trim dialog (frontend)

- [x] 5.1 Add `trimSpec` and `TASK_KINDS.trim` to `src/lib/queue.ts` and the `trim` spec type to `src/lib/api.ts`, plus en/vi strings in `src/lib/i18n.ts` (Trim video / Cắt video, dialog labels, task label); verify `bunx tsc --noEmit` passes
- [x] 5.2 Add `src/components/settings/gallery/trim-dialog.tsx` (file name, `<video>` preview via `api.mediaSrc`, two-thumb `Slider` in 100 ms steps with seek-on-drag, start / duration / end, Cancel / Trim disabled for the whole range) and `trimTarget` / `openTrim` / `closeTrim` / `trim` in `use-gallery.ts`; verify the "Trim dialog picks a range" scenarios in `bun run tauri dev`
- [x] 5.3 Add a Trim video button to the viewer header (MP4 only, beside Export GIF and Compress; the viewer is paused while the dialog is open) and a Trim video entry to `item-menu.tsx` (MP4 only); verify the "Trim video from the gallery" scenarios
- [x] 5.4 Verify the queue sheet shows a running trim as Trim video with progress and cancel, Retry re-queues a failed trim, and the gallery shows the new video with its transcript badges when it finishes
- [x] 5.5 Document the trim dialog in the gallery section of CLAUDE.md and verify it matches the code

## 6. Integration

- [x] 6.1 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`; verify both are clean
- [x] 6.2 End to end on a copy of a real output folder: start the new build, check the migration result, take a screenshot and a recording, transcribe and trim the recording, delete the trimmed copy, and verify each file is where `output-layout` and `video-trim` say and that deleting removes its tracks
