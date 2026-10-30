# Spec Delta

## MODIFIED Requirements

### Requirement: Downloads are verified before installing
The app SHALL install only an exe whose signature verifies against the public key built into snapr. A download that fails, is cancelled or does not verify SHALL install nothing, SHALL leave the running version and its exe file unchanged and SHALL show the error in About with a Retry action.

#### Scenario: Valid download
- **WHEN** the new exe downloads and its signature verifies
- **THEN** the install proceeds

#### Scenario: Bad signature
- **WHEN** the downloaded exe's signature does not match the built-in key
- **THEN** nothing is installed, the downloaded file is removed, snapr keeps running and About shows the error with Retry

#### Scenario: Network drop during download
- **WHEN** the connection fails during the download
- **THEN** nothing is installed and About shows the error with Retry

### Requirement: Install replaces snapr in place and restarts it
After a verified download the app SHALL replace its own exe file with the new one in the folder it runs from, start the new version and exit, without any installer, prompt or administrator rights. The new version SHALL wait for the old process to exit before it registers hotkeys or opens windows. The install SHALL keep the config, the captures, the downloaded OCR models and the Start with Windows setting, and SHALL work the same for an installer copy and a Scoop copy.

#### Scenario: Successful update
- **WHEN** the user updates from 1.9.0 to 1.10.0
- **THEN** snapr closes, snapr 1.10.0 starts in the tray from the same folder and About shows 1.10.0

#### Scenario: No hotkey conflict on restart
- **WHEN** the new version starts while the old process is still exiting
- **THEN** it waits for the old process to exit and then registers every hotkey without an in-use error

#### Scenario: Settings kept
- **WHEN** snapr restarts after an update
- **THEN** hotkeys, output folder, ring, theme and every other setting are as before, and it is not a first run

#### Scenario: Start with Windows kept
- **WHEN** Start with Windows was on before the update
- **THEN** after the update snapr still starts at login

#### Scenario: Replacement fails
- **WHEN** the new exe cannot be moved into place
- **THEN** the old exe is restored, snapr keeps running and About shows the error with Retry

#### Scenario: Old file removed
- **WHEN** the new version has started after an update
- **THEN** the previous exe file left beside it is deleted

#### Scenario: Installer copy shows the new version
- **WHEN** a copy installed with the installer updates to 1.10.0
- **THEN** Windows' installed apps list shows Snapr 1.10.0

### Requirement: Releases are published by hand
The owner SHALL publish a release either by running the Release workflow by hand or with the local release script. Either SHALL build snapr, sign the installer and the bare exe for the updater and publish the installer, the exe, both signatures, `latest.json` and the Scoop manifest update as a release `v<version>` of `quanvudev/snapr-releases` that becomes its latest release. It SHALL fail without publishing when the project versions disagree or the version is already released.

#### Scenario: Manual release
- **WHEN** the app version is 1.10.0 and the owner runs the Release workflow
- **THEN** a release `v1.10.0` with `Snapr_1.10.0_x64-setup.exe`, `Snapr_1.10.0_x64.exe`, their `.sig` files and `latest.json` appears in `snapr-releases` and installed copies see 1.10.0

#### Scenario: Version already released
- **WHEN** the owner runs a release for a version that already exists in `snapr-releases`
- **THEN** the release fails and nothing is published

#### Scenario: Source stays private
- **WHEN** anyone browses `snapr-releases`
- **THEN** it shows the release files, the Scoop manifest and README, and no source code

## ADDED Requirements

### Requirement: Copies from before self-update still update
Every release SHALL keep publishing the installer, its signature and a `latest.json` that copies at 1.8.0 or earlier understand, so they update through the installer to the new version, which then updates itself in place from then on.

#### Scenario: Old copy updates
- **WHEN** snapr 1.8.0 finds and installs the first release with self-update
- **THEN** the installer installs it as before and that version starts

#### Scenario: Next update after migration
- **WHEN** that version later finds a newer release
- **THEN** it updates by replacing its exe, without running an installer
