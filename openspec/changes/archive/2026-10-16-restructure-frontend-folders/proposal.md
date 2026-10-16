# Proposal

## Why

After `add-splash-onboarding-settings`, the frontend is three flat files: `App.tsx` (views, state, autosave, hotkey recording, sidebar and every section), `rows.tsx` (every setting row) and `shared.ts` (types, constants, i18n). Every row takes a `p: Ctx` prop, and Tauri commands are called by string from many places. Adding a settings group or a setting means editing the large files, so the frontend gets harder to maintain as it grows.

## What Changes

- Split the frontend by screen under `src/components/`: `splash/`, `setup/`, `settings/` (with `general/`, `preview/`, `hotkeys/`, `about/` sub-folders, one file per row).
- Each screen folder exposes a top-level `index.ts` barrel as its public API; nested folders have no barrel. Imports from outside a screen go through the barrel (`@/components/settings`); imports inside it use relative paths.
- Settings groups come from one list (`settings/sections.ts`), so adding a group is one folder plus one list entry.
- Move non-UI code to `src/lib/` (`api.ts` typed command wrappers and types, `i18n.ts`, `hotkeys.ts`) and state to `src/hooks/` (`use-settings.ts`, `use-hotkey-recorder.ts`).
- Replace the `p: Ctx` prop with a settings context (`components/settings-provider.tsx`).
- Extract the shared title bar to `components/title-bar.tsx`.
- No user-visible behavior change; no new dependencies.

## Capabilities

### New Capabilities

### Modified Capabilities

None. This is a pure frontend refactor; `skip_specs: true` is set in `.openspec.yaml`.

## Impact

- `src/App.tsx` shrinks to view selection; `src/rows.tsx` and `src/shared.ts` are removed.
- New files under `src/lib/`, `src/hooks/`, `src/components/` (except `components/ui/`, untouched).
- `CLAUDE.md` frontend notes updated.
- Backend (`src-tauri/`) unchanged.
