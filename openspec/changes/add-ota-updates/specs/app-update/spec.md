# Spec Delta

## Purpose

Keeps installed copies of snapr up to date from published builds: checking a public release feed, announcing a newer version, and downloading, verifying and installing it at the user's request without losing work in progress.

## ADDED Requirements

### Requirement: Updates come from a public release feed
The app SHALL read update information only from the `latest.json` of the latest release of the public `quanvudev/snapr-releases` repository. A version is newer only when its semantic version is greater than the running version; equal or older versions SHALL be ignored.

#### Scenario: Newer version published
- **WHEN** the feed lists version 1.6.0 and snapr 1.5.0 is running
- **THEN** 1.6.0 is reported as available

#### Scenario: Same or older version
- **WHEN** the feed lists 1.5.0 or 1.4.0 and snapr 1.5.0 is running
- **THEN** no update is reported

### Requirement: Automatic check at launch and every 6 hours
While "Check for updates automatically" is on, the app SHALL check the feed shortly after launch and then every 6 hours while it runs. While it is off, the app SHALL NOT contact the feed unless the user asks to check. A failed automatic check SHALL show nothing to the user and SHALL be retried at the next scheduled check.

#### Scenario: Launch with checks on
- **WHEN** snapr launches, including via Start with Windows, with automatic checks on
- **THEN** the feed is checked without showing any window

#### Scenario: Checks off
- **WHEN** automatic checks are off
- **THEN** no request is made to the feed at launch or later, until the user presses Check now

#### Scenario: Offline
- **WHEN** an automatic check fails because there is no network
- **THEN** no notification, error or window is shown and the next check runs 6 hours later

#### Scenario: Turned on while running
- **WHEN** the user turns automatic checks on in settings
- **THEN** a check runs without waiting for the next launch

### Requirement: Automatic checks default to on
Automatic update checks SHALL be on for a new config and for an existing config that has no setting for them, and every other setting from the file SHALL be kept.

#### Scenario: Config from before updates
- **WHEN** snapr launches with a `config.toml` that has no update check setting
- **THEN** automatic checks are on and the rest of the config is unchanged

### Requirement: A newer version is announced
When a check finds a newer version, the app SHALL show a system notification naming the version, add a tray menu item "Update to <version>" and show the version in About. The notification SHALL be shown at most once per version per run; the tray item and About SHALL keep showing the version until snapr is updated or quits. Notification and tray text SHALL follow the app language (en / vi).

#### Scenario: First time found
- **WHEN** a check finds 1.6.0 for the first time
- **THEN** a notification names 1.6.0 and the tray menu shows "Update to 1.6.0"

#### Scenario: Found again
- **WHEN** the 6-hour check finds 1.6.0 again
- **THEN** no new notification is shown and the tray item stays

#### Scenario: A later version appears
- **WHEN** 1.6.0 was announced and a later check finds 1.6.1
- **THEN** a notification names 1.6.1 and the tray item reads "Update to 1.6.1"

#### Scenario: No update
- **WHEN** no newer version is available
- **THEN** the tray menu has no update item

### Requirement: The user starts every install
The app SHALL download and install an update only after the user chooses the tray update item or the Update action in About. It SHALL never install on its own, on quit or at launch.

#### Scenario: Ignored update
- **WHEN** an update is announced and the user does nothing
- **THEN** snapr keeps running the current version and nothing is downloaded

#### Scenario: Update from the tray
- **WHEN** the user chooses "Update to 1.6.0" in the tray menu
- **THEN** the settings window opens on About showing the download progress and the update proceeds

### Requirement: No update during a recording
An update SHALL NOT start while a recording or a recording countdown is running. Choosing Update then SHALL show a message to stop the recording first and leave the recording untouched.

#### Scenario: Recording running
- **WHEN** a recording is running and the user chooses Update
- **THEN** the recording continues, nothing is downloaded and the user is told to stop the recording first

#### Scenario: After stopping
- **WHEN** the user stops the recording and then chooses Update
- **THEN** the update proceeds

### Requirement: Downloads are verified before installing
The app SHALL install only an installer whose signature verifies against the public key built into snapr. A download that fails, is cancelled or does not verify SHALL install nothing, SHALL leave the running version unchanged and SHALL show the error in About with a Retry action.

#### Scenario: Valid download
- **WHEN** the installer downloads and its signature verifies
- **THEN** the install proceeds

#### Scenario: Bad signature
- **WHEN** the downloaded installer's signature does not match the built-in key
- **THEN** nothing is installed, snapr keeps running and About shows the error with Retry

#### Scenario: Network drop during download
- **WHEN** the connection fails during the download
- **THEN** nothing is installed and About shows the error with Retry

### Requirement: Install replaces snapr in place and restarts it
After a verified download the app SHALL run the installer without prompts beyond its progress window, exit, and the installer SHALL start the new version. The install SHALL keep the config, the captures, the downloaded OCR models and the Start with Windows setting, and SHALL require no administrator prompt.

#### Scenario: Successful update
- **WHEN** the user updates from 1.5.0 to 1.6.0
- **THEN** snapr closes, the installer runs, snapr 1.6.0 starts in the tray and About shows 1.6.0

#### Scenario: Settings kept
- **WHEN** snapr restarts after an update
- **THEN** hotkeys, output folder, ring, theme and every other setting are as before, and it is not a first run

#### Scenario: Start with Windows kept
- **WHEN** Start with Windows was on before the update
- **THEN** after the update snapr still starts at login

### Requirement: Releases are published from version tags
Pushing a tag `v<version>` to the private repository SHALL build the Windows installer, sign it for the updater and publish the installer, its signature and `latest.json` as a release of `quanvudev/snapr-releases` that becomes its latest release. The release SHALL fail without publishing when the tag does not equal the app version.

#### Scenario: Tagged release
- **WHEN** the owner pushes tag `v1.6.0` and the app version is 1.6.0
- **THEN** a release `v1.6.0` with the installer, `.sig` and `latest.json` appears in `snapr-releases` and installed copies see 1.6.0

#### Scenario: Mismatched tag
- **WHEN** the owner pushes tag `v1.6.0` and the app version is 1.5.1
- **THEN** the release fails and nothing is published

#### Scenario: Source stays private
- **WHEN** anyone browses `snapr-releases`
- **THEN** it shows the release files and no source code
