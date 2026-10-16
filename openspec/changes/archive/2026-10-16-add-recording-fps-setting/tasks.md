# Tasks

## 1. Backend

- [x] 1.1 Add `record_fps: u32` (default 30) and `Config::fps()` (60 if 60, else 30) to `config.rs`. Done when `cargo clippy` is clean and an old `config.toml` without `record_fps` loads with 30.
- [x] 1.2 In `recorder.rs`, add `fps` to `Recorder::start` and the handler flags, replace `INTERVAL` with `10_000_000 / fps` stored on the handler, and use `.frame_rate(fps)`. In `lib.rs::record()`, pass `cfg.fps()`. Done when `cargo clippy` in `src-tauri/` is clean.

## 2. Frontend

- [x] 2.1 Add `record_fps: number` to `Config` in `src/lib/api.ts`, add the `recording` / `fps` / `fpsHint` labels in `en` and `vi` in `src/lib/i18n.ts`, and add `"recording"` to `SectionId` in `src/hooks/use-settings.ts`. Done when `bunx tsc --noEmit` passes.
- [x] 2.2 Create `src/components/settings/recording/recording.tsx` and `fps-row.tsx`, using the `Select` pattern from `language-row.tsx` with options 30 / 60. Register the group in `settings/sections.ts` between Preview and Hotkeys with the `Video` icon. Done when `bunx tsc --noEmit` passes and the Recording group shows the current fps.

## 3. Check and docs

- [x] 3.1 Manual check (user): set 60 fps, record a scrolling window and confirm about 60 fps in the file properties. Switch fps during a recording and confirm that recording keeps its rate and the next one uses the new rate. An old config without `record_fps` records at 30.
- [x] 3.2 Update `CLAUDE.md`: the settings sidebar list (General / Preview / Recording / Hotkeys / About) and `record_fps` / `Config::fps()`. Done when the doc matches the code.

## Workflow follow-up

- Run `/opsx:archive` after review to merge the spec deltas into `openspec/specs/`.
