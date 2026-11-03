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
The settings window SHALL show a left sidebar listing the groups Gallery, General, Capture, Preview, Recording, Hotkeys and About, and a right pane showing the selected group. General SHALL be selected when the window is opened from Settings…, and Gallery when it is opened from the tray Gallery item.

#### Scenario: Gallery group
- **WHEN** Gallery is selected
- **THEN** the pane shows the capture gallery described by the `gallery` capability

#### Scenario: General group
- **WHEN** General is selected
- **THEN** the pane shows the output folder (input, browse, open, resolved path), copy to clipboard, start with Windows and language

#### Scenario: Capture group
- **WHEN** Capture is selected
- **THEN** the pane shows a padding on/off switch, the padding size in px (1 to 512), a rounded corners on/off switch with a radius in px (1 to 128), the background image path with a browse action (PNG or JPEG) and a clear action, a text capture on/off switch and the OCR engine choice (Windows or PaddleOCR v6); with no image the padding is transparent

#### Scenario: Text capture unavailable
- **WHEN** Capture is selected, Windows has no OCR language installed and no PaddleOCR model is ready for the selected engine
- **THEN** the text capture switch is disabled, a "No OCR language installed" hint is shown, and an action opens the Windows language settings

#### Scenario: Text capture available through PaddleOCR
- **WHEN** Capture is selected, Windows has no OCR language installed and the engine is PaddleOCR with a ready model
- **THEN** the text capture switch is enabled and no "No OCR language installed" hint is shown

#### Scenario: PaddleOCR options
- **WHEN** Capture is selected and the engine is PaddleOCR
- **THEN** the pane also shows a Tiny / Small / Medium size choice set to the current size, a one-line description and download size of the selected size, a note that PaddleOCR does not read Vietnamese, and the list of sizes each marked in use, downloaded or not downloaded, with a Delete action on each downloaded size

#### Scenario: PaddleOCR options hidden
- **WHEN** Capture is selected and the engine is Windows
- **THEN** no size choice, model list or download status is shown

#### Scenario: Download progress
- **WHEN** a model size is downloading
- **THEN** the pane shows the size being downloaded, a progress bar with the percentage and a Cancel action

#### Scenario: Download failed
- **WHEN** a model download fails
- **THEN** the pane shows the error and a Retry action

#### Scenario: Selected size missing
- **WHEN** the engine is PaddleOCR, the selected size is not downloaded and no download is running
- **THEN** the pane shows that the size is not downloaded and a Download action

#### Scenario: Confirm deletion
- **WHEN** the user presses Delete on a downloaded size
- **THEN** a confirmation dialog names the size and its disk size, and the files are removed only after confirming

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

### Requirement: Tray menu has a Gallery item
The tray menu SHALL list, in order, Gallery, Settings…, Open folder, Stop recording, Pause recording and Quit. When an update is available, an "Update to <version>" item SHALL appear directly above Quit, and SHALL be absent otherwise. Gallery SHALL open the window on the Gallery group. On first run, before setup is completed, both Gallery and Settings… SHALL show the setup page.

#### Scenario: Gallery item
- **WHEN** the user right-clicks the tray icon
- **THEN** Gallery is the first menu item, above Settings…

#### Scenario: Gallery on first run
- **WHEN** setup has not been completed and the user chooses Gallery in the tray menu
- **THEN** the window shows the setup page

#### Scenario: Update item
- **WHEN** snapr has found version 1.6.0 and the user right-clicks the tray icon
- **THEN** "Update to 1.6.0" is shown between Stop recording and Quit

#### Scenario: No update item
- **WHEN** no newer version has been found
- **THEN** the tray menu shows only Gallery, Settings…, Open folder, Stop recording and Quit

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

### Requirement: General has an automatic update check switch
The General group SHALL show a "Check for updates automatically" switch set to the current value. Changing it SHALL save immediately like every other setting.

#### Scenario: Turn off
- **WHEN** the user switches automatic update checks off
- **THEN** `config.toml` is updated and snapr stops checking the feed on its own

#### Scenario: Turn on
- **WHEN** the user switches automatic update checks on
- **THEN** `config.toml` is updated and a check runs immediately

