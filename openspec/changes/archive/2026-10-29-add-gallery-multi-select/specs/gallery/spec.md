# Spec Delta

## ADDED Requirements

### Requirement: Gallery supports multi-select

The gallery SHALL support selecting multiple grid items. A long press of about 500 ms SHALL enter selection mode and select the pressed item without opening the viewer. Ctrl+click SHALL toggle an item, Shift+click SHALL select the range from the anchor in the filtered order, a Select toggle SHALL enter and leave selection mode with checkboxes, Ctrl+A SHALL select every item in the current filter, and Escape SHALL leave selection mode.

#### Scenario: Long press

- **WHEN** the user holds the left button on a tile for about 500 ms
- **THEN** selection mode starts, the tile is selected, and the viewer does not open

#### Scenario: Long press an other file

- **WHEN** the user long-presses an other-file tile
- **THEN** selection mode starts and the other file is selected

#### Scenario: Ctrl toggles

- **WHEN** the user Ctrl+clicks an unselected tile and then Ctrl+clicks it again
- **THEN** the tile becomes selected and then deselected

#### Scenario: Shift selects a range

- **WHEN** the user Shift+clicks a tile while another item is the anchor
- **THEN** every item between them in the filtered order is selected

#### Scenario: Select all

- **WHEN** the user presses Ctrl+A while the gallery is shown
- **THEN** every item in the current filter is selected

#### Scenario: Escape leaves selection mode

- **WHEN** the user presses Escape in selection mode
- **THEN** selection mode ends and the selection clears

#### Scenario: Select toggle

- **WHEN** the user turns the Select toggle on
- **THEN** checkboxes appear and clicking a tile toggles it

### Requirement: Selection is grid-only

Selection SHALL apply only to the gallery grid. Outside selection mode a plain click SHALL open the viewer as before. In selection mode a plain click SHALL toggle the tile and SHALL NOT open the viewer. The viewer SHALL keep its single-item navigation, context menu and delete.

#### Scenario: Normal click opens the viewer

- **WHEN** the gallery is not in selection mode and the user clicks a photo or video
- **THEN** the viewer opens on that item

#### Scenario: Selection click toggles

- **WHEN** selection mode is on and the user clicks a tile
- **THEN** the tile toggles its selection and no viewer opens

#### Scenario: Viewer stays single-item

- **WHEN** the viewer is open
- **THEN** it shows one item and its delete removes only that item

### Requirement: Selection lifecycle

Leaving selection mode and changing the filter or grouping SHALL clear the selection. A gallery reload SHALL keep only selected paths that still exist. Collapsing or expanding a group SHALL NOT change the selection.

#### Scenario: Filter or grouping changes

- **WHEN** the user changes the filter or grouping
- **THEN** selection mode ends and the selection clears

#### Scenario: Leaving selection mode

- **WHEN** the user leaves selection mode
- **THEN** the selection clears

#### Scenario: Reload prunes missing paths

- **WHEN** the gallery reloads and a selected file no longer exists
- **THEN** that path leaves the selection

#### Scenario: Collapsing keeps selection

- **WHEN** a group is collapsed or expanded
- **THEN** its selected items stay selected

### Requirement: Bulk action bar

While selection mode is active or the selection is not empty, the gallery SHALL show a bar with the selected count, Select all, Delete, Copy, Pin and Cancel. Delete SHALL be enabled for any non-empty selection. Copy and Pin SHALL be enabled only when at least one selected item is a photo. Cancel SHALL leave selection mode and clear the selection.

#### Scenario: Bar and count

- **WHEN** the selection becomes non-empty
- **THEN** the bar shows the selected count and the actions

#### Scenario: Copy and Pin need a photo

- **WHEN** the selection has no photos
- **THEN** Copy and Pin are disabled; when at least one photo is selected they are enabled

#### Scenario: Mixed selection

- **WHEN** the selection has photos and videos
- **THEN** Delete acts on all of them and Copy or Pin act on the photos only

#### Scenario: Cancel

