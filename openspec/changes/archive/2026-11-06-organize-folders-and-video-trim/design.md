# Design

## Context

Every writer resolves its folder in one of two ways today: `Config::dir()` (screenshots in `capture::write`, recordings in `recording::record`, `queue::clean_parts`, `gallery::guard`, `gallery_list`) or `source.parent()` (GIF, compress, subtitle, transcribe and translate tasks, `transcript_save`). `gallery::guard` requires the canonical parent to equal the canonical output folder, and the gallery pairs a video with `<stem>.srt` / `<stem>.<code>.srt` by name inside one directory listing. The video language store (`transcript/languages.rs`) is keyed by the lowercased canonical video path, so a moved video loses its language unless the key moves too.

`video.rs` already drives WinRT `MediaTranscoder` with a progress handler, a 100 ms status poll and `Cancel()`, writing through `queue::write_part`.

## Goals / Non-Goals

**Goals:**
- One place that answers "which folder does this output go to".
- Migration that can never lose a file: rename only, never copy plus delete, and keep a file in place when anything about it fails.
- Trim that keeps quality and audio without a hand-written decode/encode loop.

**Non-Goals:**
- User-configurable subfolder names, per-date folders or moving files between folders from the gallery.
- Frame-exact cuts at a non-keyframe without re-encoding (stream copy).
- Re-running the migration when the output folder is changed; files already at the new folder's root are just listed.

## Decisions

**A `library.rs` module owns the layout.** It has `images(cfg)`, `videos(cfg)` (`cfg.dir().join("Images" | "Videos")`), `folders(cfg)` (root, Images, Videos) and `migrate(app)`. `capture::write` uses `images`, `recording::record` uses `videos`, `GifTask::validated` sets `out_dir` from `images` instead of the source's parent. Every other task keeps `source.parent()`, so derived files follow their video whichever folder it is in and no task needs to know the layout. Writers already call `create_dir_all` on their folder (or get it here), so a deleted subfolder comes back on the next write. The folder names are fixed English strings: they are paths on disk, not UI, and must not change with the interface language.

**`gallery::guard` accepts three parents.** It canonicalizes each of `library::folders(cfg)` that exists and checks the file's canonical parent against them. Nested folders and other subfolders stay refused, so `..` tricks are still closed. `gallery_list` lists each folder separately with the existing logic (the transcript and translation hiding stays per folder, which is also what external players expect) and concatenates the results before sorting by date. `queue::clean_parts` runs over the three folders.

**Migration runs once, in `setup`, before `queue::spawn`.** A `folders_migrated: bool` config field with `#[serde(default)]` loads as `false` from an existing config, while `Config::default()` sets it to `true`, so a fresh install never migrates and a first run does not write the config early. When `false`, `migrate` lists the root once, moves each photo to `Images` and each video to `Videos` with `std::fs::rename` to `capture::stem_path(target, stem, ext)`, then renames each existing `Track::all()` file of that video to the new stem in the same folder and calls `languages::rename(old, new)` (a new helper: remove the old key, insert the new one, one save). If the video rename fails, its tracks are not touched. Errors are skipped per file. Afterwards it sets the flag and saves the config. It runs before the queue, hotkeys and gallery exist, so no task or capture can race it; a rename within one volume is instant, so even a large folder does not delay startup noticeably.

Alternative considered: run the move on every start (idempotent). Rejected because a user who deliberately keeps a file in the root would see it moved each time.

**Trim uses WinRT `MediaComposition`.** `TrimTask` (kind `trim`, `TaskSpec::Trim { path, start_ms, end_ms }`, validated like `GifTask`: `video::guarded_mp4`, `gif::probe`, `start_ms < end_ms <= duration` with the same 100 ms slack) builds a `MediaComposition` with one `MediaClip::CreateFromFileAsync(source)`, sets `TrimTimeFromStart` and `TrimTimeFromEnd`, and calls `RenderToFileWithProfileAsync(part, MediaTrimmingPreference::Precise, profile)`, where `profile` is `MediaEncodingProfile::CreateFromFileAsync(source)` with the video bitrate from `video::bitrate(w, h, fps, Quality::High)`, so the output keeps the source size and audio. Its progress and cancel follow `CompressTask`: progress handler into `Progress` (0–100), a 100 ms `Status` poll, `Cancel()` on the cancel flag. Output goes through `queue::write_part(dir, "<stem>_trimmed", "mp4", ..)`.

Alternatives considered: a decode/encode loop over `gif::Video` and the vendored encoder like `subtitle.rs` (full control, but a few hundred lines of A/V interleaving that `MediaComposition` already does with hardware encode); a stream copy through `IMFSinkWriter` without re-encoding (lossless and fast, but cuts snap to keyframes, which WGC recordings place far apart, so the start would be off by seconds).

**Subtitle tracks are cut after the video is renamed.** `write_part` returns the final path, so the track stem is the real output stem (including `_N`). For each `Track::all()` file of the source that exists, `parse_srt`, keep cues with `end > start_ms && start < end_ms`, clip to the range, subtract `start_ms`, `render_srt` and write through `write_part_to` to `track.path(output)`. A track failure is ignored, the video stays. The source's language goes to the new video through `languages::set`. The task then returns the video path and the existing `gallery-changed` refresh shows the tracks.

**Frontend reuses the GIF dialog's pieces.** `trim-dialog.tsx` shows the same `<video>` preview via `api.mediaSrc`, two-thumb `Slider` in ms with seek-on-drag and start / duration / end readout, without width / fps. `use-gallery.ts` gets `trimTarget` / `openTrim` / `closeTrim` / `trim`, `lib/queue.ts` gets `trimSpec` and `TASK_KINDS.trim`. The viewer header gets a Scissors button next to Export GIF and Compress, the item menu a Trim video entry for MP4.

## Risks / Trade-offs

- [Scripts or shortcuts point at old root paths] → The move happens once and is listed in the release notes; nothing is deleted.
- [Migration crashes half-way] → Each file is a single rename, and the flag is written only at the end, so the next start finishes the remaining files; files already moved are not in the root any more.
- [A moved video's track rename fails while the video moved] → The track stays in the root and lists as an other file; the video shows no transcript. Rare (same folder, same volume) and nothing is lost.
- [`MediaComposition` re-encodes] → Slower than a stream copy and slightly lossy; High bitrate keeps the loss invisible for screen recordings, and the original is never touched.
- [`MediaComposition` needs a UI-less WinRT context] → It works from an MTA worker thread like `MediaTranscoder`; if `RenderToFileAsync` turns out to need an STA, the task runs it on a short-lived STA thread.

## Migration Plan

Ships in one release. On first start the move runs before anything else; downgrading to an older build is safe for screenshots and recordings (the older gallery just ignores the subfolders), so no rollback step is needed beyond moving files back by hand if wanted.
