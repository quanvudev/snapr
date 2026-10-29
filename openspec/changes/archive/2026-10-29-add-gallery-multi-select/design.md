# Design

## Context

- The gallery is a React screen backed by `hooks/use-gallery.ts`: it owns the item list, filter, grouping, collapsed groups, viewer index and the single-item copy/delete actions. `gallery.tsx` renders the filter and groups; `gallery-group.tsx` renders the collapsible grid; `gallery-item.tsx` is the tile and `item-menu.tsx` the context menu.
- The backend is `gallery.rs`: `gallery_list`, `gallery_open`, `gallery_reveal`, `gallery_copy`, `gallery_pin` and `gallery_delete`. Every command goes through `guard`, which requires a canonical file directly inside the output folder. `gallery_delete` uses `SHFileOperationW` with the Recycle Bin flags.
- The viewer is single-item and owns its own keyboard handling. Selection must not leak into it.
- Bulk pin builds on the `screen-pin` feature from `add-screen-pin`: `Action::Pin(PathBuf)` creates a layered pin at the cursor; `Pins` already supports multiple pins, but each `Pin::open` currently centers on the same cursor, so a batch would overlap exactly.
- `components/ui/checkbox.tsx`, `alert-dialog.tsx`, `button.tsx` and the existing context menu are available. `delete-dialog.tsx` currently takes one item.

## Goals / Non-Goals

**Goals:**

- Add selection as a first-class gallery state with both long-press and modifier/toggle gestures.
- Make Delete, Copy and Pin work on a selection with one user action and precise per-file error reporting.
- Keep every bulk operation inside the output-folder guard and keep the viewer behavior unchanged.

**Non-Goals:**

- Selection in the viewer, rectangle drag-selection, persistent selection across sessions, bulk open/reveal/rename or upload.
- Changing the single-item context menu, the single-image clipboard behavior or the existing delete semantics.
- Changing pin behavior outside the bulk placement cascade.

## Decisions

### 1. Selection lives in `useGallery`; the flat filtered list is exposed

`useGallery` gains:

```
selected: Set<string>
selectMode: boolean
anchor: string | null
toggle(item), selectRange(item), selectAll(), clearSelection(), setSelectMode()
```

It also exposes the flat `visible` list (all items matching the filter, including `other` files) in addition to the existing `items` (photos and videos only, used by the viewer). Shift ranges, Ctrl+A and the bulk bar operate on `visible`, because `other` files are deletable.

Selection is cleared when the filter or grouping changes and when the user leaves selection mode. On `gallery-changed`/reload, the new item list is intersected with `selected` so deleted or externally removed paths disappear. Collapsing a group does not change selection.

Alternatives considered:

- **Selection in component state**: duplicated between the toolbar, groups and tiles; the hook already owns reload/viewer/actions.
- **Selection only over `items`**: would prevent deleting `other` files and make Ctrl+A inconsistent with the grid.

### 2. Gestures: long-press enters selection mode; normal click keeps the viewer

```
normal --long press 500ms / Ctrl+click / Shift+click / Select toggle--> selecting
selecting --Esc/Cancel/filter or grouping change--> normal (selection cleared)
selecting --click tile--> toggle tile
```

- A pointer-down on a tile starts a 500ms timer. Moving more than a few pixels or releasing earlier cancels it. When it fires, the hook enters selection mode and selects the item, and the tile suppresses the click that follows so the viewer does not open.
- Ctrl+click toggles and enters selection mode. Shift+click selects the range from the anchor in `visible`; without an anchor it behaves like a single selection and sets the anchor. A plain click in selection mode toggles. A plain click outside selection mode keeps opening the viewer.
- The Select toggle in the gallery toolbar enters selection mode and shows a checkbox on every tile; turning it off clears. Ctrl+A selects all `visible`; Escape clears and leaves selection mode. Clicking Cancel in the bulk bar does the same.
- The tile overlay uses the existing shadcn `Checkbox`, disabled for pointer interaction so the tile handles the click/long-press itself.

Alternatives considered:

- **Long-press only**: not discoverable and awkward with a mouse; the modifier and toggle paths stay.
- **Plain click always selects after the first item**: would break the primary open-viewer action and surprise users.

### 3. The bulk bar is a separate gallery component

A new `bulk-bar.tsx` renders when `selectMode || selected.size > 0`:

- Selected count, Select all, Delete, Copy, Pin and Cancel.
- Delete is enabled whenever anything is selected; Copy and Pin are enabled only when the selection contains at least one photo.
- Copy and Pin act on the selected photo subset; Delete acts on the whole selection.
- Delete opens one confirmation dialog for all selected items. The existing `delete-dialog.tsx` is generalized to take `GalleryItem[] | null`; the viewer and single-item menu pass a one-element list, so their behavior is unchanged.

