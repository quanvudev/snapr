# Proposal

## Why

The gallery only offers actions one item at a time. Cleaning up a burst of screenshots or recordings means repeating the context menu and the delete confirmation for every file. Multi-select with bulk actions makes the common cleanup flow one gesture and one confirmation.

## What Changes

- Add selection to the gallery grid:
  - A long press (~500ms) enters selection mode and selects the pressed item without opening the viewer.
  - Ctrl+click toggles an item, Shift+click selects a range, a Select toggle shows checkboxes, Ctrl+A selects every item in the current filter and Escape leaves selection mode.
  - Photos, videos and other files are all selectable; the viewer stays single-item.
- Add a bulk action bar with the selected count, Select all, Delete, Copy, Pin and Cancel:
  - Delete moves every selected file to the Recycle Bin after one confirmation. Files that cannot be deleted stay selected and their errors are reported.
  - Copy puts the selected photo files on the clipboard so they can be pasted as files; videos and other files are not copied.
  - Pin creates one pin per selected photo, cascaded so the pins do not overlap, using the existing screen-pin feature.
  - Copy and Pin act on the selected photos only and are disabled when the selection has no photos; Delete acts on the whole selection.
- Selection is cleared when leaving selection mode and when the filter or grouping changes; a gallery reload drops paths that no longer exist.
- Add guarded bulk commands `gallery_delete_many`, `gallery_copy_files` and `gallery_pin_many`; every path is validated before any file or clipboard change.
- Out of scope: bulk open/reveal, selection in the viewer, rectangle drag-selection, selection that persists across sessions, bulk rename, and bulk upload.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `gallery`: adds grid multi-select, the bulk action bar, bulk delete/copy/pin behavior and the bulk file-access rules.

## Impact

- Backend: `gallery.rs` gains the three bulk commands and a shared per-file delete helper; `lib.rs` gains `Action::PinMany`; `pin.rs` gains a placement offset for cascaded pins; the Windows clipboard file-list API needs its `windows-sys` features.
- Frontend: `hooks/use-gallery.ts` owns selection state and bulk actions; `gallery.tsx`, `gallery-group.tsx` and `gallery-item.tsx` handle selection and long-press; a new `bulk-bar.tsx` holds the actions; `delete-dialog.tsx` accepts a list; `lib/api.ts` and `lib/i18n.ts` gain the commands and strings.
- Depends on the screen-pin feature from `add-screen-pin` for bulk pinning. No config, output format, hotkey or gallery listing changes.