- **WHEN** the user chooses Cancel
- **THEN** selection mode ends and no file changes

### Requirement: Bulk delete

Delete SHALL ask for one confirmation for the whole selection and move every selected file to the Windows Recycle Bin. Deleted items SHALL leave the gallery and the selection. Files that fail SHALL stay selected and an error SHALL be shown. Cancelling SHALL change nothing.

#### Scenario: Confirm a bulk delete

- **WHEN** the user confirms Delete for three selected items
- **THEN** all three are in the Recycle Bin and disappear from the grid

#### Scenario: Cancel a bulk delete

- **WHEN** the user cancels the confirmation
- **THEN** no file changes

#### Scenario: Partial failure

- **WHEN** one selected file is locked and the user confirms Delete
- **THEN** the other files are deleted, the locked item stays selected, and an error is shown

#### Scenario: Mixed kinds

- **WHEN** the selection includes photos, videos and other files
- **THEN** all of them are moved to the Recycle Bin

### Requirement: Bulk copy photos

Copy SHALL place the selected photo files on the clipboard as a file list so they can be pasted as files. Videos and other files SHALL be ignored. Copy SHALL be disabled when the selection has no photos. The clipboard SHALL stay unchanged when the photo files cannot be validated or copied.

#### Scenario: Copy multiple photos

- **WHEN** the user chooses Copy with two photos selected
- **THEN** the clipboard holds those two files and pasting into Explorer creates them

#### Scenario: Mixed selection copies photos only

- **WHEN** the selection has a photo and a video and the user chooses Copy
- **THEN** only the photo is placed on the clipboard

#### Scenario: Single photo in bulk mode

- **WHEN** one photo is selected and the user chooses Copy
- **THEN** the clipboard holds the file, not the image bitmap

#### Scenario: No photos

- **WHEN** the selection has only videos or other files
- **THEN** Copy is disabled

#### Scenario: Copy failure

- **WHEN** a selected photo cannot be read
- **THEN** the clipboard is unchanged and an error is shown

### Requirement: Bulk pin photos

Pin SHALL create one pin per selected photo, offset in a cascade so the pins do not overlap, using the screen-pin behavior. Videos and other files SHALL be ignored. Pin SHALL be disabled when the selection has no photos. A photo that cannot be decoded SHALL be skipped and reported while the remaining photos pin.

#### Scenario: Pin multiple photos

- **WHEN** the user chooses Pin with three photos selected
- **THEN** three pins appear in a cascade and each can be moved and closed independently

#### Scenario: Mixed selection pins photos only

- **WHEN** the selection has a photo and a video and the user chooses Pin
- **THEN** only the photo is pinned

#### Scenario: One photo

- **WHEN** one photo is selected and the user chooses Pin
- **THEN** one pin appears at the cursor

#### Scenario: No photos

- **WHEN** the selection has no photos
- **THEN** Pin is disabled

#### Scenario: Decode failure

- **WHEN** one selected photo cannot be decoded
- **THEN** it is skipped with an error and the other photos pin

### Requirement: Bulk file access is limited to the output folder

Every bulk delete, copy and pin path SHALL point to a file directly inside the resolved output folder and be of a type the action accepts. If any path is invalid, the whole request SHALL be refused before any file or clipboard change. Delete accepts photos, videos and other files; copy and pin accept photos only.

#### Scenario: Outside the folder

- **WHEN** a bulk request includes a path outside the output folder
- **THEN** no file is deleted, copied or pinned and the clipboard is unchanged

#### Scenario: Mixed valid and invalid delete

- **WHEN** a bulk delete has one valid path and one outside path
- **THEN** no file is deleted

#### Scenario: Non-photo copy

- **WHEN** a bulk copy request includes a video
- **THEN** the request is refused and the clipboard is unchanged

#### Scenario: Non-photo pin

- **WHEN** a bulk pin request includes an other file
- **THEN** no pins are created

#### Scenario: Video delete

- **WHEN** a bulk delete request includes a video inside the output folder
- **THEN** it is accepted
