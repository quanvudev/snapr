# settings-window Specification

## Purpose

Defines the settings window opened from the tray: its frameless dark chrome, fixed size, sidebar grouping of settings, and how changes are saved.

## Requirements

### Requirement: Frameless window with a single close button
The settings window SHALL have no OS title bar. Its top strip SHALL be draggable to move the window, and a single close button SHALL sit at the top right. There SHALL be no minimize or maximize control.

#### Scenario: Move window
- **WHEN** the user drags the top strip of the sidebar or the content pane
- **THEN** the window moves

#### Scenario: Close hides to tray
- **WHEN** the user presses the close button or Escape
- **THEN** the window hides and the app keeps running in the tray

### Requirement: Fixed window size capped to the screen
The settings window SHALL open at 1366x768 logical pixels, centered, and SHALL NOT be resizable. If the monitor work area is smaller, each dimension SHALL be reduced to fit the work area.

#### Scenario: Large screen
- **WHEN** the settings window opens on a monitor whose work area is at least 1366x768 logical pixels
- **THEN** the window is 1366x768 and centered

#### Scenario: Small or scaled screen
- **WHEN** the settings window opens on a monitor whose work area is 1280x672 logical pixels
- **THEN** the window is 1280x672 and centered, with content scrolling inside the right pane

#### Scenario: No resize
- **WHEN** the user drags a window edge
- **THEN** the window size does not change

### Requirement: Dark blurred appearance
The settings and splash windows SHALL use a dark theme over the native Windows blurred backdrop, independent of the system light/dark setting.

#### Scenario: Light system theme
- **WHEN** Windows is set to light mode
- **THEN** the window still renders the dark theme over the blurred backdrop

### Requirement: Settings are grouped in a sidebar
The settings window SHALL show a left sidebar listing the groups General, Capture, Preview, Recording, Hotkeys and About, and a right pane showing the selected group as grouped cards. General SHALL be selected when the window opens.

#### Scenario: General group
- **WHEN** General is selected
- **THEN** the pane shows the output folder (input, browse, open, resolved path), copy to clipboard, start with Windows and language

#### Scenario: Capture group
- **WHEN** Capture is selected
- **THEN** the pane shows a padding on/off switch, the padding size in px (1 to 512), a rounded corners on/off switch with a radius in px (1 to 128), the background image path with a browse action (PNG or JPEG) and a clear action, and a text capture on/off switch; with no image the padding is transparent

#### Scenario: Text capture unavailable
- **WHEN** Capture is selected and Windows has no OCR language installed
- **THEN** the text capture switch is disabled, a "No OCR language installed" hint is shown, and an action opens the Windows language settings

#### Scenario: Preview group
- **WHEN** Preview is selected
- **THEN** the pane shows preview seconds (0 to 30, 0 = off)

#### Scenario: Recording group
- **WHEN** Recording is selected
- **THEN** the pane shows the recording frame rate with a choice of 30 or 60 fps, set to the current value

#### Scenario: Hotkeys group
- **WHEN** Hotkeys is selected
- **THEN** the pane shows the modifier toggles, the five action hotkeys (fullscreen, window, area, text and record) with their resolved combo and status, and a reset-to-defaults action

#### Scenario: Inactive text hotkey row
- **WHEN** Hotkeys is selected and text capture is off or unavailable
- **THEN** the text hotkey row is disabled and shows no error

#### Scenario: About group
- **WHEN** About is selected
- **THEN** the pane shows the app icon, name and version, and actions to open the config file and open the screenshot folder

#### Scenario: Reopen from tray
- **WHEN** the user opens Settings from the tray
- **THEN** the window reloads the current config and shows the General group

### Requirement: Changes save immediately
Each settings change SHALL be saved and applied without a Save button. Text inputs SHALL save when they lose focus or on Enter. A change SHALL NOT be saved while any hotkey has an error; the error stays visible until fixed.

#### Scenario: Toggle saves
- **WHEN** the user switches Copy to clipboard off
- **THEN** `config.toml` is updated and the next capture is not copied to the clipboard

#### Scenario: Hotkey saves and applies
- **WHEN** the user records a new valid hotkey for Area
- **THEN** the config is saved and the new hotkey triggers area capture immediately

#### Scenario: Folder typed
- **WHEN** the user types a folder path
- **THEN** nothing is saved until the input loses focus or Enter is pressed

#### Scenario: Invalid hotkey blocks save
- **WHEN** a hotkey has a `duplicate` error
- **THEN** no change is saved and the row shows the error until it is resolved

#### Scenario: Save fails
- **WHEN** writing the config fails
- **THEN** an inline error is shown in the window
