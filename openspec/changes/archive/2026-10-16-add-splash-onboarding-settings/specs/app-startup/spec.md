# Spec Delta

## Purpose

Defines what snapr shows when it launches: a splash and guided setup for a first-time user, and a silent start to the tray once the user has completed setup.

## ADDED Requirements

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
The setup page SHALL show a welcome header, the output folder (with browse and the resolved path), copy to clipboard, start with Windows, language, the hotkey modifiers and the three action hotkeys with inline validation, and a single Start button.

#### Scenario: Start with defaults
- **WHEN** the user presses Start without changing anything
- **THEN** the default config is written to `config.toml`, hotkeys are applied and the window hides to the tray

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
