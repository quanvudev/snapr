# Spec Delta

## MODIFIED Requirements

### Requirement: Setup page collects first-day settings
The setup page SHALL show a welcome header, the output folder (with browse and the resolved path), copy to clipboard, start with Windows, language, the hotkey modifiers and the four action hotkeys (fullscreen, window, area and record) with inline validation, and a single Start button.

#### Scenario: Start with defaults
- **WHEN** the user presses Start without changing anything
- **THEN** the default config is written to `config.toml`, hotkeys are applied and the window hides to the tray

#### Scenario: Start with invalid hotkey
- **WHEN** a hotkey has an `invalid`, `needs_modifier` or `duplicate` error
- **THEN** the Start button is disabled and the error is shown on that hotkey row

#### Scenario: Close during setup
- **WHEN** the user presses the close button or Escape on the setup page
- **THEN** the window hides to the tray, the app keeps running with defaults, and no config file is written

## ADDED Requirements

### Requirement: Older configs get the default record hotkey
When `config.toml` has no record hotkey, the app SHALL use the default record hotkey and keep every other setting from the file.

#### Scenario: Config from before recording
- **WHEN** the app launches with a `config.toml` that has fullscreen, window and area hotkeys but no record hotkey
- **THEN** the launch is not a first run, the existing settings are kept and the record hotkey is the default `4`
