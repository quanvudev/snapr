# Tasks

## 1. Drop the plugin

- [x] 1.1 Remove `tauri-plugin-updater` from `src-tauri/Cargo.toml` and its registration in `lib.rs`, add `minisign-verify` and `semver` as direct dependencies (versions already in `Cargo.lock`), remove `plugins.updater` from `tauri.conf.json` and move its `pubkey` into `update.rs`; verify `cargo clippy` passes and no `tauri_plugin_updater` / `updater` plugin reference is left (`rg updater src-tauri src`)

## 2. Check and install without the plugin

- [x] 2.1 Rewrite `update::check` to fetch `latest.json` with `ureq` on a blocking thread, strip a leading `v`, compare with the running version through `semver` and store the found version in `Updater.found` instead of an `Update`; verify in a release build (`bun run tauri build`, run the bundled exe) that Check now in About reports 1.8.0 as "up to date" against the live feed and a lowered local version (temporarily 1.7.0) as available
- [x] 2.2 Implement download in `update::install`: fetch `Snapr_<v>_x64.exe.sig` first, stream the exe into `<exe>.new` with progress through `set`, verify with `minisign-verify` against the built-in key, delete `.new` on any failure; verify a corrupted `.sig` (local test feed or a tampered file) shows the error with Retry and leaves no `.new` file
- [x] 2.3 Implement the swap (`<exe>` to `.old`, `.new` to `<exe>`, spawn `--after <pid>`, roll back each step on failure), the recording re-check before it, `cleanup_before_exit` and exit; verify by updating a copy in a scratch folder from a lower version to the published one that it restarts from the same folder and About shows the new version
- [x] 2.4 In `run()`, wait for the `--after <pid>` process (10 s cap) before building Tauri and delete `<exe>.old` on every start; verify after 2.3's update that no hotkey shows in-use and the `.old` file is gone
- [ ] 2.5 After a successful swap, write `DisplayVersion` under `HKCU\...\Uninstall\Snapr` only when its `InstallLocation` is the exe's folder; verify with `reg query` that an installer copy shows the new version and a copy elsewhere leaves the entry untouched

## 3. Scoop-aware autostart

- [x] 3.1 In `settings.rs` `autostart_set`, write the `apps\snapr\current\<exe>` path when the exe sits in `apps\snapr\<version>\` beside a `current` folder; verify with a mock folder layout (or a real Scoop install from group 4) that `reg query HKCU\Software\Microsoft\Windows\CurrentVersion\Run /v snapr` shows the `current` path

## 4. Publishing

- [x] 4.1 In `scripts/release.ts`, after the build copy `target/release/snapr-v2.exe` to `Snapr_<v>_x64.exe`, sign it with `bunx tauri signer sign`, add it and its `.sig` to `gh release create`, then render `bucket/snapr.json` (design.md, Manifest) with its SHA-256 and PUT it to `quanvudev/snapr-releases` through `gh api` (with the existing `sha` when present); verify with a dry run that prints the manifest and the asset list without publishing, or on the real release
- [x] 4.2 Add the same steps to `.github/workflows/release.yml` after `tauri-action`, using `RELEASES_TOKEN` and the signing secrets; verify the workflow file parses (`gh workflow view release.yml`) and, on the first release, that the exe, its `.sig` and `bucket/snapr.json` appear on `snapr-releases`
- [x] 4.3 Add a Scoop section to the `snapr-releases` README (bucket add, install, `scoop update` needs snapr closed) as part of the same release steps or by hand; verify it renders on GitHub

## 5. Docs

- [x] 5.1 Update `CLAUDE.md` (`update.rs` bullet, Release commands, the bundle note about the bare exe, Scoop bucket) to describe the self-update and the manifest; verify no mention of `tauri-plugin-updater` remains (`rg -n "plugin-updater|updater_builder" CLAUDE.md`)

## 6. Integration

- [ ] 6.1 Release the version and verify end to end: a 1.8.0 installer copy updates through the installer to it; that copy then self-updates to a follow-up release; `scoop bucket add` + `scoop install snapr/snapr` installs it, an in-app update and a later `scoop update snapr` both keep the settings, and `scoop uninstall snapr` removes the Run value

## Workflow follow-up

- Archive the change after the release has been verified.
