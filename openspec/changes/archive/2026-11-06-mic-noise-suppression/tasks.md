# Tasks

## 1. Backend: suppression in the mixer

- [x] 1.1 Add `nnnoiseless` to `src-tauri/Cargo.toml` and verify `cargo build` in `src-tauri/` succeeds
- [x] 1.2 In `audio.rs`, add `denoise: bool` to `Capture::start` and an `Option<Box<DenoiseState>>` to `Mixer` (created on the mixer thread only when `denoise` and a microphone exist); after `take` on the microphone, downmix to mono, scale by 32767, run `process_frame`, scale back and write both channels, before the RMS sum and the mix. Unit-test a helper on a synthetic 480-frame buffer (white noise in, lower RMS out; `denoise` off leaves it unchanged) and verify `cargo test --lib audio` passes
- [x] 1.3 Add `record_denoise: bool` (`true` in `Config::default()`) to `config.rs` and verify that a config TOML without the key loads with it `true` (unit test in `config.rs`)

## 2. Backend: panel wiring

- [x] 2.1 Add `denoise` to `PanelState`; `open` reads `cfg.record_denoise`; `listen` takes and forwards the flag; `panel::audio` passes it to the recording capture. Verify `cargo clippy` is clean
- [ ] 2.2 Add a `denoise: bool` argument to `record_select`, restart the Setup monitor through `swap_monitor` when the microphone or `denoise` changes, and save `cfg.record_denoise` in `record_start`. Verify that a run of `bun run tauri dev`, unticking and recording, then reopening the panel shows it unticked

## 3. Frontend

- [x] 3.1 Add `DropdownMenuCheckboxItem` to `src/components/ui/dropdown-menu.tsx` from the base-vega shadcn registry (`bunx --bun shadcn@latest add dropdown-menu`, keeping local changes, or copy only that component) and verify `bunx tsc --noEmit` passes
- [x] 3.2 Update `lib/api.ts` (`Config.record_denoise`, `PanelState.denoise`, `recordSelect(mic, camera, speaker, mode, denoise)`) and add en "Noise suppression" / vi "Lọc tiếng ồn" to `lib/i18n.ts`; verify `bunx tsc --noEmit` passes
- [ ] 3.3 In `components/panel/setup-panel.tsx`, add the checkbox item under the Microphone radio group in the audio dropdown, disabled when no microphone is selected, held in state and sent through `recordSelect`. Verify in the app that the level meter drops on fan or keyboard noise while it is ticked and rises when unticked

## 4. Docs and integration

- [x] 4.1 Update the `audio.rs`, `panel.rs` and `config.rs` paragraphs in `CLAUDE.md` (`denoise` flag, `record_denoise`, `record_select` argument) and verify that they name the real identifiers
- [ ] 4.2 End-to-end: record 20 s with a fan or typing near the microphone and a YouTube video playing through system audio, once ticked and once unticked; verify that the ticked file has clearly less noise, the video audio sounds identical in both, and audio stays in sync

## Workflow follow-up

- Archive the change with `/opsx:archive` after review.
