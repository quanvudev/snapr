# Spec Delta

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Tray menu has a Gallery item
The tray menu SHALL list, in order, Gallery, Settings…, Open folder, Stop recording and Quit. When an update is available, an "Update to <version>" item SHALL appear directly above Quit, and SHALL be absent otherwise. Gallery SHALL open the window on the Gallery group. On first run, before setup is completed, both Gallery and Settings… SHALL show the setup page.

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
