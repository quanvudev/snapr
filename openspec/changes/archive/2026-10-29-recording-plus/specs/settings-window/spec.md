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
- **THEN** the pane shows the recording frame rate with a choice of 30 or 60 fps, set to the current value, and a countdown on/off switch, set to the current value

#### Scenario: Hotkeys group
- **WHEN** Hotkeys is selected
- **THEN** the pane shows the Fixed / Manual mode switch with a one-line description of the selected mode, the hotkey rows in two headed groups, Capture (fullscreen, window, area, text, record, draw) and Tools (ring), and a Restore defaults action

#### Scenario: Combo shown as key chips
- **WHEN** a hotkey row has a combo
- **THEN** the row shows one key chip per key, for example Ctrl, Shift and 3, and does not repeat the combo as text or show a success mark

#### Scenario: Empty hotkey row
- **WHEN** a hotkey row has no combo
- **THEN** the row shows a muted "Not set"

#### Scenario: Problem badge
- **WHEN** a hotkey has an invalid, needs-modifier, duplicate or in-use problem
- **THEN** a badge naming the problem is shown under the row title, and a healthy row shows no status

#### Scenario: Clear an optional hotkey
- **WHEN** the user presses the clear button on the text, draw or ring row
- **THEN** that hotkey is unbound and the row shows "Not set"

#### Scenario: Required rows have no clear button
- **WHEN** Hotkeys is selected
- **THEN** the fullscreen, window, area and record rows show no clear button and can only be re-recorded

#### Scenario: Recording state
- **WHEN** the user clicks a row's key chips to record
- **THEN** that row is highlighted and reads "Press keys… Esc to cancel", the other rows are dimmed, and Escape or recording a combo returns every row to normal

#### Scenario: Fixed mode rows
- **WHEN** Hotkeys is selected and the mode is Fixed
- **THEN** the pane also shows the modifier toggles, and each row's chips record a key for that slot

#### Scenario: Manual mode rows
- **WHEN** Hotkeys is selected and the mode is Manual
- **THEN** the modifier toggles are hidden and each row's chips record a full combo for that slot

#### Scenario: Restore defaults for the active mode
- **WHEN** the user presses Restore defaults in Fixed mode
- **THEN** the modifier prefix and keys return to their defaults, the Manual combos and every other setting are unchanged, and the mode stays Fixed

#### Scenario: Restore defaults in Manual
- **WHEN** the user presses Restore defaults in Manual mode
- **THEN** the Manual combos return to the default combos, the Fixed prefix and keys and every other setting are unchanged, and the mode stays Manual

#### Scenario: Inactive text hotkey row
- **WHEN** Hotkeys is selected and text capture is off or unavailable
- **THEN** the text hotkey row is disabled and shows no problem badge

#### Scenario: About group
- **WHEN** About is selected
- **THEN** the pane shows the app icon, name and version, and actions to open the config file and open the screenshot folder

#### Scenario: Reopen from tray
- **WHEN** the user opens Settings from the tray
- **THEN** the window reloads the current config and shows the General group

#### Scenario: Open gallery from tray
- **WHEN** the user chooses Gallery in the tray menu
- **THEN** the window reloads the current config and shows the Gallery group

