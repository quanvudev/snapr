# Proposal

## Why

Users often capture the screen only to get the text in it (an error message, a code snippet, a UI label) and then retype it. Windows 10/11 ships an offline OCR engine, so snapr can copy the text of a selected area directly without bundling any model.

## What Changes

- New "Capture text" hotkey: opens the area picker; the selected area is recognized with the Windows built-in OCR and the text is copied to the clipboard. No PNG is saved and no image preview toast is shown.
- Recognition uses the user's Windows profile languages. There is no language picker.
- Results are reported with a Windows notification: "Copied N lines" on success, "No text found" when nothing is recognized (clipboard untouched), and an error message on failure. Notification text follows the app language (en / vi).
- New "Enable text capture" switch in the Capture settings group (default on).
- Text capture is available only when Windows has at least one OCR language. When it does not, the switch is disabled with a "No OCR language installed" hint and an action that opens the Windows language settings. Availability is checked at startup and each time the settings window opens.
- The text hotkey is registered only while the switch is on and OCR is available; an inactive slot is ignored by hotkey validation.
- Hotkeys grow from four to five slots. Defaults become fullscreen `1`, window `2`, area `3`, text `4`, record `5`. **BREAKING** (defaults only): the default record hotkey moves from `4` to `5`.
- An existing `config.toml` without a text hotkey keeps all its keys (including record `4`) and leaves text capture unbound until the user records a key. No migration.

## Capabilities

### New Capabilities
- `text-capture`: extracting text from a selected screen area with the Windows OCR engine, copying it to the clipboard, reporting the result, and the availability/enable rules for the feature.

### Modified Capabilities
- `settings-window`: the Capture group gains the text capture switch; the Hotkeys group shows five action hotkeys including text.
- `app-startup`: the setup page shows five action hotkeys; older configs get record default `5` and an unbound text hotkey.

## Impact

- Backend: new `src-tauri/src/ocr.rs`; `config.rs` (text hotkey, `text_capture` switch, five combos, new defaults); `lib.rs` (new action, picker mode, worker branch, conditional registration, availability re-check on settings open); `settings.rs` (five rows, inactive/empty slot rules, availability in `load`, command to open Windows language settings).
- Frontend: `lib/api.ts`, `lib/hotkeys.ts` (`SLOTS`), `lib/i18n.ts`, new row in `components/settings/capture/`, hotkey rows render the fifth slot (settings and setup).
- Dependencies: `windows` crate (WinRT OCR, imaging, buffer features) and `tauri-plugin-notification`.
- Not affected: screenshot capture, padding, recording behavior (only the record default key changes).
