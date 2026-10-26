# Spec Delta

## MODIFIED Requirements

### Requirement: Settings are grouped in a sidebar
The settings window SHALL show a left sidebar listing the groups Gallery, General, Capture, Preview, Recording, Hotkeys and About, and a right pane showing the selected group. General SHALL be selected when the window is opened from Settings…, and Gallery when it is opened from the tray Gallery item.

#### Scenario: Gallery group
- **WHEN** Gallery is selected
- **THEN** the pane shows the capture gallery described by the `gallery` capability

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

#### Scenario: Open gallery from tray
- **WHEN** the user chooses Gallery in the tray menu
- **THEN** the window reloads the current config and shows the Gallery group

## ADDED Requirements

### Requirement: Tray menu has a Gallery item
The tray menu SHALL list, in order, Gallery, Settings…, Open folder, Stop recording and Quit. Gallery SHALL open the window on the Gallery group. On first run, before setup is completed, both Gallery and Settings… SHALL show the setup page.

#### Scenario: Gallery item
- **WHEN** the user right-clicks the tray icon
- **THEN** Gallery is the first menu item, above Settings…

#### Scenario: Gallery on first run
- **WHEN** setup has not been completed and the user chooses Gallery in the tray menu
- **THEN** the window shows the setup page

### Requirement: No browser context menu or reload in release builds
In release builds the window SHALL NOT show the WebView default context menu, and F5, Ctrl+R and Ctrl+P SHALL do nothing. Context menus provided by snapr itself SHALL still open. Development builds keep the default menu and shortcuts.

#### Scenario: Right-click on empty area
- **WHEN** the user right-clicks a part of the window that has no snapr menu, in a release build
- **THEN** no menu appears

#### Scenario: Reload shortcut
- **WHEN** the user presses F5 or Ctrl+R in a release build
- **THEN** the window does not reload and the current state is kept

#### Scenario: Gallery menu still works
- **WHEN** the user right-clicks a gallery item in a release build
- **THEN** the gallery item menu opens
