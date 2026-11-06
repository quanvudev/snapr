# Tasks

## 1. Transparent splash window

- [x] 1.1 Add `SPLASH_DONE` (`"splash-done"`) to `events.rs`, and `"splash"` to the window list in `capabilities/default.json`. Verify `cargo clippy` is clean.
- [x] 1.2 In `settings.rs`, add `open_splash(app)`:
  - builds the `splash` `WebviewWindowBuilder` (`index.html?view=splash`, 240x240 logical, centered, frameless, transparent, no shadow, `skip_taskbar`, not resizable, visible);
  - records the creation `Instant` in a static.

  In `lib.rs`, call it on first run in place of `show_splash`, and remove `show_splash`. Verify that with `config.toml` renamed, launching shows a second window and that `main` stays hidden.
- [x] 1.3 Add `components/splash/splash-app.tsx` and export it from the splash barrel:
  - clear the `html` background like `PanelApp`;
  - the 96 px `128x128@2x.png` logo with a zoom/fade-in and a `Spinner` under it, no text;
  - listen for `splash-done` and fade out.

  Route `?view=splash` to it in `main.tsx`, and delete the old `splash.tsx`. Verify `bunx tsc --noEmit` passes and that, on a first run, only the logo and spinner float over the desktop with no box, border or shadow.

## 2. Setup window sizing and handoff

- [x] 2.1 In `settings.rs`, add `SETUP_WIDTH` (520 logical), `SETUP_HEIGHT` (content height from `setup_fit`) and a shared `show_window(app, size)` helper that clamps to the work area, centers, shows and focuses. In `show_settings`, use the setup size while `state.path` does not exist. Verify that on a first run, tray Settings… and Gallery open the compact size.
- [x] 2.2 Add the async `show_setup` command:
  1. wait until 800 ms after the splash was created;
  2. emit `SPLASH_DONE` to `splash`;
  3. wait 200 ms, then `destroy()` the splash;
  4. `show_window` at the setup size.

  Register it in `invoke_handler` and add `api.showSetup` to `lib/api.ts`. Verify the splash fades out and the setup window appears centered.
- [x] 2.3 In `use-settings.ts`:
  - remove the `splash` view, `timer` and `setTimeout`;
  - on the first load with `first_run`, set the view to `setup` and call `api.showSetup()`;
  - drop the `version` prop path that only the splash used, if nothing else reads it.

  Update `App.tsx` so it no longer renders `Splash`. Verify `bunx tsc --noEmit` passes, and that a normal launch (config present) shows no window.

## 3. Compact setup page

- [x] 3.1 Add `components/setup/shortcut-grid.tsx` and `modifier-keys.tsx`. For the fullscreen, window, area and record slots (indices from `SLOTS`), render one `Item` per slot from `rows` in the context: the slot label and the combo in `Kbd`. An error or `in_use` shows the same `Badge` text as `hotkey-row.tsx`. Verify that changing the modifiers updates the combos at once.
- [x] 3.2 Rewrite `setup.tsx` as one column:
  - `TitleBar` and a header with the logo, `welcome` and `welcomeSub`;
  - a `SettingsCard` with `LanguageRow` and `ModifiersRow`;
  - `ShortcutList` in a card, then the "more in Settings" note;
  - a card with `StartupRow`;
  - a footer with Start, disabled when any of the four shown rows has `error` or `in_use`.

  Verify that the window fits without scrolling at 520x600, and that output folder, clipboard, mode and ring do not appear.
- [x] 3.3 Remove the `view !== "settings"` `RingSwitchRow` branch from `HotkeyRows`, and remove the `ClipboardRow` / `FolderRow` / `HotkeyRows` barrel exports that only setup used. Verify `bunx tsc --noEmit` passes and the Hotkeys and General settings pages look unchanged.
- [x] 3.4 Add the en/vi i18n strings to `lib/i18n.ts`:
  - a shorter `welcomeSub`;
  - `moreShortcuts` ("More shortcuts in Settings" / "Thêm phím tắt trong Cài đặt").

  Verify both languages render on the setup page when the language is switched.

## 4. Finish setup notification

- [x] 4.1 Add the `finish_setup` command in `settings.rs`:
  - hide `main` and call `apply()`;
  - send a `tauri-plugin-notification` notification from `State.cfg`. In en: "Snapr is running in the system tray. Press {area} to capture an area." In vi: "Snapr đang chạy ở khay hệ thống. Nhấn {area} để chụp vùng." `{area}` is `combos()[2]`.

  Register the command, add `api.finishSetup`, and make `start` in `use-settings.ts` call it instead of `api.hide()`. Verify that pressing Start writes `config.toml`, hides the window and shows the notification in the chosen language with the current modifiers.
- [x] 4.2 Verify that closing setup with X or Escape shows no notification and writes no `config.toml`.

## 5. Docs and integration

- [x] 5.1 Update `CLAUDE.md`:
  - the splash window (`splash` label, `?view=splash`, `show_setup`, `SPLASH_DONE`);
  - the setup size and contents;
  - `finish_setup`;
  - the `main` window description of "three views", which becomes settings and setup only.

  Verify no remaining mention of `show_splash` or the 480x300 splash view.
- [x] 5.2 Run `cargo clippy` and `bunx tsc --noEmit`, then do a full first-run pass with `bun run tauri dev` and `config.toml` removed: splash, setup, change the modifiers and language, Start, notification, hotkeys work. Then do a second launch, which goes straight to the tray.

## Workflow follow-up

- Archive the change after review with `/opsx:archive`.
