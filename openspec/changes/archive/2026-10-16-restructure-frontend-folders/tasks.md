# Tasks

## 1. Non-UI modules

- [x] 1.1 Create `src/lib/api.ts` (types `Config`, `Hotkeys`, `Row`, `Loaded` and one typed wrapper per Tauri command: `load`, `defaults`, `resolveDir`, `check`, `save`, `openFolder`, `openConfigFile`, `openSettings`), `src/lib/i18n.ts` (`T`, `Strings`, `LANGS`) and `src/lib/hotkeys.ts` (`MODS`, `SLOTS`, `normalize`). Verify: no `invoke(` string calls remain outside `lib/api.ts`.
- [x] 1.2 Create `src/hooks/use-settings.ts` (state, load, persist, update, reset, `settings-open` listener, splash timer, view) and `src/hooks/use-hotkey-recorder.ts` (recording + keydown/Escape), moving logic verbatim from `App.tsx`. Verify: `bunx tsc --noEmit` passes.
- [x] 1.3 Create `src/components/settings-provider.tsx` (`SettingsProvider`, `useSettingsContext`) and `src/components/title-bar.tsx`. Verify: `bunx tsc --noEmit` passes.

## 2. Screens

- [x] 2.1 Create `src/components/splash/{splash.tsx,index.ts}` and `src/components/setup/{setup.tsx,index.ts}`; setup imports rows only from `@/components/settings`. Verify: `bunx tsc --noEmit` passes.
- [x] 2.2 Create `src/components/settings/` with `settings.tsx`, `sections.ts`, `general/` (general + folder/clipboard/startup/language rows), `preview/preview.tsx`, `hotkeys/` (hotkeys + modifiers-row + hotkey-row, exporting `HotkeyRows`), `about/about.tsx`, and `index.ts` exporting `Settings` plus the rows setup uses. Rows read state from `useSettingsContext()`; inside `settings/` imports are relative. Verify: no file under `components/settings/` imports `@/components/settings`.
- [x] 2.3 Reduce `src/App.tsx` to `useSettings` + `SettingsProvider` + view switch; delete `src/rows.tsx` and `src/shared.ts`. Verify: `bunx tsc --noEmit` passes and `grep -r "shared\"\|rows\"" src` finds nothing.

## 3. Docs and check

- [x] 3.1 Update the frontend bullet in `CLAUDE.md` to describe the new layout and the barrel rule (screen-root `index.ts` only; relative imports inside a screen; `ui/` and `title-bar.tsx` never import screens). Verify: every path named in CLAUDE.md exists.
- [x] 3.2 Manual check with `bun run tauri dev`: first-run splash → setup → Start, tray Settings with all four groups, autosave, hotkey recording and Escape behave exactly as before. Verify: no visible difference from before the refactor.

## Workflow follow-up

- Archive after review.
