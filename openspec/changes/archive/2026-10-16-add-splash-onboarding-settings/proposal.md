# Proposal

## Why

snapr starts silently in the tray, so a new user gets no confirmation that it launched and no prompt to pick an output folder or hotkeys. The settings window is a plain single-column form with an OS title bar that does not match the polished, Raycast-style look the app is aiming for.

## What Changes

- On first run (no `config.toml` yet), show a small frameless splash (~1s), then a guided setup page in the same window. Pressing **Start** saves the config and hides to tray. Every later launch goes straight to the tray as today.
- `Config::load` no longer writes `config.toml` when missing; the file is only created by `save`, so "configured" means the user finished setup.
- Redesign the settings window (opened from the tray):
  - Left sidebar with groups: General, Preview, Hotkeys, About. Right pane shows the selected group as grouped cards (label + hint on the left, control on the right).
  - Frameless window: no OS title bar, a draggable top strip, and a single close button on the right that hides to tray.
  - Fixed size 1366x768, not resizable, capped to the monitor work area on small or high-DPI screens.
  - Dark theme with native Windows Mica backdrop (blurred dark background).
  - Changes save immediately (no Save/Cancel footer); invalid hotkeys are shown inline and not saved until fixed.
  - Language moves from header buttons to a select under General; Open config file and Open folder move to About.
- Checkboxes become switches for boolean settings.

## Capabilities

### New Capabilities
- `app-startup`: what the app shows on launch: first-run detection, splash, setup page, and silent-to-tray for configured users.
- `settings-window`: the settings window's chrome, size, layout, grouping, and save behavior.

### Modified Capabilities

## Impact

- `src-tauri/src/config.rs`: `load` stops writing the default file.
- `src-tauri/src/settings.rs`: `load` reports `first_run`; new command to switch the window to the settings size.
- `src-tauri/src/lib.rs`: show splash on first run; tray "Settings…" resizes/centers before showing.
- `src-tauri/tauri.conf.json`: `main` window becomes frameless, transparent, non-resizable, Mica, dark.
- `src-tauri/capabilities/default.json`: add `core:window:allow-start-dragging`.
- `src/App.tsx`: splash, setup page, sidebar layout, autosave.
- No new dependencies; uses existing shadcn `sidebar`, `item`, `switch`, `select` components.
