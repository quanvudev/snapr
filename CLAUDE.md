# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

snapr: a Windows-only tray screenshot app (fullscreen, window, area capture with global hotkeys and a preview toast).
Tauri 2 backend in `src-tauri/`, React 19 + TypeScript + Tailwind v4 + shadcn (base-ui, "base-vega") settings UI in `src/`.
It is a port of the v1 winit app at `V:\source\repos\snapr` (commit `8ac4086`).

## Commands

Use **bun**, never npm/pnpm.

- `bun run tauri dev`: run the app (starts Vite on port 1420, strict)
- `bun run tauri build`: release bundle
- `bunx tsc --noEmit`: type-check the frontend
- `cargo clippy` (in `src-tauri/`): lint the backend
- `bunx --bun shadcn@latest add <component>`: add a shadcn component to `src/components/ui/`

There are no tests.

## Architecture

The app runs **two event loops in one process**:

- **Tauri (tao) on the main thread** owns the tray, global hotkeys (`tauri-plugin-global-shortcut`), the settings webview window (label `main`) and config state.
- **A winit event loop on a dedicated thread** (`spawn_capture` in `src-tauri/src/lib.rs`, built `with_any_thread(true)`) owns the native capture overlay and the preview toast. They are drawn with softbuffer.
  - This split exists because Tauri window events carry no mouse or keyboard input, which the overlay needs.
  - Hotkey handlers send `Action` (Fullscreen / Pick(Window) / Pick(Area) / Text / Record) through an `EventLoopProxy` that is stored in managed `State`. There are five hotkey slots (`Hotkeys::combos()` is `[String; 5]`, order fullscreen, window, area, text, record; defaults `1`..`5`). `text` has a field-level `#[serde(default)]` so an older config without it stays unbound (empty) instead of colliding with its `record = "4"`.
  - Recording: `Action::Record` opens the window picker with `Snap.mode = Mode::Record`. The picked hwnd goes to `Recorder::start` (`recorder.rs`, `windows-capture`: WGC + H.264 MP4) as `Target::Window` instead of the save worker. In record mode `Snap::window_event` intercepts a left press whose `Overlay::hovered()` is `None` or a desktop window (`Progman`/`WorkerW`) and records the monitor under the cursor as `Target::Monitor`; the window screenshot picker uses `Mode::Shot`, so it is unaffected. The recorder lives in `State.recording` (`Mutex<Option<Recorder>>`) so the main thread can stop it: the record hotkey (while recording), the tray "Stop recording" item (`State.stop`) and Quit take it out of the mutex, then call `stop()`, which finalizes synchronously. On stop or window close the handler finishes the file and sends `Action::Recorded`, which clears `State.recording`, restores the tooltip and opens the toast. Never hold `State.recording` while calling `stop()`.
  - The loop never encodes: it hands each capture to a background save worker thread as a `Job` (`Save(Shot)` / `Text(Shot)`) (`std::sync::mpsc`, started in `spawn_capture`). When `padding.enabled`, the worker first pads the image with `capture::pad` (transparent, or the `padding.image` background scaled to cover), so the file, clipboard and toast all get the padded image; a background load error pads transparently and is shown in the tooltip after `refresh`. The worker saves the PNG, copies to the clipboard and reports back with `Action::Saved`, which opens the toast.
  - Text capture: `Action::Text` opens the area picker in `Mode::Text`; the worker (WinRT initialized MTA once) runs `ocr::extract`, copies the text with `arboard` regardless of `copy_to_clipboard`, and reports via `tauri-plugin-notification` from Rust (en/vi strings in `lib.rs`), never the toast. The text hotkey is registered only when `text_capture && State.ocr`; `State.ocr` caches `ocr::available()`, re-probed in `apply()` and in `show_settings()` (which re-applies when it changed). `settings::rows` skips an inactive or empty text slot (no error, no in_use, no duplicate).
  - The capture thread reads config from `State.cfg` (`RwLock`) and reports status and errors through the tray tooltip.

