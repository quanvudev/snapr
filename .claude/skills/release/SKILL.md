---
name: release
description: Release a new Snapr version from this machine. Reads the commits and the actual diff since the last release, decides the next semantic version, writes the CHANGELOG.md entry and release notes, bumps the three version files, commits, pushes master and publishes the signed installer to quanvudev/snapr-releases with `bun run release`. Use only when the user asks to release, publish or ship a new version.
disable-model-invocation: true
argument-hint: "[major|minor|patch|x.y.z] [--yes]"
---

Release Snapr locally. Installed apps update from the latest release of the public `quanvudev/snapr-releases`, whatever made it, so this replaces the manual Release workflow for the version it ships.

`gh` may not be on PATH; use `"/c/Program Files/GitHub CLI/gh.exe"` in Bash when `gh` is missing.

Arguments (`$ARGUMENTS`, any order): an optional `major` / `minor` / `patch` / `x.y.z` that overrides the decided bump, and an optional `--yes` that skips the confirmation in step 6.

## 1. Preconditions

Stop and report if any fails:

- The branch is `master`, `git status --porcelain` is empty, and after `git fetch` it is level with `origin/master`.
- `~/.tauri/snapr.key` and `~/.tauri/snapr.key.password` exist, or `TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` are set.
- The versions in `src-tauri/tauri.conf.json`, `package.json` and `src-tauri/Cargo.toml` are equal.
- No Release workflow run is in progress: `gh run list -R quanvudev/snapr -w Release -s in_progress` is empty.

## 2. Collect the changes

- Last released version: `gh release view -R quanvudev/snapr-releases --json tagName --jq .tagName` (strip `v`). With no release yet, the base is the root commit.
- Base commit: the oldest commit that introduced that version line in `src-tauri/tauri.conf.json`:
  `git log --format=%H -G'"version": "<last>"' -- src-tauri/tauri.conf.json | tail -1`.
- Commits: `git log --format='%h%x09%s%n%b%x1e' <base>..HEAD`. If there are none, stop: nothing to release.
- Diff: `git diff --stat <base>..HEAD`, then read the hunks that matter for users. Do not stop at commit messages: one commit here often bundles several features and fixes. Read in this order and skip the rest:
  1. `openspec/changes/**` and `openspec/changes/archive/**` added in the range: each `proposal.md` What Changes and each delta spec's ADDED / MODIFIED / REMOVED requirements are the best summary of user-visible behaviour.
  2. `src-tauri/src/config.rs`, `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`, `lib.rs` (`Action`, tray menu, `generate_handler!`), `src/lib/i18n.ts` (new user-facing strings), `src/components/settings/**` (new rows and screens).
  3. Other `src-tauri/src/*.rs` and `src/**` hunks only to tell a fix from a feature when 1 and 2 leave it unclear.
  Ignore `CHANGELOG.md`, `CLAUDE.md`, `.claude/**`, `scripts/**`, `.github/**`, lockfiles and formatting-only hunks when judging the bump.

## 3. Decide X.Y.Z

Classify every user-visible change, then take the highest level found. Commit types (`feat`, `fix`, `!`, `BREAKING CHANGE`) are a hint; the diff decides.

- **Major (X)**: an existing user loses something or must act after updating. A config field removed or renamed without a `#[serde(default)]` / migration so an existing `config.toml` loses the setting; a default hotkey or default behaviour changed for existing configs; a feature, tray item, hotkey slot or setting removed; the output file name, folder or format changed; install location or install mode changed so the update needs a reinstall; a delta spec REMOVED requirement or a MODIFIED one that drops behaviour.
- **Minor (Y)**: something new the user can see or use, backward compatible. A new capture mode, setting, hotkey slot, tray item, ring action, gallery action, UI page or row, OCR engine or model, a new `ADDED` requirement.
- **Patch (Z)**: everything else that reaches users: bug fixes, performance, visual polish, copy or translation fixes, dependency bumps without new behaviour.
- **Nothing**: only `CLAUDE.md`, `.claude/**`, `openspec` planning without code, `scripts/**`, `.github/**`. Stop and tell the user there is nothing to release.

Bump from the last released version and reset the lower parts (1.6.0 + minor = 1.7.0, + major = 2.0.0). An argument version overrides the decision; an explicit `x.y.z` must be greater than the last release. Keep a one-line reason per level that triggered, citing the change (e.g. "minor: new Recording countdown setting (config.rs record_countdown)").

## 4. Write the changelog

`CHANGELOG.md` at the repo root, Keep a Changelog style, newest first. Create it with this header if missing:

```
# Changelog

All notable changes to Snapr. Versions follow semantic versioning.
```

Insert the new entry below the header:

```
## [<version>] - <YYYY-MM-DD>

### Breaking
- ...

### Added
- ...

### Changed
- ...

### Fixed
- ...
```

- One bullet per user-visible change from step 3, in English, written for users (what they can now do or what no longer goes wrong), not file or function names. Merge related changes; split a bundled commit into its real changes.
- Breaking bullets also say what the user has to do.
- Omit empty sections. If only internal changes reach users, write a single `### Changed` bullet `Maintenance and stability improvements.`
- Do not rewrite older entries.

## 5. Release notes

Write the new entry's body (its sections, without the `## [<version>]` heading) to a temporary notes file in the scratchpad. It is shown on the GitHub release and stored in `latest.json`.

## 6. Confirm

Unless `--yes` was given, show: last version → new version, the reasons from step 3, and the changelog entry. Ask the user to confirm, to pick another level, or to edit the entry. Apply their edits to both `CHANGELOG.md` and the notes file. Stop if they decline. Publishing is public and the version number cannot be reused.

## 7. Bump, commit, push

- Set `"version"` in `src-tauri/tauri.conf.json` and `package.json`, and the `[package]` `version` in `src-tauri/Cargo.toml`.
- Run `cargo update -p snapr-v2 --offline` in `src-tauri/` (fall back to without `--offline`) so `Cargo.lock` matches.
- Commit only those four files and `CHANGELOG.md`: `chore: release v<version>`, ending with the attribution lines from the system reminder.
- `git push origin master`. Never create or push a `v<version>` tag, and do not run the manual Release workflow for this version.

## 8. Publish

Run `bun run release <notes-file>` from the repo root with a 10 minute timeout. It refuses an already released version, builds and signs, writes `latest.json` and creates the release with `gh`. On failure, report the output; the release commit stays pushed, so after fixing the cause rerun only this step.

## 9. Verify

- `gh release view v<version> -R quanvudev/snapr-releases --json isDraft,isPrerelease,assets` shows the installer, its `.sig` and `latest.json`, not draft or prerelease.
- `curl -sL https://github.com/quanvudev/snapr-releases/releases/latest/download/latest.json` reports the new version, and the `url` in it downloads (`curl -sIL <url>` ends in 200).

Report the version, the release URL and the changelog entry, and remind the user that installed copies pick it up at their next check, or at once with Settings, About, Check now.
