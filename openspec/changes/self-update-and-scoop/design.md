# Design

## Context

- `update.rs` wraps `tauri-plugin-updater`: `check` builds the updater and stores the found `Update`, `install` downloads with progress, re-checks the recording block and calls `update.install(bytes)`, which runs the NSIS installer and exits. Everything around it (the `Status` / `Phase` machine, `update-status`, the tray item in `MENU_SLOT`, notifications, `start` / `enabled` / `from_tray`, the three commands) is plugin-independent and stays.
- The NSIS build installs per user into `%LOCALAPPDATA%\Snapr\` (`snapr-v2.exe`, `uninstall.exe`) and registers `HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Snapr` with `DisplayVersion` and `InstallLocation`.
- `tauri-action` builds `target/release/snapr-v2.exe`, bundles it, signs the installer (`createUpdaterArtifacts`) and writes `latest.json` (`version`, `platforms.windows-x86_64{,-nsis}.url/signature` pointing at the installer). `scripts/release.ts` does the same locally and writes `latest.json` itself.
- `ureq` (rustls) is already a direct dependency (`paddle.rs` has an `agent()` with 30 s timeouts); `minisign-verify` 0.3 and `semver` 1 are already in `Cargo.lock` through the plugin.
- Autostart writes `"<current_exe>"` to `HKCU\...\Run\snapr` (`settings.rs`).

## Goals / Non-Goals

**Goals:**
- One build and one update path for installer and Scoop copies.
- Keep every user-facing behaviour of the current updater (announce, About states, Retry, recording block).
- Copies at 1.8.0 or earlier keep updating through the plugin they ship with.

**Non-Goals:**
- Updating anything other than the exe (no resources are bundled today; the PaddleOCR runtime is downloaded at run time).
- Rolling back automatically when the new exe crashes after starting.
- Keeping Scoop's recorded version in sync after an in-app update.
- Submitting to ScoopInstaller/Extras.

## Decisions

### Feed and asset naming
`check` GETs `https://github.com/quanvudev/snapr-releases/releases/latest/download/latest.json` with `ureq` (on `spawn_blocking`), reads only `version` (a leading `v` stripped) and compares it with `app.package_info().version` using `semver`. The exe and signature URLs are derived by convention: `releases/download/v<version>/Snapr_<version>_x64.exe` and `.sig`. `latest.json` is left exactly as `tauri-action` writes it, so old copies keep reading the installer from it.
Alternative: add a custom `platforms` key to `latest.json` for the exe. Rejected: `tauri-action` writes the file, so CI would have to download, edit and re-upload it, and a convention is enough.

### Signature
The `.sig` asset is produced by `bunx tauri signer sign` with the existing key, the same format the plugin used: base64 of a minisign signature file. The public key stays in `tauri.conf.json` as `plugins.updater.pubkey` (base64 of a minisign public key file) and `update.rs` reads it from `app.config()`: the bundler refuses to sign the installer (`createUpdaterArtifacts`) when `plugins.updater` is missing, so a minimal block with only `pubkey` is kept although the plugin is gone (found in the first full build). Verification: base64-decode both, `minisign_verify::PublicKey::decode` / `Signature::decode`, `verify(bytes, false)`. The `.sig` is fetched before the exe so a missing signature fails before the large download.

### Download and swap
The exe streams into `<exe>.new` in the folder of `current_exe()` (same volume, so the swap is two renames) with progress through the existing `set` throttle. Then, with the recording block re-checked:

```
verify(<exe>.new)              fail -> delete .new, Phase::Error
rename <exe>      -> <exe>.old fail -> delete .new, Phase::Error
rename <exe>.new  -> <exe>     fail -> rename .old back, Phase::Error
spawn <exe> --after <pid>      fail -> rename back both, Phase::Error
write DisplayVersion (installer copy only)
cleanup_before_exit(); exit(0)
```

Windows lets a running exe be renamed but not overwritten or deleted, which is why the old file is moved aside instead of replaced. `Phase::Installing` covers verify-to-exit.
Alternative: a helper script that waits and copies. Rejected: an extra process and file for what two renames do.

