# Spec Delta

## MODIFIED Requirements

### Requirement: Setup page collects first-day settings
The setup page SHALL show a welcome header, the output folder (with browse and the resolved path), copy to clipboard, start with Windows, language, the hotkey modifiers and the five action hotkeys (fullscreen, window, area, text and record) with inline validation, and a single Start button.

#### Scenario: Start with defaults
- **WHEN** the user presses Start without changing anything
- **THEN** the default config is written to `config.toml` with hotkeys fullscreen `1`, window `2`, area `3`, text `4` and record `5`, hotkeys are applied and the window hides to the tray

#### Scenario: Start with invalid hotkey
- **WHEN** a hotkey has an `invalid`, `needs_modifier` or `duplicate` error
- **THEN** the Start button is disabled and the error is shown on that hotkey row

#### Scenario: Close during setup
- **WHEN** the user presses the close button or Escape on the setup page
- **THEN** the window hides to the tray, the app keeps running with defaults, and no config file is written

### Requirement: Older configs get the default record hotkey
When `config.toml` has no record hotkey, the app SHALL use the default record hotkey and keep every other setting from the file.

#### Scenario: Config from before recording
- **WHEN** the app launches with a `config.toml` that has fullscreen, window and area hotkeys but no record hotkey
- **THEN** the launch is not a first run, the existing settings are kept and the record hotkey is the default `5`

## ADDED Requirements

### Requirement: Older configs leave the text hotkey unbound
When `config.toml` has no text hotkey, the app SHALL leave the text hotkey empty, keep every other setting from the file, and SHALL NOT report an error for the empty text hotkey. The text hotkey stays unbound until the user records one.

#### Scenario: Config from before text capture
- **WHEN** the app launches with a `config.toml` that has record `4` and no text hotkey
- **THEN** the record hotkey stays `4`, the text hotkey is empty and not registered, and no duplicate or invalid error is shown

#### Scenario: Bind the text hotkey
- **WHEN** the user records a valid key for the empty text hotkey in settings
- **THEN** the config is saved and the text hotkey works immediately
