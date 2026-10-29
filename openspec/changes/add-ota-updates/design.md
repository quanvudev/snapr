# Design

## Context

- Snapr is a tray app: `setup` in `lib.rs` builds `State`, the tray (`build_tray`, items Gallery / Settings… / Open folder / Stop recording / Quit, with `State.stop` the only stored item) and calls `apply()`. No window is shown on a configured launch.
- Long-running work with UI progress already has a pattern: the PaddleOCR download (`paddle.rs` + `hooks/use-paddle.ts`): backend thread, a throttled `*-progress` event, a `*-done` event, a status command the hook polls on `settings-open`.
- Recording state lives in two places: `State.recording` (`Mutex<Option<Recorder>>`, main thread can read it) and `Snap.countdown` (capture thread only).
- `Config` is `#[serde(default)]` at struct level, so a missing field takes its value from `Config::default()`.
- Bundles today: `targets: "all"` (MSI + NSIS), NSIS default install mode. Version lives in `tauri.conf.json`, `Cargo.toml` and `package.json`; the frontend reads it with `getVersion()`.
- No CI exists. The source repo is private; builds are published to the public `quanvudev/snapr-releases` (see proposal).

## Goals / Non-Goals

**Goals:**
- All update logic in Rust, so the background check works with the settings window never opened and the frontend only renders state.
- Reuse the Paddle download shape (status command + progress / done events) so the About row mirrors `paddle-model-row.tsx`.

**Non-Goals:**
- No `@tauri-apps/plugin-updater` on the JS side and no updater permissions in `capabilities/default.json`.
- No version-bump tooling; the tag check in CI is the guard.
- No caching of a downloaded installer across runs.

## Decisions

### Updater: `tauri-plugin-updater`, NSIS passive, per-user
- `tauri.conf.json`: `bundle.targets: ["nsis"]`, `bundle.createUpdaterArtifacts: true`, `bundle.windows.nsis.installMode: "currentUser"`, `plugins.updater.pubkey: <minisign public key>`, `plugins.updater.endpoints: ["https://github.com/quanvudev/snapr-releases/releases/latest/download/latest.json"]`, `plugins.updater.windows.installMode: "passive"`. `bundle.homepage` → `https://github.com/quanvudev/snapr-releases`.
- `currentUser` keeps the install under `%LOCALAPPDATA%\Snapr` with no UAC, and the same exe path the HKCU `Run` value (`autostart_set`) already points to, so Start with Windows survives updates.
- Passive mode shows only the NSIS progress bar; Tauri's NSIS updater relaunches the app after installing.
- Alternatives: MSI (needs admin for per-machine, and mixing MSI and NSIS users creates two installs); a hand-rolled downloader (would re-implement signature checks the plugin already does).

### `update.rs` owns the state machine
`State.update: Mutex<Updater>` with the last found `tauri_plugin_updater::Update` (or `None`), the `announced` version (notification once per version per run) and a `busy` flag (checking / downloading). Commands:
- `update_status` → `{ state, version, done, total, error }` where `state` is `idle | checking | latest | available | downloading | installing | error`.
- `update_check` → runs a check now (manual; ignores `update_check` config).
- `update_install` → guard, download, guard again, install.

Events to `main`: `update-status` (same shape as `update_status`, progress throttled to 100 ms like `paddle-progress`). The hook `use-update.ts` calls `update_status` on mount and on `settings-open`, and listens to `update-status`.
- Alternative: JS-side plugin calls. Rejected: the 6-hour check must run with no webview interaction, and the recording guard needs `State`.

### Scheduling
`update::start(app)` from `setup` spawns one `tauri::async_runtime` task: wait ~10 s after launch (let tray and hotkeys settle, and the network come up after login), then loop: if `cfg.update_check` and not `busy`, check silently; sleep 6 h. Automatic failures are dropped (no tooltip, no event state change beyond back to the previous state). `settings::save` triggers an immediate check when `update_check` goes from false to true. Automatic checks are skipped in debug builds (`cfg!(debug_assertions)`) so `tauri dev` never nags; Check now still works there.

