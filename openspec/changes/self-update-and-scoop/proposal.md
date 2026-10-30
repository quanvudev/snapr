# Proposal

## Why

Snapr is to be installable with Scoop, but the current updater (`tauri-plugin-updater`) can only run the NSIS installer. A Scoop copy that updated itself that way would install a second, NSIS-managed snapr beside the Scoop one. One update mechanism that works wherever snapr runs from, an in-place replacement of its own exe, removes the need for per-channel builds or install-type detection.

## What Changes

- **BREAKING** (internal): `tauri-plugin-updater` is removed. The app checks `latest.json` itself, downloads the signed bare exe of the new version, verifies its minisign signature with the existing public key, swaps it with the running exe (rename to `.old`, move new in), starts the new exe and exits. The new process waits for the old one to exit before it starts, and removes the leftover `.old` file.
- Every release additionally publishes `Snapr_<version>_x64.exe` and its `.sig`. The installer, its `.sig` and `latest.json` keep being published, so copies at 1.8.0 or earlier (still on the plugin) update through the installer once and use the new mechanism from then on.
- After an update of an NSIS install the app writes the new version into its Add/Remove Programs entry.
- Snapr is published as a Scoop package: `snapr-releases` carries `bucket/snapr.json`, written by the Release workflow and the local release script on every release, so `scoop bucket add snapr https://github.com/quanvudev/snapr-releases` + `scoop install snapr` works and `scoop update` follows releases.
- Uninstalling the Scoop package removes the Start with Windows entry.
- Start with Windows records the stable Scoop `current` path when snapr runs from a Scoop install, so it survives `scoop update`.
- Unchanged: announcing, the tray item, About, the recording block, Retry, automatic checks.

## Capabilities

### New Capabilities
- `scoop-package`: publishing snapr as a Scoop package from `snapr-releases` and how a Scoop install behaves (install, update through Scoop, Start with Windows path, uninstall cleanup).

### Modified Capabilities
- `app-update`: verification now covers the downloaded exe instead of the installer; the install step replaces the exe in place and restarts instead of running the installer; releases publish the bare exe and its signature too; copies from before this change still update through the installer.

## Impact

- `src-tauri/src/update.rs` (rewritten download / verify / swap / restart), `src-tauri/src/lib.rs` (plugin registration removed, `--after <pid>` wait and `.old` cleanup at startup), `src-tauri/src/settings.rs` (autostart path).
- `src-tauri/Cargo.toml`: drop `tauri-plugin-updater`, add `minisign-verify`, add `semver` if not already pulled in directly; `ureq` is reused. `tauri.conf.json`: `plugins.updater` keeps only `pubkey` (the bundler needs it to sign the installer, and the app verifies with it); `createUpdaterArtifacts` stays so the installer keeps its `.sig` for older copies.
- `.github/workflows/release.yml` and `scripts/release.ts`: sign and upload the bare exe, write and commit `bucket/snapr.json` to `quanvudev/snapr-releases`.
- `CLAUDE.md` update section and `README` of `snapr-releases` (Scoop install instructions).
- Scoop shows the version it installed until the next `scoop update`, even after snapr updated itself (accepted trade-off).
