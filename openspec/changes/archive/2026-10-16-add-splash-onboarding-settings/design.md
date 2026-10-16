# Design

## Context

The `main` webview window is declared in `tauri.conf.json` (500x600, hidden, decorated) and only shown from the tray "Settings…" item, which also emits `settings-open`. `Config::load` writes `DEFAULT_TOML` when `config.toml` is missing, so today there is no first-run signal. The whole UI is `src/App.tsx`; `save` validates hotkeys, writes the file, toggles autostart and calls `apply()`. Behaviour is specified in `specs/app-startup` and `specs/settings-window`.

## Goals / Non-Goals

**Goals:**
- One window (`main`) serves splash, setup and settings; no second window.
- First-run state derived from the file system, no new flag or field.
- Window sizing and showing live in Rust so the frontend needs no window-size permissions.

**Non-Goals:**
- Splash on later launches, or distinguishing autostart from manual launch.
- Custom logo artwork (uses the existing app icon).
- Light theme, single-instance handling, Windows 10 acrylic tuning.

## Decisions

**First run = `!path.exists()`, computed live.** `Config::load` returns defaults without writing when the file is missing. `settings::load` adds `first_run: !state.path.exists()` to `Loaded`. Because it is computed on every `load`, a user who closes setup and later opens Settings from the tray lands on the setup page again, and after Start it flips to false with no extra state. Alternative: a `setup_done` field in the TOML; rejected, it duplicates what file existence already says.

**Window states driven from Rust.** Two small helpers in `lib.rs`:
- `show_splash(app)`: called in `setup` when the config file is missing; size 480x300, center, show.
- `show_settings(app)`: size `min(1366x768, monitor work area / scale factor)`, center, show, focus, emit `settings-open`. Used by the tray item and by a new `open_settings` command the splash calls after its timer.

`tauri.conf.json` keeps the window hidden at 480x300 with `decorations: false`, `transparent: true`, `resizable: false`, `shadow: true`, `theme: "Dark"`, `windowEffects: { effects: ["mica"] }`. Alternative: `setSize` from JS; rejected because it needs extra window permissions and duplicates the tray path.

**Frontend view selection.** `App.tsx` holds `view: "splash" | "setup" | "settings"`. On mount, `load` → if `first_run` and the window was just shown by `show_splash`, render splash, wait ~1s, `invoke("open_settings")`, then render setup. `settings-open` reloads and picks `setup` or `settings` from `first_run`. To tell "fresh splash" from "tray open", the splash is rendered only on the first `load` of the page lifetime when `first_run` is true; every `settings-open` skips it.

**Layout from existing shadcn components.** `SidebarProvider` + `Sidebar` + `SidebarMenuButton` for groups, `Item`/`ItemGroup` (variant `muted`) for card rows, `Switch` for booleans, `Select` for language, `Button variant="ghost" size="icon"` with lucide `X` for close. Drag strip: `data-tauri-drag-region` on the sidebar header and the content title bar; add `core:window:allow-start-dragging` to `capabilities/default.json`. Content column `max-w-3xl mx-auto`, right pane scrolls independently. Setup page reuses the same row components in one column, no sidebar.

**Autosave.** A single `persist(next)` replaces `save`: it sets local state, then if every `check` row is error-free it invokes `save` with the new config and autostart; on failure it shows the inline error. Switches, selects, hotkey recording, modifier toggles and reset call it directly. The folder and preview-seconds inputs keep a local draft and call it on blur/Enter, so partial paths never reach `save` (which runs `create_dir_all`). Because `check` is async, `persist` awaits `check` for the new hotkeys before deciding. The backend `save` already rejects invalid hotkeys, which is the safety net.

**Forced dark.** Drop the `prefers-color-scheme` effect and set `class="dark"` on `<html>` in `index.html`. Body/root background is a translucent dark token so Mica shows through.

## Risks / Trade-offs

- [Mica is Windows 11 only; on Windows 10 a transparent window would be see-through] → root background uses a mostly opaque dark color (e.g. `oklch(0.18 0 0 / 0.85)`), so Windows 10 shows a solid dark window and Windows 11 still shows the tint of Mica.
- [Mica tint is subtle compared to the Raycast screenshot] → acceptable; switching to `acrylic` is a one-word config change if a stronger blur is wanted, at the cost of drag lag on some machines.
- [Autosave fires `save` → `apply()` on each change, re-registering all hotkeys] → cheap (three shortcuts); text inputs only save on blur.
- [Splash timer races with tray "Settings…" during the first second] → `show_settings` is idempotent; the splash view is skipped once `settings-open` arrives.
- [Existing users keep their `config.toml`] → they never see setup, which matches the spec. A user with a config written by the old auto-write behavior is treated as configured.

## Migration Plan

No data migration. Rollback is reverting the change; an old build would recreate `config.toml` on launch as before.