### Startup wait and cleanup
`run()` checks for `--after <pid>` before building Tauri: `OpenProcess(SYNCHRONIZE)` + `WaitForSingleObject` (10 s cap) so hotkeys, the tray and the mouse hook are free before `setup`. Every start then deletes `<exe>.old` best-effort (it also cleans up after a crash between swap and the next start).

### Add/Remove Programs version
After a successful swap, if `HKCU\...\Uninstall\Snapr` exists and its `InstallLocation` (quotes trimmed) equals the exe's folder, `DisplayVersion` is set to the new version. A Scoop copy never matches, so it never touches an installer entry that may coexist.

### Scoop detection (autostart only)
A Scoop path is `<root>\apps\snapr\<version>\<exe>` with a sibling `<root>\apps\snapr\current`. When `current_exe()` matches that shape, `autostart_set` writes the `current\<exe>` path. No other code knows about Scoop.

### Manifest
`bucket/snapr.json` in `snapr-releases`:

```json
{
  "version": "<v>",
  "description": "Screenshot, screen recording and OCR from the tray",
  "homepage": "https://github.com/quanvudev/snapr-releases",
  "license": "Proprietary",
  "architecture": { "64bit": {
    "url": "https://github.com/quanvudev/snapr-releases/releases/download/v<v>/Snapr_<v>_x64.exe#/snapr.exe",
    "hash": "<sha256>" } },
  "bin": "snapr.exe",
  "shortcuts": [["snapr.exe", "Snapr"]],
  "uninstaller": { "script": "<remove HKCU Run 'snapr' when it points under $dir's parent>" },
  "checkver": { "github": "https://github.com/quanvudev/snapr-releases" },
  "autoupdate": { "architecture": { "64bit": {
    "url": "https://github.com/quanvudev/snapr-releases/releases/download/v$version/Snapr_$version_x64.exe#/snapr.exe" } } }
}
```

The uninstaller only removes the Run value when it points into the Scoop app folder, so an installer copy's entry survives. No `persist`: config and models live outside the app folder.

### Publishing
Both `release.yml` (after `tauri-action`) and `release.ts` (after the build):
1. copy `target/release/snapr-v2.exe` to `Snapr_<v>_x64.exe`, `bunx tauri signer sign` it (key and password from the same env vars), upload both to the release;
2. compute SHA-256, render the manifest, and PUT it with `gh api repos/quanvudev/snapr-releases/contents/bucket/snapr.json` (passing the current blob `sha` when the file exists).
The exe is the one `tauri-action` / `tauri build` already produced, so there is no second build. `createUpdaterArtifacts` stays on so the installer keeps its `.sig` for old copies; `includeUpdaterJson` stays on.

## Risks / Trade-offs

- [Antivirus or a file lock blocks a rename] → each step rolls back and reports the error with Retry; the running exe is never deleted before the new one is in place.
- [The new exe crashes on start] → the old exe stays as `.old` until the next successful start; recovery is manual (reinstall or `scoop update`). Accepted as a non-goal.
- [Scoop shows an outdated version after an in-app update] → accepted; the next `scoop update` re-downloads the same version and re-points `current`.
- [`scoop update` refuses while snapr runs] → Scoop's own message tells the user to close it; documented in the README.
- [Removing `plugins.updater` breaks `createUpdaterArtifacts`] → confirmed; a minimal `plugins.updater` with only `pubkey` is kept, which is also the key `update.rs` verifies with.
- [The manifest commit fails after the release is published] → the run fails visibly; rerunning only the manifest step (or `gh api` by hand) repairs it, the release itself is valid.

## Migration Plan

1. Release the version with this change (planned 1.9.0). Copies at 1.8.0 or earlier update through the installer from `latest.json`, unchanged.
2. From that version on, every copy updates by swapping its exe; the bucket exists from the same release, so Scoop installs start there.
3. Rollback: a later release can bring the plugin back; copies on the self-update path would still self-update to it, since the exe asset convention is independent of the plugin.
