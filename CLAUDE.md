# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Snapr: a Windows-only tray screenshot app (fullscreen, window, area capture with global hotkeys and a preview toast).
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
  - The capture overlay (window/area/text/record pickers) covers all monitors (`overlay::desktop()` stitches every monitor into one frame over the union rect), while fullscreen capture, the toast and the record-picker desktop fallback stay on the cursor's monitor.
  - This split exists because Tauri window events carry no mouse or keyboard input, which the overlay needs.
  - Hotkey handlers send `Action` (Fullscreen / Pick(Window) / Pick(Area) / Text / Record) through an `EventLoopProxy` that is stored in managed `State`. There are five hotkey slots (`Hotkeys::combos()` is `[String; 5]`, order fullscreen, window, area, text, record; defaults `1`..`5`). `text` has a field-level `#[serde(default)]` so an older config without it stays unbound (empty) instead of colliding with its `record = "4"`.
  - Recording: `Action::Record` opens the window picker with `Snap.mode = Mode::Record`. The picked hwnd goes to `Recorder::start` (`recorder.rs`, `windows-capture`: WGC + H.264 MP4) as `Target::Window` instead of the save worker. In record mode `Snap::window_event` intercepts a left press whose `Overlay::hovered()` is `None` or a desktop window (`Progman`/`WorkerW`) and records the monitor under the cursor as `Target::Monitor`; the window screenshot picker uses `Mode::Shot`, so it is unaffected. The recorder lives in `State.recording` (`Mutex<Option<Recorder>>`) so the main thread can stop it: the record hotkey (while recording), the tray "Stop recording" item (`State.stop`) and Quit take it out of the mutex, then call `stop()`, which finalizes synchronously. On stop or window close the handler finishes the file and sends `Action::Recorded`, which clears `State.recording`, restores the tooltip and opens the toast. Never hold `State.recording` while calling `stop()`.
  - The loop never encodes: it hands each capture to a background save worker thread as a `Job` (`Save(Shot)` / `Text(Shot)`) (`std::sync::mpsc`, started in `spawn_capture`). When `padding.enabled`, the worker first pads the image with `capture::pad` (transparent, or the `padding.image` background scaled to cover), so the file, clipboard and toast all get the padded image; a background load error pads transparently and is shown in the tooltip after `refresh`. The worker saves the PNG, reports back with `Action::Saved` (which opens the toast), then copies to the clipboard. It caches the decoded `padding.image` in a `BackgroundCache` keyed by path + modified time.
  - Text capture: `Action::Text` opens the area picker in `Mode::Text`; the worker (WinRT initialized MTA once) runs `ocr::extract`, copies the text with `arboard` regardless of `copy_to_clipboard`, and reports via `tauri-plugin-notification` from Rust (en/vi strings in `lib.rs`), never the toast. The text hotkey is registered only when `text_capture && State.ocr`; `State.ocr` caches `ocr::available()`, re-probed in `apply()` and in `show_settings()` (which re-applies when it changed). `settings::rows` skips an inactive or empty text slot (no error, no in_use, no duplicate).
  - The capture thread reads config from `State.cfg` (`RwLock`) and reports status and errors through the tray tooltip.

Backend modules:
- `capture.rs`, `overlay.rs` and `toast.rs` come from v1. `overlay.rs` and `toast.rs` re-apply their physical size and position after `create_window` (mixed-DPI fix), and the overlay spans all monitors.
  - Win32 capture uses `PrintWindow` + DWM bounds, with rounded corners added.
  - Saves a timestamped PNG and can also copy the image to the clipboard. `capture::unique_path(dir, ext)` gives the timestamp + `_N` name for both PNGs and recordings.
- `config.rs` holds the TOML config at `app_config_dir()/config.toml` (`%APPDATA%\com.quanvudev.snapr`).
  - The path is resolved in `setup` and stored in `State`.
  - `load` returns defaults without writing when the file is missing; only `save` creates it. A missing file means first run.
  - `parse_hotkey` maps `Win` → `Super`.
  - `record_fps` is read only through `Config::fps()`, which returns 60 for 60 and 30 for anything else; `recorder.rs` derives both the frame throttle and the encoder rate from it.