The bar sits above the groups, follows the existing `Button`/`AlertDialog` styling and uses new en/vi strings.

### 4. Bulk delete is validated all-or-nothing, deleted per-file

`gallery_delete_many(paths)` first resolves every path through `guarded`. If any path is outside the output folder or missing, it returns an error without deleting anything. It then deletes each file with a shared `delete_file` helper extracted from `gallery_delete` (same `SHFileOperationW` Recycle Bin flags), collecting failures instead of stopping at the first one. The return value carries the failed paths and their errors.

The frontend removes successful paths from the list and selection, keeps failures selected and shows the error. The existing single `gallery_delete` keeps working and uses the same helper. Deleting many files may issue several silent Recycle Bin operations; this keeps per-file failure reporting.

Alternatives considered:

- **One `SHFileOperationW` call with a path list**: fewer calls but coarser failure reporting and harder all-or-nothing validation.
- **Frontend loop over `gallery_delete`**: multiple confirmations/errors and no atomic validation.

### 5. Bulk copy uses a Windows file-list clipboard (CF_HDROP)

`gallery_copy_files(paths)` validates every path as a photo, then puts the files on the clipboard as `CF_HDROP` (a `DROPFILES` header plus a double-null-terminated UTF-16 path list). The bulk action copies files, not bitmaps, so multiple images can be pasted into Explorer or another file-aware target. The single-item context menu keeps its current bitmap `Copy image`.

The implementation needs the `Win32_System_DataExchange` and `Win32_System_Memory` features. It opens the clipboard, empties it, allocates the global memory, sets `CF_HDROP` and closes the clipboard; any failure frees the buffer and leaves the clipboard unchanged. Validation failures return before the clipboard is touched.

Alternatives considered:

- **Copy a bitmap per photo**: the clipboard holds one image, so a multi-photo copy would silently keep only the last.
- **Loop `gallery_copy`**: N bitmap copies, same last-one-wins problem and no file-list behavior.

### 6. Bulk pin cascades pins through the existing pin feature

`gallery_pin_many(paths)` validates every path as a photo, then sends `Action::PinMany(Vec<PathBuf>)` to the capture thread. `Snap::open_pins` decodes each path in order and creates a pin per photo. `Pin::open`/`Pins::add` gain an `(i32, i32)` placement offset applied before the existing work-area clamp; the batch uses `(index * 24, index * 24)` so the pins cascade instead of stacking exactly. The first pin is at the cursor. Each pin is independent for drag/copy/close; a decode failure skips that path with a tray tooltip and continues with the rest. The existing single `Action::Pin` passes `(0, 0)`.

Alternatives considered:

- **N frontend calls to `gallery_pin`**: all pins center on the same cursor point and hide each other.
- **Pin only the first photo**: not a bulk action and loses the user's selection.

### 7. Every bulk command validates before it acts

The bulk commands use the same `guard` and kind checks as the single-item commands:

- Delete accepts photos, videos and other files.
- Copy and pin accept photos only; the frontend sends the photo subset and the backend re-checks it.
- If any path in a bulk request is invalid, the request is refused before any deletion, clipboard write or pin creation. This prevents a mixed request from partially acting on untrusted paths.

## Risks / Trade-offs

- **[Long-press vs click]** A long-press timer can fire just as the user releases, opening the viewer and entering selection mode. → Cancel on movement/release, suppress the next click in the tile, and keep the threshold at 500ms; verify with mouse testing.
- **[Shift range across collapsed groups]** The range uses the filtered order, so it can include items hidden in collapsed groups. → The count in the bulk bar makes the scope visible; collapsing does not change the existing selection. Document the behavior in tests.
- **[Stale selection]** A reload or filter change could leave paths selected that are no longer shown. → Clear on filter/grouping changes and intersect with the reloaded paths; bulk commands still guard every path.
- **[CF_HDROP ownership]** Incorrect global-memory ownership or leaving the clipboard open can break clipboard access. → Use one open/empty/set/close path, free on failure, and verify pasting into Explorer.
- **[Bulk copy expectation]** The bulk Copy is a file-list copy, while the single-item Copy image is a bitmap. → Label the bulk action as copying files and keep the single-image behavior unchanged; the spec states the difference.
- **[Pin cascade memory]** Selecting many photos and pinning all creates one layered window and image buffer per pin. → The action is explicit and pins are ephemeral; no cap in this change. Revisit if users pin large batches.
- **[Per-file delete]** Multiple Recycle Bin operations can be slower and can partially fail. → Report failures per path, keep failed items selected and remove successes.
- **[Bar layout]** The bulk bar adds a row above the grid and can crowd a narrow window. → Use flex-wrap, compact buttons and the existing spacing tokens.

## Migration Plan

None. This is a UI/backend behavior change with no config, file format or stored-state migration.