### Announcing
On a found version newer than `announced`: set `announced`, send a notification through the existing `tauri-plugin-notification` (en/vi like `read_text`, from `cfg.language`), insert the tray item and emit `update-status`.
- Tray: the `Menu` and an `update` `MenuItem` are stored in `State`; `Menu::insert(&item, 4)` (above Quit) when first found, `set_text` on a later version. Tauri menu items cannot be hidden, so insert/remove is the only way to make it absent. The `"update"` menu event calls `show_settings(app, "about")` and then `update_install`.
- `show_settings` with `"about"` opens the Settings section and scrolls to `group-about`: `use-settings` maps a group id payload to section `settings` plus a scroll after render.

### Recording guard
Add `State.counting: AtomicBool`, set in `Snap::begin` when a countdown starts and cleared wherever `Snap.countdown` is taken (start, cancel, failed start). The guard is `State.recording.lock().is_some() || counting`. It runs before downloading and again right before `install`, because a recording can start during the download; if it fails the second time the bytes are dropped and the row shows the blocked message (the user presses Update again later).
- Alternative: stop the recording automatically like Quit does. Rejected: a silent stop loses the user's take timing; the spec asks to refuse.

### Download and install
`Update::download(on_chunk, on_finish)` then the guard, then `Update::install(bytes)`. The plugin verifies the minisign signature in `download`; a mismatch is an error. On Windows `install` launches the installer and exits the process, so before calling it the handler emits `installing` and nothing else needs cleanup: config is saved on every change, recordings are excluded by the guard, pins are in memory by design.

### Config
`Config.update_check: bool`, `true` in `Config::default()`, so a new config and an old one without the field both load as on (struct-level `#[serde(default)]`). Frontend: `update_check` in the `Config` type, `update-check-row.tsx` in `settings/general/` (Switch, same shape as `startup-row.tsx`), an update row in `settings/about/` (Button / Progress / Spinner from `components/ui/`).

### Release workflow
`.github/workflows/release.yml` on `push: tags: ['v*']`, `runs-on: windows-latest`:
1. Checkout, `oven-sh/setup-bun`, Rust stable, `Swatinem/rust-cache` (workspace `src-tauri`).
2. Version check: the tag without `v` must equal the version in `tauri.conf.json`, `Cargo.toml` and `package.json`; otherwise fail.
3. `bun install`, then `tauri-apps/tauri-action` with `tauriScript: bunx tauri`, `owner: quanvudev`, `repo: snapr-releases`, `tagName: ${{ github.ref_name }}`, `releaseDraft: false`, `includeUpdaterJson: true`, env `GITHUB_TOKEN: ${{ secrets.RELEASES_TOKEN }}`, `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`.
- `RELEASES_TOKEN` is a fine-grained PAT with Contents read/write on `snapr-releases` only.
- Alternative: a local `gh` script. Rejected by the owner in favour of CI.

## Risks / Trade-offs

- [Losing the private signing key strands every install] → owner keeps the key file and password in a password manager in addition to the GitHub secret; documented in tasks.
- [1.5.0 and older have no updater] → the first updater release (1.6.0) must be installed by hand once; later ones arrive in-app.
- [MSI installs do not migrate] → anyone on the MSI uninstalls it and installs the NSIS build once; NSIS-only from now on.
- [GitHub `releases/latest` points at the newest non-draft, non-prerelease release] → a broken release reaches everyone; mitigation is publishing a fixed higher version (no rollback, per proposal).
- [SmartScreen warns on first download] → accepted, out of scope (Authenticode).
- [Install exits the process abruptly, including the winit capture thread and the mouse hook] → acceptable: the guard excludes recordings, nothing else holds unsaved state.
- [Private repo Actions minutes: Windows counts double] → ~30 billed minutes per release, well within the free 2000.

## Migration Plan

1. Owner, once: create public `quanvudev/snapr-releases` with a README; `bunx tauri signer generate -w ~/.tauri/snapr.key`; back up key + password; add `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`, `RELEASES_TOKEN` secrets to the private repo; paste the public key into `tauri.conf.json`.
2. Ship 1.6.0 through the workflow; existing users install it manually from the releases page.
3. Verify the path end to end with a 1.6.1 release: a 1.6.0 install is notified, updates and restarts with settings intact.
4. Rollback: there is no downgrade; a bad release is superseded by a higher version.
