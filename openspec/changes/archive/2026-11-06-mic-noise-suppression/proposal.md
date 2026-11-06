# Proposal

## Why

Recordings made on laptops and in open offices pick up fan hum, keyboard clicks and room noise through the microphone, and the only fix today is editing the audio in another app. A local, model-free noise suppressor can clean the voice as it is recorded at almost no CPU cost. Snapr's audio already runs at 48 kHz in 480-frame (10 ms) ticks, which is exactly RNNoise's frame size.

## What Changes

- Noise suppression for the microphone, done with RNNoise through the pure-Rust `nnnoiseless` crate. There is no model download and no ONNX runtime. It applies only to the microphone signal, before that signal is mixed with system audio, so system audio is never altered.
- A "Noise suppression" checkbox item in the recording panel's audio dropdown, under the Microphone group. It is disabled when no microphone is selected.
- The setup panel's live level meter follows the suppressed signal, so the user can see the noise drop before recording. Toggling the item restarts the level monitor.
- A new config field `record_denoise` (on by default, field-level default so an older config loads with it on). It is saved with the other panel choices when a recording starts.
- The choice is locked while recording, like the devices.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `recording-panel`: device selection gains the noise suppression option and its effect on the level meter; remembered choices include it.
- `screen-recording`: the audio requirement states that, with noise suppression on, the microphone part of the track is suppressed and system audio is unchanged.

## Impact

- `src-tauri/Cargo.toml`: new dependency `nnnoiseless` (BSD-3-Clause).
- `src-tauri/src/audio.rs`: `Capture::start` takes a `denoise` flag, and `Mixer` runs a `DenoiseState` on the microphone buffer each tick.
- `src-tauri/src/panel.rs`: `PanelState.denoise`, `listen`, `record_select` (new argument), `record_start` (saves it), `open` (reads it).
- `src-tauri/src/config.rs`: `record_denoise`.
- Frontend: `lib/api.ts` (`recordSelect` signature, `PanelState`, `Config`), `components/panel/setup-panel.tsx`, `components/ui/dropdown-menu.tsx` (add `DropdownMenuCheckboxItem`), `lib/i18n.ts` (en/vi strings).
- `CLAUDE.md` audio and panel paragraphs.