Backend modules:
- `capture.rs`, `overlay.rs` and `toast.rs` are v1 code kept verbatim.
  - Win32 capture uses `PrintWindow` + DWM bounds, with rounded corners added.
  - Saves a timestamped PNG and can also copy the image to the clipboard. `capture::unique_path(dir, ext)` gives the timestamp + `_N` name for both PNGs and recordings.
- `config.rs` holds the TOML config at `app_config_dir()/config.toml` (`%APPDATA%\com.quanvudev.snapr`).
  - The path is resolved in `setup` and stored in `State`.
  - `load` returns defaults without writing when the file is missing; only `save` creates it. A missing file means first run.
  - `parse_hotkey` maps `Win` → `Super`.
  - `record_fps` is read only through `Config::fps()`, which returns 60 for 60 and 30 for anything else; `recorder.rs` derives both the frame throttle and the encoder rate from it.
- `ocr.rs` wraps `Windows.Media.Ocr` (`windows` 0.62, same version as `windows-capture` pulls in): `available()` and `extract()` using the Windows profile languages, downscaling above `MaxImageDimension`.
- `settings.rs` holds the commands the settings window invokes, plus the HKCU `Run` registry autostart.

Settings window ↔ backend:
- The JSON contract uses snake_case fields, while command argument names are camelCase on the JS side.
- Commands: `load`, `defaults`, `resolve_dir`, `check`, `save`, `open_folder`, `open_config_file`, `open_settings`, `open_language_settings` (spawns `explorer ms-settings:regionlanguage`). `check` takes `text` (text slot active); `load` returns `ocr_available`.
- `save` is called on every change from the settings view (autosave; no Save/Cancel). The setup view saves once on Start.
- `save` applies immediately by calling `apply()` in `lib.rs`, which unregisters all hotkeys, re-registers them and updates the tooltip. There is no restart and no file polling.
- `check` reports hotkey errors (`invalid` / `needs_modifier` / `duplicate`) and `in_use`.
  - `in_use` is probed by test-registering the combo.
  - A combo snapr itself has registered counts as free.
- One frameless (`decorations: false`), transparent, Mica, non-resizable `main` window is used for three views: splash (480x300, first run only, shown from `setup`), setup (first run) and settings (sidebar: General / Capture / Preview / Recording / Hotkeys / About).
  - `show_settings` sizes it to `min(1366x768, monitor work area)`, centers, shows and emits `settings-open`. The tray "Settings…" item and the splash (via `open_settings`) both use it.
  - Drag via `data-tauri-drag-region`. The X button and Escape hide the window.
  - Closing the window hides it instead of exiting.
  - The frontend hides itself via `getCurrentWindow().hide()`.
- `src/App.tsx` only picks the view (splash / setup / settings) from `useSettings`.
- `lib/api.ts`: types and typed wrappers for every Tauri command; `invoke` is called only here. `lib/i18n.ts`: the `{ en, vi }` table `T` (default `vi`). `lib/hotkeys.ts`: `MODS`, `SLOTS`, `normalize`.
- `hooks/use-settings.ts` owns state, load, autosave (`persist`) and the `settings-open` listener; `hooks/use-hotkey-recorder.ts` owns key recording and Escape.
- `components/settings-provider.tsx` exposes it through `useSettingsContext()` (no prop drilling); `components/title-bar.tsx` is the drag strip + X.
- Screens live in `components/splash/`, `components/setup/`, `components/settings/` (groups in `general/`, `capture/`, `preview/`, `recording/`, `hotkeys/`, `about/`, one file per row). A new settings group = a folder + an entry in `settings/sections.ts`.
- Always dark (`class="dark"` on `<html>`).

## Conventions

- No code comments.
- Always build UI from the existing shadcn components in `src/components/ui/`. Don't hand-roll native `<input>`/`<button>` or custom equivalents.
- Keep it minimal: reuse v1 code over rewriting, no speculative abstractions.
- Add new Tauri plugin permissions to `src-tauri/capabilities/default.json`.
- Barrels: only screen roots (`components/splash|setup|settings/index.ts`) have an `index.ts`. Import a screen from outside via `@/components/<screen>`; inside a screen use relative paths, never its own barrel. `components/ui/` and `title-bar.tsx` never import screens.
