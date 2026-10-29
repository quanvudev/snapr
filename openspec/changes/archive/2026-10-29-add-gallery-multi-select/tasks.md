# Tasks

## 1. Bulk delete backend

- [x] 1.1 Extract the per-file Recycle Bin delete from `gallery_delete` into a shared helper and add `gallery_delete_many` in `gallery.rs`: validate every path with `guarded`, refuse the whole request on an invalid path, delete the valid files, and return the failed paths and errors; register the command; verify `cargo clippy` and call it from the settings devtools with one valid and one outside path to confirm nothing is deleted.
- [ ] 1.2 Verify `gallery_delete_many` with a locked file after the UI is wired (group 5): the other files move to the Recycle Bin, the locked path is reported, and no file outside the folder is touched.

## 2. Bulk copy backend

- [x] 2.1 Add the Windows clipboard file-list support and `gallery_copy_files` in `gallery.rs`: validate every path as a photo, build a `CF_HDROP` list, and leave the clipboard unchanged on any validation or allocation failure; enable the required `windows-sys` features and register the command; verify `cargo clippy` and paste a two-file copy into Explorer.
- [ ] 2.2 Verify a copy request containing a video or an outside path is refused before the clipboard is touched, and that the single-item `gallery_copy` still copies a bitmap.

## 3. Bulk pin backend

- [x] 3.1 Add an `(i32, i32)` placement offset to `Pin::open`/`Pins::add`, keep the existing `Action::Pin` at `(0, 0)`, and add `Action::PinMany(Vec<PathBuf>)` plus `Snap::open_pins` to decode and create a cascaded pin per photo; verify `cargo clippy` and that a two-path batch creates two pins offset by about 24px.
- [ ] 3.2 Add `gallery_pin_many` (guard every path as a photo, send `Action::PinMany`) and register it; verify `cargo clippy` and that a batch with one undecodable photo still pins the valid photos and reports the failure in the tray tooltip.

## 4. Selection state and gestures

- [x] 4.1 Extend `hooks/use-gallery.ts` with `selected`, `selectMode`, `anchor`, the flat filtered `visible` list, toggle/range/select-all/clear operations and lifecycle rules (clear on filter/grouping/leave, prune on reload); verify `bunx tsc --noEmit`.
- [x] 4.2 Wire `gallery-item.tsx`, `gallery-group.tsx` and `gallery.tsx` for normal click, selection click, Ctrl+click, Shift+range, the Select toggle and the `Checkbox` overlay; implement the 500ms long-press with movement/release cancellation and click suppression; verify long-press on a photo and a video does not open the viewer, and Ctrl/Shift/Esc work.
- [x] 4.3 Add `bulk-bar.tsx` with the count, Select all, Delete, Copy, Pin and Cancel actions, generalize `delete-dialog.tsx` to a list of items, keep the viewer/single-menu delete using a one-item list, and add en/vi strings for the new labels and errors; verify `bunx tsc --noEmit` and that Copy/Pin disable when the selection has no photos.

## 5. Wire bulk actions

- [x] 5.1 Add `api.galleryDeleteMany`, `api.galleryCopyFiles` and `api.galleryPinMany`, then wire the bulk bar through `use-gallery`: Delete confirms once, removes successes, keeps failures selected and shows the error; Copy sends the selected photos; Pin sends the selected photos and keeps the selection; verify `bunx tsc --noEmit`.
- [ ] 5.2 Verify mixed selections: photos + videos + other files delete together; Copy copies only the photos as files; Pin creates only the photo pins; a selection with no photos disables Copy and Pin; the viewer and single-item context menu still work unchanged.
- [ ] 5.3 Verify the selection lifecycle in the UI: changing filter or grouping clears, leaving selection mode clears, collapsing a group keeps, a gallery reload drops missing selected paths, and Ctrl+A selects the current filter.

## 6. Integration

- [x] 6.1 Run `cargo clippy` in `src-tauri/` and `bunx tsc --noEmit`, then a `bun run tauri dev` smoke pass over every scenario in `specs/gallery/spec.md`; record any gap.
- [x] 6.2 Update `CLAUDE.md` with the gallery selection state, bulk commands, CF_HDROP copy behavior and pin cascade; verify the description matches the implementation.

## Workflow follow-up

- Run `openspec validate add-gallery-multi-select --strict` and resolve any warnings before review.
- Archive the change after the implementation is reviewed.
