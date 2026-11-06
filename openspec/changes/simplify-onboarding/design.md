# Design

## Context

Today `setup` in `lib.rs` calls `settings::show_splash` on first run. That shows the single `main` window at 480x300, which is created from `tauri.conf.json` with Mica and a shadow. `use-settings.ts` then renders the `splash` view, waits a fixed `setTimeout(1000)` and calls `api.openSettings()`. That goes through `show_settings`, which resizes `main` to `min(1366x768, work area)` and switches the view to `setup`.

`Setup` reuses the settings rows: `FolderRow`, `ClipboardRow`, `StartupRow`, `LanguageRow` and the full `HotkeyRows`. While the view is `setup`, `update` only changes local state; `start` calls `api.save` and then `api.hide`.

The recording panel already uses the pattern this change needs:
- `panel.rs` builds frameless, transparent, shadowless webview windows on demand with `index.html?view=...`;
- `main.tsx` picks the app for each `view`;
- the panel and camera apps clear the `html` background so the window is transparent.

## Goals / Non-Goals

**Goals:**
- A splash with nothing around the logo. Because `main` has Mica and a shadow, the splash cannot reuse that window.
- The splash waits for real work (the `main` webview being ready) instead of a fixed timer, with a short floor so it never just flickers.
- A setup window sized to its content.

**Non-Goals:**
- A splash on normal launches. Those still go straight to the tray.
- A multi-step wizard, animations beyond fades, or new settings.
- Changing `config.toml`, defaults or the settings window itself.

## Decisions

### A separate `splash` window, not `main` with its effects switched off
`lib.rs` builds a `splash` `WebviewWindow` on first run (`index.html?view=splash`):
- 240x240 logical, centered, `decorations(false)`, `transparent(true)`, `shadow(false)`;
- no window effects, `skip_taskbar(true)`, `resizable(false)`, visible.

`main` keeps loading hidden in parallel.

Alternative: clear `main`'s effects with `set_effects(None)` and `set_shadow(false)`, then restore them for setup. Rejected because:
- switching Mica and the shadow at runtime makes the window flash while it resizes;
- the splash cannot outlive `main`'s load, since it is drawn by that same page.

The splash view is a new `components/splash/splash-app.tsx`. It clears the `html` background like `PanelApp` does, then shows the `128x128@2x.png` icon at 96 px (scale and fade in) and a `Spinner` under it, with no text. On the `splash-done` event it fades out.

### `main` decides when the splash ends
In `use-settings.ts`, the first load with `first_run` calls a new `show_setup` command instead of starting a timer, and sets the view to `setup`. The `splash` view value and the timer go away.

`show_setup` is an async command:
1. It waits until about 800 ms have passed since the splash was created. `lib.rs` stores that `Instant` in a static in `settings.rs`.
2. It emits `events::SPLASH_DONE` to `splash` and waits about 200 ms for the fade.
3. It calls `destroy()` on the splash. `destroy` bypasses the `CloseRequested` handler that blocks closing non-`main` windows.
4. It sizes, centers and shows `main` at the setup size.

Alternative: the splash frontend closes itself. Rejected because it would need a `close` window permission, and the splash does not know when `main` is ready.

### Setup size is decided in one place
`settings.rs` gets `SETUP_WIDTH` (520 logical) and a `show_window(app, size)` helper that `show_settings` and `show_setup` share. The height is the content height: `Setup` measures its root with a `ResizeObserver` and calls `setup_fit(height)`, which stores it in `SETUP_HEIGHT` (600 until the first measure) and resizes `main` (keeping its position). `show_setup` and the first-run `show_settings` use `(SETUP_WIDTH, SETUP_HEIGHT)`, clamped to the work area. `show_window` also takes the window effect: Acrylic for setup (it blurs what is behind the window, like Raycast; Mica only tints with the wallpaper), Mica for settings, so the next settings open after Start goes back to Mica. `Setup` clears the `html` background while mounted and draws `bg-background/45` on its root. There is no `TitleBar` and no `ScrollArea`: the header is the drag region, and the footer has Quit (`api.hide`, same as Escape) on the left and Start on the right.

While `state.path` does not exist (first run, not yet saved), `show_settings` uses the setup size, so the tray Gallery and Settings… items show the compact setup. `show_splash` is removed.

### Setup page: a dark, Raycast-like column
`setup.tsx` is one centred column on near-black glass. It forces the dark tokens by adding `dark` to `<html>` while mounted, restoring the previous state on unmount, so portalled popups match. The root is `bg-neutral-950/80` over Acrylic. In order:
- the logo, `welcome` and `welcomeSub`;
- the modifiers as keycap toggles (`setup/modifier-keys.tsx`, a `ToggleGroup`), picked once;
- a 2x2 grid (`setup/shortcut-grid.tsx`) of fullscreen, window, area and record, each tile showing only the last key, since the modifiers are already shown above. An error or `in_use` turns the tile border red and shows the same text as `hotkey-row.tsx`;
- the "more in Settings" note;
- an inline language `Select` and Start with Windows `Switch`;
- a footer with Quit and Start. Start carries an Enter hint, and Enter starts too, unless focus is on a button or select.

Start is `disabled` when any of the four rows has an `error` or `in_use`. Rows for other slots are ignored, which matches the spec's hidden-shortcut scenario. `rows` is still checked with `textActive` as today, so nothing about `check` changes. The settings rows (`LanguageRow`, `ModifiersRow`, `StartupRow`) are not reused: their `Item` layout is what made the first version look like a settings page.

`HotkeyRows` stops being used by setup. Its `view !== "settings"` branch, which shows the ring switch outside settings, becomes dead and is removed.

### Notification after Start from the backend
A new `finish_setup` command:
1. hides `main` and calls `apply()`, the same as the existing close path;
2. sends a `tauri-plugin-notification` notification built from `State.cfg`, using the en/vi strings next to the ones in `jobs.rs` and the area combo `combos()[2]`.

The frontend `start` calls `api.save` and then `api.finishSetup()` instead of `api.hide()`.

The notification is sent from Rust because that is the only place snapr sends notifications today, and the frontend has no notification plugin permission.

## Risks / Trade-offs

- **[Risk]** Acrylic lags while the window is dragged on some Windows 11 builds. → Setup is shown once and rarely dragged; settings keep Mica.

- **[Risk]** On some GPUs or remote sessions, WebView2 transparency draws a black or white square instead of showing the desktop. → The splash is only up for about a second on first run. The behaviour is the same as the recording panel, which already relies on it.
- **[Risk]** `show_setup` runs while the splash webview has not painted yet, so the logo is visible for less than 800 ms. → The floor counts from window creation, and both webviews load the same bundle at the same time. Accepted.
- **[Trade-off]** Users who want another output folder or no clipboard copy must find those in Settings. → The note under the shortcut list points to Settings, and General is the first settings page.
- **[Risk]** A hidden slot (text, draw or ring) is in use by another app and the user is not told during setup. → `apply()` reports it in the tray tooltip as it does today, and the Hotkeys page shows it.

## Migration Plan

None. Only launches with no `config.toml` are affected, and the config format is unchanged.
