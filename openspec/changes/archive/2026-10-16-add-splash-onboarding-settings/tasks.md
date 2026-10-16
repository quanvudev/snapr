# Tasks

## 1. Backend: first-run and window states

- [x] 1.1 In `config.rs`, make `Config::load` return defaults without writing `config.toml` when it is missing (still create the output dir); remove `DEFAULT_TOML` if unused. Verify: delete `%APPDATA%\com.quanvudev.snapr\config.toml`, launch, quit; the file is still absent.
- [x] 1.2 In `settings.rs`, add `first_run: bool` (`!state.path.exists()`) to `Loaded` and an `open_settings` command that calls `show_settings`; register it in `generate_handler!`. Verify: `cargo clippy` in `src-tauri/` is clean.
- [x] 1.3 In `lib.rs`, add `show_splash` (480x300, center, show) and `show_settings` (size `min(1366x768, work area / scale)`, center, show, focus, emit `settings-open`); call `show_splash` from `setup` when the config file is missing, and use `show_settings` in the tray "Settings…" handler. Verify: `cargo clippy` is clean.
- [x] 1.4 In `tauri.conf.json`, set the `main` window to 480x300, `decorations: false`, `transparent: true`, `resizable: false`, `shadow: true`, `theme: "Dark"`, `windowEffects: { "effects": ["mica"] }`, drop `minWidth`/`minHeight`; add `core:window:allow-start-dragging` to `capabilities/default.json`. Verify: `bun run tauri dev` starts without config/capability errors.

## 2. Frontend: shell, theme and splash

- [x] 2.1 Force dark: add `class="dark"` to `<html>` in `index.html`, set the title to `snapr`, remove the `prefers-color-scheme` effect from `App.tsx`, and give the root a translucent dark background in `App.css`. Verify: with Windows in light mode the window renders dark with Mica showing through.
- [x] 2.2 Add `view` state (`splash | setup | settings`) driven by `load().first_run`; show splash only on the first load of the page lifetime when `first_run`, then after ~1s `invoke("open_settings")` and switch to setup; `settings-open` reloads and picks setup/settings without splash. Verify: first launch shows splash then setup; tray "Settings…" never shows the splash.
- [x] 2.3 Build the splash: centered app icon (import `src-tauri/icons/128x128.png`), "snapr", version via `getVersion()`, `Spinner`. Verify: splash matches the wireframe at 480x300 with no title bar.
- [x] 2.4 Build the shared frameless header: `data-tauri-drag-region` strip with a ghost icon `Button` (lucide `X`) on the right that calls `hide()`; keep Escape → `hide()`. Verify: dragging the strip moves the window, X and Escape hide it.

## 3. Frontend: autosave and settings layout

- [x] 3.1 Replace `save`/`canSave`/footer with `persist(next)`: update state, await `check` for the new hotkeys, invoke `save` only if no row has an error, show the inline failed message on error. Folder and preview-seconds inputs keep a local draft and persist on blur/Enter. Verify: toggling clipboard updates `config.toml`; typing a folder writes nothing until blur; a duplicate hotkey writes nothing and shows the error.
- [x] 3.2 Build the settings view with `SidebarProvider`/`Sidebar`/`SidebarMenuButton` (General, Preview, Hotkeys, About; General selected on open and on `settings-open`) and a scrolling right pane with a centered `max-w-3xl` column of `ItemGroup`/`Item` cards. Verify: each group shows exactly the rows listed in `specs/settings-window` and the window is 1366x768 on a 1080p/100% screen.
- [x] 3.3 Fill the groups using existing components only: General (folder input + Browse + Open + resolved path, `Switch` clipboard, `Switch` autostart, `Select` language), Preview (seconds input, 0 = off hint), Hotkeys (modifier `Switch`/`Checkbox` row, three hotkey rows with combo and status, Reset to defaults), About (icon, name, version, Open config file, Open folder). Add any new `en`/`vi` strings to `T`. Verify: `bunx tsc --noEmit` passes and both languages render every label.

## 4. Frontend: setup page

- [x] 4.1 Build the setup view: frameless header, welcome title/subtitle, the General rows plus modifiers and the three hotkeys, and a primary Start button disabled while any hotkey has an error; Start invokes `save` (even with no changes) and hides. Rows edit local state only until Start. Verify: Start with defaults writes `config.toml` and hides; next launch shows no window; closing setup leaves no config and the next launch shows the splash again.

## 5. Integration and docs

- [x] 5.1 Update `CLAUDE.md` Architecture/Settings sections: first-run detection, `open_settings` command, frameless window states, autosave replacing Save/Cancel. Verify: the doc names match the code.
- [x] 5.2 End-to-end check with `bun run tauri dev`: first run (splash → setup → Start → tray), relaunch (tray only), tray Settings (sidebar, autosave, hotkeys apply immediately), and at 150% scaling the window fits the work area. Verify: all four flows behave per `specs/app-startup` and `specs/settings-window`.

## Workflow follow-up

- Archive the change after review.
