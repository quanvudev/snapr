# Proposal

## Why

Snapr has no way to reach users who already installed it: every fix means they must notice a new build exists, download it and reinstall by hand. The source stays private, so updates must come from published builds only, and since snapr runs silently in the tray from login, the app itself is the only reliable place to tell the user a new version is out.

## What Changes

- Snapr checks a public update feed at launch and every 6 hours while the "Check for updates automatically" setting is on (default on, including older configs).
- When a newer version exists: one notification per version, a tray menu item "Update to x.y.z", and an update row in About with the new version and an Update action.
- About always offers "Check now", which works even when automatic checks are off, and reports up to date / failed.
- Updating downloads the signed installer with progress in About, verifies its signature and, unless a recording or countdown is running (then it is refused), runs the installer passively and restarts snapr on the new version. Config, captures and PaddleOCR models are untouched.
- Releases come from a GitHub Actions workflow in the private repo: pushing a `v*` tag builds, signs and publishes the NSIS installer, its `.sig` and `latest.json` as a release of the public `quanvudev/snapr-releases` repo. The workflow fails if the tag does not match the app version.
- **BREAKING** (distribution only): the bundle becomes NSIS only (per-user install); the MSI is no longer produced. `bundle.homepage` points to the releases repo.
- Out of scope: Authenticode code signing (SmartScreen still warns on first download), silent or forced installs, release channels (beta), delta updates, downgrade/rollback, a changelog view inside the app.

## Capabilities

### New Capabilities

- `app-update`: checking the public feed, announcing a newer version, downloading, verifying and installing it, the guard against updating during a recording, and the release feed contract.

### Modified Capabilities

- `settings-window`: General gains the automatic update check switch; About gains the update status row with Check now / Update and download progress; the tray menu gains the update item.

## Impact

- Backend: new `update.rs` (check, schedule, download, install) on `tauri-plugin-updater`; `config.rs` `update_check` (default true); `lib.rs` plugin registration, a tray `MenuItem` in `State` for the update entry, update state, timer and commands; install path reuses `stop_recording` / countdown state as the guard.
- Frontend: `settings/general/update-check-row.tsx`, an update row in `settings/about/`, a `use-update` hook (status, progress events), `lib/api.ts` wrappers, en/vi strings.
- Config: `tauri.conf.json` gets `bundle.createUpdaterArtifacts`, `bundle.targets: ["nsis"]`, NSIS `installMode: currentUser`, `plugins.updater` (pubkey, endpoint, `windows.installMode: passive`); capability permissions for the updater if the frontend calls it.
- New dependency: `tauri-plugin-updater` (Rust).
- New CI: `.github/workflows/release.yml`. Secrets: `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, a PAT with write access to `snapr-releases`.
- Manual one-time setup by the owner: create the public `snapr-releases` repo, generate and back up the signing key, add the secrets.
