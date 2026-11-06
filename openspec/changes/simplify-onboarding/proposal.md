# Proposal

## Why

A first run currently shows a boxed Mica splash on a fixed one-second timer, then a full settings-sized window that asks for about fourteen things at once: output folder, clipboard, startup, language, the hotkey mode, the modifiers and seven hotkey rows. A new user only needs to know which keys capture and where the app went. Most of these choices already have good defaults and can be changed later in Settings.

## What Changes

- The first-run splash becomes a separate frameless, transparent, shadowless window. It shows only the app logo and a spinner floating over the desktop, in the style of Raycast. It stays up until the settings window has loaded (at least about 800 ms), then fades out. The fixed one-second timer goes away.
- The setup page becomes a compact single-column window, 520 px wide and as tall as its content, with no title bar, close button or scrolling, instead of the full settings size. It has:
  - a welcome header with the language selector at the top;
  - the hotkey modifiers (editable);
  - a read-only list of the four main shortcuts (fullscreen, window, area, record) resolved with the current modifiers;
  - a hint that more shortcuts are in Settings;
  - Start with Windows;
  - a footer with Quit on the left and Start on the right.
- Removed from setup, keeping their defaults and staying editable in Settings: output folder, copy to clipboard, hotkey mode (fixed / manual), per-slot key editing and the text, draw and ring hotkeys (including the ring switch).
- Start is disabled only when one of the four shown shortcuts has an error or is in use by another app.
- After Start, the window hides to the tray and a Windows notification (en/vi) says snapr is running in the tray and names the area shortcut.
- Opening Gallery or Settings… from the tray before setup is finished shows the compact setup window, not a settings-sized one.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `app-startup`: the splash requirement, the setup page contents and Start validation change, and a notification is added after Start.
- `settings-window`: the dark blurred appearance requirement no longer covers the splash, which is now transparent with no backdrop.

## Impact

- Backend:
  - `lib.rs` creates the `splash` window on first run instead of calling `show_splash`;
  - `settings.rs` gets a setup-sized show path, used by a new `show_setup` command and by `show_settings` while there is no config file, plus a `finish_setup` command that sends the welcome notification and hides `main`;
  - `events.rs` gets a new `splash-done` event;
  - `capabilities/default.json` lists the `splash` window.
- Frontend:
  - `main.tsx` routes `?view=splash`;
  - `components/splash/` becomes the transparent splash app;
  - `components/setup/setup.tsx` is rewritten using `LanguageRow`, `ModifiersRow` and `StartupRow`;
  - `use-settings.ts` drops the `splash` view and the timer;
  - `lib/api.ts` gets the new commands;
  - `lib/i18n.ts` gets the new en/vi strings.
- No config format change. Existing installs are unaffected; the change only shows when `config.toml` is missing.
