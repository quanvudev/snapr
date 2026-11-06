# Design

## Context

`audio::Capture` runs one `Mixer` thread per capture. Every 10 ms (`TICK`) it pulls `FRAMES` (480) stereo f32 frames at 48 kHz from the microphone `Source` and the system `Source`, adds them, clamps the sum and sends it as an i16 `Chunk`. It also accumulates the microphone's RMS for the `level` callback, which runs every `LEVEL_TICKS`. The panel runs two kinds of capture: a level-only monitor in Setup (`listen(app, mic, "", false)`), which is restarted by `swap_monitor` whenever the microphone changes in `record_select`, and the recording capture from `panel::audio`. Panel choices live in `PanelState` and are persisted to the config by `record_start`.

RNNoise works on mono frames of exactly 480 samples at 48 kHz, which is the size of one mixer tick.

## Goals / Non-Goals

**Goals:**
- Suppress microphone noise in both the recording and the Setup level meter, using the same code path.
- Add no download and no ONNX dependency, and add only negligible CPU.

**Non-Goals:**
- Processing system audio.
- Echo cancellation, automatic gain or a strength slider.
- Applying suppression to existing videos after recording (a possible later queue task).
- Changing the setting from the settings window (the panel stays its only UI, like the devices).

## Decisions

- **`nnnoiseless` (pure-Rust RNNoise port) over DeepFilterNet or the `rnnoise-c` bindings.** It needs no C build step and no model file, since its weights are compiled in, and its frame size matches `FRAMES`. DeepFilterNet sounds better but needs an ONNX model and much more CPU, which the user rejected for this iteration.
- **Run in `Mixer::run`, on the microphone buffer only, right after `take`.** The microphone frames are downmixed to mono, scaled to the i16 range RNNoise expects (`* 32767`), processed by one `DenoiseState`, scaled back and written to both channels. The processing happens before the RMS sum and before mixing, so the meter and the chunk both see the suppressed signal and system audio is never touched. Alternative: process inside the cpal callback. That was rejected because callback buffer sizes are not 480 and the callback runs on a real-time thread.
- **Mono downmix.** Almost all microphones are mono, and cpal's `stereo()` duplicates a mono input. One state halves the cost, and a stereo microphone loses only its stereo image, which is not useful for voice.
- **`Capture::start` gains `denoise: bool`.** `Mixer` holds `Option<Box<DenoiseState>>`, which is `None` when the flag is off or there is no microphone. The state is created on the mixer thread.
- **The choice lives in `PanelState.denoise`.** `record_select` gains a `denoise` argument. When it changes (like the microphone), `swap_monitor` restarts the Setup monitor with the new flag, which meets the "follows within half a second" requirement because a restart takes about 100 ms. `panel::audio` passes the flag to the recording capture, `record_start` saves it to `config.record_denoise`, and `open` reads it.
- **Config `record_denoise: bool`, `true` in `Config::default()`.** The struct is container-level `#[serde(default)]` (like `update_check`), so an older config loads with it on, as specified.
- **UI: a `DropdownMenuCheckboxItem` in the audio dropdown under the Microphone radio group**, disabled when `mic === ""`. The local `components/ui/dropdown-menu.tsx` lacks the checkbox item, so add it from the base-vega shadcn registry rather than hand-rolling one. Strings: en "Noise suppression", vi "Lọc tiếng ồn".

## Risks / Trade-offs

- [RNNoise can make music or singing into the microphone sound gated] → The checkbox turns it off, and the choice is remembered.
- [The first frame after a state is created contains a fade-in artifact] → It is only 10 ms at the start of the capture, and in a recording it falls inside the encoder's start-up, so it is accepted.
- [An underrun fills `take` with zeros mid-buffer, and RNNoise sees a short dropout] → This is the same dropout heard without suppression, so nothing gets worse.
- [Added CPU on the mixer thread] → RNNoise costs about 1% of one core at 48 kHz mono, well within the 10 ms tick.

## Migration Plan

This is additive. An older config gains `record_denoise = true` on its next save. An older build ignores the extra key (the config does not deny unknown fields), so rolling back needs no step.