### Requirement: About shows the update status
The About group SHALL show an update row under the version. The row SHALL show one state at a time: idle with a Check now action; checking; up to date; a newer version with an Update action; downloading with a progress bar and percentage; installing; or an error with a Retry action. Check now SHALL work whether automatic checks are on or off.

#### Scenario: Check now, up to date
- **WHEN** the user presses Check now and no newer version exists
- **THEN** the row shows a spinner while checking, then "Up to date"

#### Scenario: Check now, update found
- **WHEN** the user presses Check now and 1.6.0 is available
- **THEN** the row shows "Version 1.6.0 available" and an Update action

#### Scenario: Check failed
- **WHEN** the user presses Check now and the feed cannot be reached
- **THEN** the row shows the error and a Retry action

#### Scenario: Already announced
- **WHEN** an automatic check found 1.6.0 earlier and the user opens About
- **THEN** the row already shows "Version 1.6.0 available" with Update, without checking again

#### Scenario: Downloading
- **WHEN** the user presses Update
- **THEN** the row shows a progress bar with the percentage until the download finishes, then an installing state before snapr closes

#### Scenario: Update blocked by a recording
- **WHEN** the user presses Update while a recording or countdown is running
- **THEN** the row shows that the recording must be stopped first and keeps the Update action

### Requirement: Ring section in the sidebar
The sidebar SHALL list a Ring section after Gallery and Settings. Selecting it SHALL show the ring configuration described by the `ring-config` capability in its own pane, separate from the Settings page. Opening the window from the tray or the ring SHALL NOT select Ring. Ring SHALL be hidden while the action ring is turned off.

#### Scenario: Open the Ring section
- **WHEN** the user selects Ring in the sidebar
- **THEN** the pane shows the ring entry list and the Settings page content is not shown

#### Scenario: Ring settings stay out of Settings
- **WHEN** the user scrolls the Settings section
- **THEN** it contains no ring entry list

#### Scenario: Ring turned off
- **WHEN** the action ring switch is off
- **THEN** the sidebar lists only Gallery and Settings

### Requirement: Recording shows transcript settings
The Recording group SHALL show an "Automatically transcribe after recording" switch, a Whisper model choice Base / Small / Large v3 Turbo with each model's disk size and download state, and a language choice Auto-detect / Vietnamese / English, all set to their current values. Changes SHALL save immediately.

#### Scenario: Defaults
- **WHEN** the Recording group is shown for a config without transcript settings
- **THEN** the switch is off, Small is selected and Auto-detect is selected

#### Scenario: Change language
- **WHEN** the user selects Vietnamese
- **THEN** the config is saved with Vietnamese and the next transcription uses it

### Requirement: Selecting a model downloads it
When the user selects a model that is not downloaded, or turns on automatic transcription while the selected model is not downloaded, the download SHALL start. The model row SHALL show its progress with Cancel, an inline error with Retry on failure, and Delete with a confirmation for each downloaded model.

#### Scenario: Turn on automatic transcription
- **WHEN** Small is selected but not downloaded and the user turns automatic transcription on
- **THEN** the config is saved and the Small download starts with its progress shown

#### Scenario: Delete a model
- **WHEN** the user deletes the downloaded Base model and confirms
- **THEN** Base shows as not downloaded and its disk space is freed

### Requirement: Recording shows a quality choice
The Recording group SHALL show a recording quality choice Low / Medium / High set to the current value, next to the frame rate, with a one-line hint that lower quality gives smaller files. Changing it SHALL save immediately like every other setting.

#### Scenario: Quality row
- **WHEN** the Recording group is shown
- **THEN** it shows the quality choice with the current value selected

#### Scenario: Change quality
- **WHEN** the user selects Low
- **THEN** the config is saved with Low and the next recording uses it

### Requirement: Recording shows automatic compression
The Recording group SHALL show an "Automatically compress after recording" switch set to the current value, with a hint that the original is replaced. While it is on, a 1080p / 720p choice set to the current target SHALL be shown under it. Changes SHALL save immediately.

#### Scenario: Switch off
- **WHEN** the Recording group is shown and automatic compression is off
- **THEN** the switch is off and no resolution choice is shown

#### Scenario: Turn it on
- **WHEN** the user turns the switch on
- **THEN** the config is saved with automatic compression on and the 1080p / 720p choice appears with 1080p selected
