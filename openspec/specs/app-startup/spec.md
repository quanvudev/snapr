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
On first run the app SHALL show a frameless splash window of about 480x300, centered, showing the app icon, name and version, for about one second, then switch the same window to the setup page at the settings window size.

#### Scenario: Splash then setup
- **WHEN** the app launches on first run
- **THEN** the splash is shown centered for about one second
- **AND** the window then grows to the settings size, re-centers and shows the setup page

### Requirement: Setup page collects first-day settings
The setup page SHALL show a welcome header, the output folder (with browse and the resolved path), copy to clipboard, start with Windows, language, the hotkey modifiers and the six action hotkeys (fullscreen, window, area, text, record and draw) with inline validation, and a single Start button.

#### Scenario: Start with defaults
- **WHEN** the user presses Start without changing anything
- **THEN** the default config is written to `config.toml` with hotkeys fullscreen `1`, window `2`, area `3`, text `4`, record `5` and draw `6`, hotkeys are applied and the window hides to the tray

#### Scenario: Start with invalid hotkey
- **WHEN** a hotkey has an `invalid`, `needs_modifier` or `duplicate` error
- **THEN** the Start button is disabled and the error is shown on that hotkey row

#### Scenario: Close during setup
- **WHEN** the user presses the close button or Escape on the setup page
- **THEN** the window hides to the tray, the app keeps running with defaults, and no config file is written

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
