# Design

## Context

See proposal.md for motivation. Current frontend: `src/App.tsx` (321 lines), `src/rows.tsx` (210), `src/shared.ts`, plus shadcn in `src/components/ui/`, `src/hooks/use-mobile.ts` (used by `sidebar.tsx`) and `src/lib/utils.ts`. The `@/` alias maps to `src/` in both `tsconfig.json` and `vite.config.ts`. File names in `components/ui/` and `hooks/` are kebab-case.

## Goals / Non-Goals

**Goals:**
- One obvious place for each kind of code; adding a settings group or row touches only its own folder plus one list entry.
- Screens import each other only through a small, explicit public API.

**Non-Goals:**
- Any UI, behavior or styling change.
- Changing `components/ui/`, `lib/utils.ts`, `lib/highlighter.tsx`, `hooks/use-mobile.ts`, `hooks/use-highlighted-code.ts`.
- A separate `features/` folder (decided: screens live under `components/`).
- Renaming `App.tsx` / `App.css`.

## Decisions

**Target layout.**
```
src/
  main.tsx
  App.tsx                     view selection only (splash | setup | settings)
  App.css
  lib/
    api.ts                    Config, Hotkeys, Row, Loaded types + one typed function per Tauri command
    i18n.ts                   T table, Strings, LANGS
    hotkeys.ts                MODS, SLOTS, normalize()
  hooks/
    use-settings.ts           cfg/autostart/dir/rows/failed state, load, persist, update, reset, settings-open listener
    use-hotkey-recorder.ts    recording index + keydown handling (record, Escape cancels / hides)
  components/
    ui/                       shadcn, untouched, no barrel
    title-bar.tsx             drag region + X
    settings-provider.tsx     SettingsProvider + useSettingsContext()
    splash/    index.ts  splash.tsx
    setup/     index.ts  setup.tsx
    settings/  index.ts  settings.tsx  sections.ts
      general/  general.tsx  folder-row.tsx  clipboard-row.tsx  startup-row.tsx  language-row.tsx
      preview/  preview.tsx
      hotkeys/  hotkeys.tsx  modifiers-row.tsx  hotkey-row.tsx
      about/    about.tsx
```

**Barrels only at screen roots.** `splash/index.ts`, `setup/index.ts`, `settings/index.ts` re-export named exports; nested folders have none. `settings/index.ts` exports `Settings` and the rows setup reuses (`FolderRow`, `ClipboardRow`, `StartupRow`, `LanguageRow`, `HotkeyRows`).
- Outside a screen: `import { Settings, FolderRow } from "@/components/settings"`.
- Inside a screen: relative paths only (`./general/folder-row`); a file never imports its own screen's barrel, which prevents circular imports.
- Generic files (`components/ui/*`, `title-bar.tsx`) never import from screen folders.
- Rationale: explicit public API per screen with minimal Vite dev cost; components keep meaningful file names instead of `index.tsx`.

**Context instead of the `p` prop.** `use-settings.ts` owns the state and actions currently in `App`; `App` calls it once and wraps the views in `SettingsProvider`. Rows call `useSettingsContext()` and keep only their own local drafts (folder, preview seconds). The `view` flag (setup = local edits, settings = autosave) stays in the hook so `update` behaves as today. Alternative: keep prop drilling; rejected, every new row would repeat it.

**Sections list.** `sections.ts` exports `[{ id, icon, component }]`; `settings.tsx` renders the sidebar and the active section from it. Section titles stay keyed by `id` in `T`.

**Typed API.** `lib/api.ts` exposes e.g. `api.load()`, `api.check(hotkeys)`, `api.save(config, autostart)`, `api.openSettings()`; components and hooks never call `invoke` with a string directly.

## Risks / Trade-offs

- [A refactor can silently change behavior: splash timer, setup local-vs-autosave, Escape handling] → move logic verbatim into hooks; verify with `bunx tsc --noEmit` and the manual flow from `add-splash-onboarding-settings` task 5.2.
- [Context re-renders all rows on any change] → negligible at ~10 rows.