- `ocr.rs` wraps `Windows.Media.Ocr` (`windows` 0.62, same version as `windows-capture` pulls in): `available()` and `extract()` using the Windows profile languages, downscaling above `MaxImageDimension`.
- `settings.rs` holds the commands the settings window invokes, plus the HKCU `Run` registry autostart.
- `gallery.rs` lists the output folder (png/mp4 only, no subfolders, `date = min(created, modified)`, newest first) and holds the gallery commands. `guard(app, path)` requires the canonical parent to equal the canonical output folder and a png/mp4 extension; every command and scheme goes through it, so `..\` or outside paths fail. `gallery_delete` uses `SHFileOperationW` (Recycle Bin); `gallery_copy` refuses mp4.
  - Two async URI schemes are registered in `run`: `thumb` returns Windows Shell thumbnails (`IShellItemImageFactory`) from an STA worker pool, as PNG with `Cache-Control: max-age=3600`; `media` serves the full file with `Range` (206 + `Content-Range`). Both return 404 on a guard or Shell failure.

Settings window ↔ backend:
- The JSON contract uses snake_case fields, while command argument names are camelCase on the JS side.
- Commands: `load`, `defaults`, `resolve_dir`, `check`, `save`, `open_folder`, `open_config_file`, `open_settings`, `open_language_settings` (spawns `explorer ms-settings:regionlanguage`). `check` takes `text` (text slot active); `load` returns `ocr_available`. Gallery commands (`gallery_list`, `gallery_open`, `gallery_reveal`, `gallery_copy`, `gallery_delete`) live in `gallery.rs`.
- `save` is called on every change from the settings view (autosave; no Save/Cancel). The setup view saves once on Start.
- `save` applies immediately by calling `apply()` in `lib.rs`, which unregisters all hotkeys, re-registers them and updates the tooltip. There is no restart and no file polling.
- `check` reports hotkey errors (`invalid` / `needs_modifier` / `duplicate`) and `in_use`.
  - `in_use` is probed by test-registering the combo.
  - A combo snapr itself has registered counts as free.
- One frameless (`decorations: false`), transparent, Mica, non-resizable `main` window is used for three views: splash (480x300, first run only, shown from `setup`), setup (first run) and settings (sidebar: Gallery / Settings; Settings is one scrollable page stacking General / Capture / Preview / Recording / Hotkeys / About via `settings/preferences/preferences.tsx`; Gallery is the only `wide` one).
  - `show_settings(app, section)` sizes it to `min(1366x768, monitor work area)`, centers, shows and emits `settings-open` with the section id. The tray menu is Gallery (`"gallery"`), Settings… (`"settings"`), Open folder, Stop recording, Quit; `open_settings` and the splash also pass `"settings"`.
  - `Saved` and `Recorded` emit `gallery-changed` to `main`.
  - Drag via `data-tauri-drag-region`. The X button and Escape hide the window.
  - Closing the window hides it instead of exiting.
  - The frontend hides itself via `getCurrentWindow().hide()`.
- `src/App.tsx` only picks the view (splash / setup / settings) from `useSettings`.
- `lib/api.ts`: types and typed wrappers for every Tauri command; `invoke` is called only here. `lib/i18n.ts`: the `{ en, vi }` table `T` (default `vi`). `lib/hotkeys.ts`: `MODS`, `SLOTS`, `normalize`. `lib/app-info.ts`: `APP_INFO` (name, author, links) used by the sidebar, splash, About and i18n.
- `hooks/use-settings.ts` owns state, load, autosave (`persist`) and the `settings-open` listener (`load(true, section)` selects the section); `hooks/use-hotkey-recorder.ts` owns key recording and skips `defaultPrevented` events (the gallery viewer handles its own Escape).
- `lib/gallery.ts` (`groupBy(items, period)` driven by the `PERIODS` mapping for day / month / year, `matches`, `isViewable`) and `hooks/use-gallery.ts` (items, filter, period, collapsed group keys (a `Set`, in memory only, kept across reloads), viewer index over photos and videos only (`kind: "other"` files are skipped by navigation, strip and delete-in-viewer), copy/delete; reloads on mount, `settings-open` and `gallery-changed`; delete moves the viewer to next, else previous, else closes).
- `components/settings-provider.tsx` exposes it through `useSettingsContext()` (no prop drilling); `components/title-bar.tsx` is the drag strip + X.
- Screens live in `components/splash/`, `components/setup/`, `components/settings/` (groups in `gallery/`, `general/`, `capture/`, `preview/`, `recording/`, `hotkeys/`, `about/`, one file per row). A new settings group = a folder + an entry in `settings/sections.ts`.
  - `gallery/`: `gallery.tsx` (type filter and Day / Month / Year ToggleGroups, `Empty`), `gallery-group.tsx` (collapsible day / month / year group: header chevron + label + count is the `Collapsible` trigger, the grid is the content), `gallery-item.tsx` (`other` tiles show the Shell icon plus the file name and do not open the viewer; tiles via `convertFileSrc(path, "thumb")`, `loading="lazy"`, video marker), `item-menu.tsx` (ContextMenu; Copy image only on photos), `delete-dialog.tsx` (AlertDialog), `viewer.tsx` (image fitted or `<video controls>` via `convertFileSrc(path, "media")`; capture-phase keydown for Left/Right/Escape with `preventDefault`, plus `viewer-strip.tsx`: a horizontal `ScrollArea` of the filtered items' thumbnails that centers and rings the active one and selects on click; `thumb.tsx` is the tile content shared with the grid).
  - `wide: true` in `settings/sections.ts` widens the content pane.
- `main.tsx` disables the WebView context menu and F5/Ctrl+R/Ctrl+P, but only in production builds.
- Theme is `config.theme` (`system` / `light` / `dark`, default `system`): `hooks/use-theme.ts` toggles `class="dark"` on `<html>` (following `prefers-color-scheme` for `system`) and calls `api.setWindowTheme` so Mica matches. The capture overlay and toast are not themed.

## Conventions

- No code comments.
- Always build UI from the existing shadcn components in `src/components/ui/`. Don't hand-roll native `<input>`/`<button>` or custom equivalents.
- Keep it minimal: reuse v1 code over rewriting, no speculative abstractions.
- Add new Tauri plugin permissions to `src-tauri/capabilities/default.json`.
- Barrels: only screen roots (`components/splash|setup|settings/index.ts`) have an `index.ts`. Import a screen from outside via `@/components/<screen>`; inside a screen use relative paths, never its own barrel. `components/ui/` and `title-bar.tsx` never import screens.
