# Changelog

All notable changes to Snapr. Versions follow semantic versioning.

## [1.6.0] - 2026-10-29

### Breaking
- The installer is now a per-user NSIS setup only; the MSI is no longer published. If you installed the MSI, uninstall it and install `Snapr_1.6.0_x64-setup.exe` once. Later versions update in place.

### Added
- In-app updates: Snapr checks for a new version at launch and every 6 hours, shows a notification and an "Update to x.y.z" tray item, and installs it when you choose Update. Updates wait while a recording or countdown is running.
- Settings, General: "Check for updates automatically" (on by default).
- Settings, About: the update status with Check now, download progress and Retry.
