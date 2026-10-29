# Tasks

## 1. Owner setup (manual, outside the code)

- [x] 1.1 Create the public repo `quanvudev/snapr-releases` with only a README; verify it is reachable signed out
- [ ] 1.2 Run `bunx tauri signer generate -w ~/.tauri/snapr.key`, back up the key file and password in a password manager; verify `snapr.key` and `snapr.key.pub` exist
- [x] 1.3 Add secrets `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` and `RELEASES_TOKEN` (fine-grained PAT, Contents read/write on `snapr-releases` only) to the private repo; verify they appear under Settings → Secrets → Actions

## 2. Bundle and updater config

- [x] 2.1 Add `tauri-plugin-updater` to `src-tauri/Cargo.toml` and register it in `run()`; verify `cargo clippy` passes
- [x] 2.2 In `tauri.conf.json` set `bundle.targets: ["nsis"]`, `bundle.createUpdaterArtifacts: true`, `bundle.windows.nsis.installMode: "currentUser"`, `bundle.homepage` to the releases repo and `plugins.updater` (pubkey from 1.2, endpoint, `windows.installMode: "passive"`); verify `bun run tauri build` with the signing env vars produces only `nsis/Snapr_*_x64-setup.exe` and its `.sig`
- [x] 2.3 Update the CLAUDE.md bundle/command notes (NSIS only, signing env vars needed for a local release build); verify the text matches the config

## 3. Config

- [x] 3.1 Add `update_check: bool` to `Config`, `true` in `Config::default()`; verify a `config.toml` without the field loads with it on and keeps the other values
- [x] 3.2 Add `update_check` to the `Config` type in `lib/api.ts`; verify `bunx tsc --noEmit` passes
- [x] 3.3 Document `update_check` in the CLAUDE.md `config.rs` notes

## 4. Backend update module

- [x] 4.1 Add `State.counting: AtomicBool`, set where `Snap.countdown` becomes `Some` and cleared wherever it is taken (start, cancel, failed start); verify by logging during a countdown and after cancel
- [x] 4.2 Store the tray `Menu` and an `update` `MenuItem` in `State`; add an `"update"` menu event that calls `show_settings(app, "about")` and starts the install; verify the menu is unchanged while no update is found
- [x] 4.3 Create `update.rs` with `State.update` (last `Update`, `announced`, `busy`), the status shape `{ state, version, done, total, error }` and an `update-status` emitter (progress throttled 100 ms); verify `cargo clippy` passes
- [x] 4.4 Implement `update_check` (manual) and the shared check: on a newer version emit `available`, and if newer than `announced` notify (en/vi) and insert the tray item above Quit or `set_text` it; verify against a test feed that 1.6.0 notifies once and a repeat check does not
- [x] 4.5 Implement `update::start` from `setup`: 10 s delay, then check every 6 h while `cfg.update_check`, silent on failure, skipped in debug builds; verify in a release build with the network off that nothing is shown
- [x] 4.6 Trigger an immediate check from `settings::save` when `update_check` turns on; verify toggling on in settings starts a check
- [x] 4.7 Implement `update_install`: guard (`State.recording` or `counting`) → `blocked` error, download with progress, guard again, emit `installing`, `install`; verify a recording blocks it, a bad signature reports an error and installs nothing
- [x] 4.8 Implement `update_status` and register the three commands in `generate_handler!`; verify they are callable from the webview devtools
- [x] 4.9 Document `update.rs`, the state, events, commands, tray item and guard in CLAUDE.md

## 5. Frontend

- [x] 5.1 Add `updateStatus`, `updateCheck`, `updateInstall` and `onUpdateStatus` to `lib/api.ts`; verify `bunx tsc --noEmit` passes
- [x] 5.2 Add `hooks/use-update.ts` (status on mount and `settings-open`, `update-status` listener, check / install) and expose it through `use-settings` as `update`
- [x] 5.3 Map a `settings-open` payload of `"about"` to the Settings section and scroll to `group-about` after render; verify the tray update item opens the window at About
- [x] 5.4 Add `settings/general/update-check-row.tsx` (Switch, autosave) to General; verify toggling writes `update_check` to `config.toml`
- [x] 5.5 Add the About update row with every state (Check now, checking spinner, up to date, version + Update, Progress + percentage, installing, error + Retry, blocked by recording) from `components/ui/`; verify each state renders in `bun run tauri dev` with a test feed
- [x] 5.6 Add en/vi strings to `lib/i18n.ts`; verify both languages show no missing keys
- [x] 5.7 Document the new rows and hook in CLAUDE.md

## 6. Release workflow

- [x] 6.1 Add `.github/workflows/release.yml` (tag `v*`, windows-latest, bun, Rust, rust-cache, version check across `tauri.conf.json` / `Cargo.toml` / `package.json`, `tauri-action` publishing to `quanvudev/snapr-releases` with `includeUpdaterJson`); verify a mismatched test tag fails before building
- [x] 6.2 Document the release steps (bump the 3 versions, tag, push) in CLAUDE.md under Commands

## 7. End-to-end verification

- [ ] 7.1 Release 1.6.0 through the workflow; verify the release has the installer, `.sig` and `latest.json`, and `releases/latest/download/latest.json` serves 1.6.0
- [ ] 7.2 Install 1.6.0 by hand with Start with Windows on, then release 1.6.1; verify the notification and tray item appear, Update installs with no UAC, snapr restarts on 1.6.1 with settings, captures, Paddle models and Start with Windows intact

## Workflow follow-up

- Archive the change after the 1.6.1 end-to-end check passes.
