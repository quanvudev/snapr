# app-startup Specification

## Purpose

Defines what snapr shows when it launches: a splash and guided setup for a first-time user, and a silent start to the tray once the user has completed setup.

## Requirements

### Requirement: First run is detected by the absence of the config file
The app SHALL treat a launch as a first run when the config file does not exist at startup. Launching SHALL NOT create the config file; only completing setup or saving settings creates it.

#### Scenario: No config file
- **WHEN** the app launches and `config.toml` does not exist in the app config directory
- **THEN** the launch is a first run and the app runs with default settings and hotkeys

#### Scenario: Launch does not create the config
- **WHEN** the app launches on first run and the user quits or closes the window without pressing Start
- **THEN** `config.toml` still does not exist and the next launch is again a first run

#### Scenario: Config file exists
- **WHEN** the app launches and `config.toml` exists
- **THEN** the launch is not a first run

### Requirement: First run shows a splash then the setup page
On first run the app SHALL show a transparent splash: a small frameless window centered on screen, with no backdrop, border or shadow, showing only the app logo and a spinner. It SHALL stay visible until the setup page has loaded, and for at least about 800 ms. It SHALL then fade out and close while the setup page appears.

#### Scenario: Splash then setup
- **WHEN** the app launches on first run
- **THEN** the logo and a spinner appear centered over the desktop with nothing drawn around them
- **AND** once the setup page has loaded, and no earlier than about 800 ms after the splash appeared, the splash fades out and the setup page is shown centered

#### Scenario: Slow load
- **WHEN** the setup page takes longer than 800 ms to load
- **THEN** the splash stays visible with its spinner until the setup page is ready

#### Scenario: Splash is not in the taskbar
- **WHEN** the splash is visible
- **THEN** it has no taskbar entry and cannot be resized

### Requirement: Setup page collects first-day settings
The setup page SHALL be a compact single-column window, 520 logical px wide and exactly as tall as its content (no scrolling, no title bar or close button; the header drags the window). It SHALL show:
- a welcome header;
- the language selector;
- the hotkey modifiers;
- a read-only list of the fullscreen, window, area and record shortcuts, resolved with the current modifiers;
- a note that more shortcuts are in Settings;
- Start with Windows;
- a footer with Quit on the left and Start on the right.

It SHALL NOT show the output folder, copy to clipboard, hotkey mode, per-shortcut editing, the text, draw or ring shortcuts, or the ring switch; those keep their defaults.

#### Scenario: Start with defaults
- **WHEN** the user presses Start without changing anything
- **THEN** the default config is written to `config.toml` with hotkeys fullscreen `1`, window `2`, area `3`, text `4`, record `5` and draw `6`, hotkeys are applied and the window hides to the tray

#### Scenario: Change the modifiers
- **WHEN** the user changes the modifiers from `Ctrl+Shift` to `Ctrl+Alt`
- **THEN** the shortcut list immediately reads `Ctrl+Alt+1`, `Ctrl+Alt+2`, `Ctrl+Alt+3` and `Ctrl+Alt+5`
- **AND** pressing Start saves `Ctrl+Alt` as the modifiers

#### Scenario: Change the language
- **WHEN** the user picks a different language
- **THEN** the setup page text switches to that language at once

#### Scenario: Start with invalid hotkey
- **WHEN** one of the four shown shortcuts has an `invalid`, `needs_modifier` or `duplicate` error, or is in use by another app
- **THEN** that shortcut row shows the problem and Start is disabled

#### Scenario: Hidden shortcut problem does not block Start
- **WHEN** only the text, draw or ring shortcut is in use by another app
- **THEN** Start stays enabled and that shortcut is reported after Start as it would be from Settings

#### Scenario: Close during setup
- **WHEN** the user presses Quit or Escape on the setup page
- **THEN** the window hides to the tray, the app keeps running with defaults, and no config file is written

#### Scenario: Tray opens setup before it is finished
- **WHEN** setup has not been completed and the user chooses Gallery or Settings… in the tray menu
- **THEN** the compact setup page is shown, at the setup size

### Requirement: Configured launch goes straight to the tray
When the launch is not a first run, the app SHALL show no window at startup and only register the tray icon and hotkeys.

#### Scenario: Normal launch
- **WHEN** the app launches with an existing `config.toml`, manually or via Start with Windows
- **THEN** no splash or window is shown and hotkeys are active

### Requirement: Older configs get the default record hotkey
When `config.toml` has no record hotkey, the app SHALL use the default record hotkey and keep every other setting from the file.

#### Scenario: Config from before recording
- **WHEN** the app launches with a `config.toml` that has fullscreen, window and area hotkeys but no record hotkey
- **THEN** the launch is not a first run, the existing settings are kept and the record hotkey is the default `5`

### Requirement: Older configs leave the text hotkey unbound
When `config.toml` has no text hotkey, the app SHALL leave the text hotkey empty, keep every other setting from the file, and SHALL NOT report an error for the empty text hotkey. The text hotkey stays unbound until the user records one.

#### Scenario: Config from before text capture
- **WHEN** the app launches with a `config.toml` that has record `4` and no text hotkey
- **THEN** the record hotkey stays `4`, the text hotkey is empty and not registered, and no duplicate or invalid error is shown

#### Scenario: Bind the text hotkey
- **WHEN** the user records a valid key for the empty text hotkey in settings
- **THEN** the config is saved and the text hotkey works immediately

### Requirement: Older configs leave the draw hotkey unbound
When `config.toml` has no draw hotkey, the app SHALL leave the draw hotkey empty, keep every other setting from the file, and SHALL NOT report an error for the empty draw hotkey. The draw hotkey stays unbound until the user records one.

#### Scenario: Config from before draw capture
- **WHEN** the app launches with a `config.toml` that has area `7` and no draw hotkey
- **THEN** the area hotkey stays `7`, the draw hotkey is empty and not registered, and no duplicate or invalid error is shown

#### Scenario: Bind the draw hotkey
- **WHEN** the user records a valid key for the empty draw hotkey in settings
- **THEN** the config is saved and the draw hotkey works immediately

### Requirement: Finishing setup confirms where the app went
After Start saves the config and hides the setup page, the app SHALL show a Windows notification in the chosen language. It SHALL say snapr is running in the system tray and name the area capture shortcut.

#### Scenario: Notification after Start
- **WHEN** the user presses Start with the language set to Vietnamese and the default modifiers `Ctrl+Shift`
- **THEN** the setup page hides and a notification says snapr is running in the system tray and to press `Ctrl+Shift+3` to capture an area

#### Scenario: No notification when setup is closed
- **WHEN** the user presses Quit or Escape on the setup page
- **THEN** no notification is shown
