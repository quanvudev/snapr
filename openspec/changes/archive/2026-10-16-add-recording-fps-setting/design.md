# Design

## Context

See proposal.md for motivation and the specs for behavior. Today `recorder.rs` hardcodes 30 fps in two places:
- the throttle `INTERVAL: i64 = 330_000`, in 100 ns ticks;
- `.frame_rate(30)` on the encoder.

`Recorder::start(proxy, hwnd, path)` is called from `record()` in `lib.rs`, which already reads `State.cfg`. The settings UI autosaves every change through `update()`. Groups are a folder plus an entry in `settings/sections.ts`. `language-row.tsx` is the existing `Select` row pattern.

## Goals / Non-Goals

**Goals:**
- One value drives both the throttle and the encoder, so they can't disagree.
- A bad config value can never reach the encoder.

**Non-Goals:**
- A generic "recording options" struct. Add one when there is a second option.

## Decisions

### 1. `record_fps: u32` on `Config`, normalized in Rust
- Add `record_fps: u32` with default `30`. `#[serde(default)]` on `Config` covers old files.
- Add `Config::fps(&self) -> u32`. It returns `60` when `record_fps == 60` and `30` otherwise.
- `record()` passes `cfg.fps()`. The raw field is never used directly.

Alternative: a serde enum (`Fps30` / `Fps60`). It was rejected because it complicates the TS type and the JSON contract for two numbers.

### 2. `Recorder::start(proxy, hwnd, path, fps)`
- The fps goes into the handler `Flags`.
- The handler stores `interval = 10_000_000 / fps as i64` and builds the encoder with `.frame_rate(fps)`.
- `INTERVAL` is removed.

The value is fixed when the handler is created, so a running recording keeps its rate (spec: Changed during a recording).

### 3. Recording settings group
- New `components/settings/recording/recording.tsx`. It renders an `ItemGroup` with `fps-row.tsx`, one file per row as the project convention requires.
- The row reuses `language-row.tsx`'s `Select` pattern with the items `30` and `60`, shown as `30 fps` / `60 fps`. It calls `update({ ...cfg, record_fps: Number(v) })`.
- `SectionId` gains `"recording"`.
- `sections.ts` inserts it between Preview and Hotkeys, with the lucide `Video` icon.
- i18n keys:
  - `recording`: "Recording" / "Quay video";
  - `fps`: "Frame rate" / "Tốc độ khung hình";
  - `fpsHint`: "Higher is smoother, larger files" / "Cao hơn mượt hơn, file lớn hơn".

## Risks / Trade-offs

- [At 60 fps the `keep()` GPU readback for the toast frame runs twice as often] → Accept for now, and watch CPU in the manual check. If it is too slow, read back only every Nth sent frame.
- [Larger files at 60 fps with the encoder's default bitrate] → This is the user's explicit trade-off, and the row hint says so.
- [WGC delivers fewer than 60 frames when the window changes slowly] → Expected. Timestamps keep playback speed correct.

## Migration Plan

No migration is needed. Old configs load with `record_fps = 30`. Older builds ignore the extra field when rolled back.
