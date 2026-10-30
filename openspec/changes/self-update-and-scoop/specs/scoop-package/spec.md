# Spec Delta

## Purpose

Lets users install, update and uninstall snapr with Scoop from a bucket kept in the public `snapr-releases` repository, with the same build and the same in-app updates as the installer copy.

## ADDED Requirements

### Requirement: snapr-releases is a Scoop bucket
The public `quanvudev/snapr-releases` repository SHALL hold a Scoop manifest for snapr at `bucket/snapr.json`, so that adding the repository as a bucket and installing `snapr` installs the latest release's bare exe with a Start menu shortcut and a `snapr` command.

#### Scenario: Install from the bucket
- **WHEN** a user runs `scoop bucket add snapr https://github.com/quanvudev/snapr-releases` and `scoop install snapr/snapr`
- **THEN** the latest released version is installed, a Snapr Start menu shortcut exists and `snapr` starts it

#### Scenario: Hash checked
- **WHEN** the downloaded exe does not match the manifest's hash
- **THEN** Scoop refuses to install it

### Requirement: Every release updates the manifest
The Release workflow and the local release script SHALL, after publishing a release, set the manifest's version, download URL and SHA-256 to that release's bare exe and commit it to the default branch of `snapr-releases`. The manifest SHALL also let Scoop's own `checkver` / `autoupdate` find a release from its GitHub tag.

#### Scenario: Release published
- **WHEN** the owner releases 1.10.0
- **THEN** `bucket/snapr.json` on `snapr-releases` names 1.10.0, its exe URL and its hash, and `scoop update snapr` installs 1.10.0

#### Scenario: Manifest step fails
- **WHEN** the release is published but committing the manifest fails
- **THEN** the run reports the failure and the release itself stays published

### Requirement: Scoop copies update in place like every other copy
A Scoop copy SHALL show and install in-app updates exactly as an installer copy does. `scoop update snapr` SHALL keep working afterwards, installing the bucket's version even when snapr already updated itself to it.

#### Scenario: In-app update of a Scoop copy
- **WHEN** a Scoop copy of 1.9.0 is updated to 1.10.0 from About
- **THEN** snapr 1.10.0 runs from the Scoop folder and the config is unchanged

#### Scenario: Later scoop update
- **WHEN** that copy runs `scoop update snapr` while the bucket names 1.10.0
- **THEN** Scoop installs 1.10.0 and snapr starts and keeps its settings

### Requirement: Start with Windows survives Scoop updates
When snapr runs from a Scoop install and Start with Windows is turned on, the login entry SHALL point at the exe through Scoop's `current` folder instead of a version folder.

#### Scenario: After scoop update
- **WHEN** Start with Windows was turned on in a Scoop copy and `scoop update snapr` then installs a new version
- **THEN** snapr still starts at the next login

### Requirement: Uninstalling the Scoop package leaves no login entry
Uninstalling snapr with Scoop SHALL remove snapr's Start with Windows entry. The config, captures and downloaded OCR models SHALL be left in place.

#### Scenario: Uninstall
- **WHEN** Start with Windows was on and the user runs `scoop uninstall snapr`
- **THEN** nothing starts at the next login and `%APPDATA%\com.quanvudev.snapr` still exists
