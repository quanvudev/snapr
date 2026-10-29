# gallery Specification

## Purpose

Lets users browse, view and manage the screenshots and recordings in the output folder from inside snapr, grouped by day like a phone photo library.

## Requirements

### Requirement: Gallery lists every file in the output folder
The gallery SHALL list every file directly inside the resolved output folder and classify it by extension, case-insensitive: photos are `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`; videos are `mp4` and `webm`; every other file is an other file. Subfolders SHALL be ignored. Files are not required to follow snapr's naming.

#### Scenario: Mixed folder
- **WHEN** the output folder holds `a.png`, `b.MP4`, `c.jpg`, `notes.txt`, `clip.mov` and a subfolder `old` containing `d.png`
- **THEN** the gallery shows `a.png` and `c.jpg` as photos, `b.MP4` as a video, `notes.txt` and `clip.mov` as other files, and nothing from `old`

#### Scenario: Empty or missing folder
- **WHEN** the output folder has no file, or does not exist
- **THEN** the gallery shows an empty state message instead of a grid

### Requirement: Item date is the earlier of created and modified time
Each item's date SHALL be the earlier of the file's created time and last modified time, in local time. The file name SHALL NOT be used to date an item.

#### Scenario: Copied file
- **WHEN** a file last modified on 2026-10-01 was copied into the folder on 2026-10-20
- **THEN** the item is dated 2026-10-01

#### Scenario: Edited file
- **WHEN** a file created on 2026-10-01 was edited and saved on 2026-10-20
- **THEN** the item is dated 2026-10-01

#### Scenario: Renamed file
- **WHEN** the user renames `2026-10-01_10-00-00.png` to `bug.png`
- **THEN** the item keeps its date

### Requirement: Items are grouped by day, month or year, newest first
The gallery SHALL offer a Day / Month / Year grouping choice, defaulting to Day. Items SHALL be grouped under one header per local calendar day, month or year, with groups in descending order and items within a group in descending date order. Each header SHALL show the period in the app language and the item count of that group.

#### Scenario: Two days
- **WHEN** grouping is Day and the folder has two items from today and one from 2026-10-24
- **THEN** the grid shows a header for today with two items followed by a header for 2026-10-24 with one item

#### Scenario: Month grouping
- **WHEN** the user selects Month and the folder has items from 2026-10-02, 2026-10-20 and 2026-09-15
- **THEN** the grid shows a header for October 2026 with two items followed by a header for September 2026 with one item

#### Scenario: Year grouping
- **WHEN** the user selects Year and the folder has items from 2026 and 2025
- **THEN** the grid shows a header for 2026 followed by a header for 2025, each with its count

#### Scenario: Grouping and filter combine
- **WHEN** grouping is Month and the filter is Videos
- **THEN** only video items are shown and months without videos are hidden

### Requirement: Groups can be collapsed
Each group header SHALL toggle its group between expanded and collapsed. Groups SHALL start expanded. A collapsed group SHALL keep its header and count and hide its items. Collapsed state SHALL last while the gallery is shown and is not saved to the config.

#### Scenario: Collapse a day
- **WHEN** the user clicks the header of today's group
- **THEN** today's items are hidden, the header and its count stay, and other groups are unchanged

#### Scenario: Expand again
- **WHEN** the user clicks the header of a collapsed group
- **THEN** its items are shown again

#### Scenario: Live update keeps collapsed state
- **WHEN** today's group is collapsed and a new capture is saved
- **THEN** today's count increases and the group stays collapsed

### Requirement: Filter by media type
The gallery SHALL offer an All / Photos / Videos filter, defaulting to All. All SHALL show every item including other files, Photos only photo items and Videos only video items; groups with no matching item SHALL be hidden.

#### Scenario: Videos only
- **WHEN** the user selects Videos
- **THEN** only video items are shown and days that contain only photos or other files disappear

#### Scenario: Other files only under All
- **WHEN** the folder holds `notes.txt` and the user selects Photos
- **THEN** `notes.txt` is not shown; selecting All shows it again

### Requirement: Thumbnails match Windows Explorer
Each photo and video item SHALL show the Windows-provided thumbnail of the file; video items SHALL carry a video marker. Other files SHALL show the Windows file-type icon with the file name. When Windows provides neither a thumbnail nor an icon, the item SHALL show a generic placeholder and remain usable. Thumbnails SHALL load only as items scroll into view.

#### Scenario: Recording thumbnail
- **WHEN** the folder holds an MP4 recording
- **THEN** its grid item shows a frame of the video with a video marker

#### Scenario: Other file
- **WHEN** the folder holds `archive.zip`
- **THEN** its grid item shows the Windows zip icon and the name `archive.zip`

#### Scenario: Thumbnail unavailable
- **WHEN** Windows fails to produce a thumbnail for a file
- **THEN** the item shows a placeholder and can still be opened, revealed and deleted

### Requirement: In-app viewer
Clicking a photo or video item SHALL open a viewer inside the window that shows the image fitted to the pane, or plays the video with playback controls and seeking. Clicking an other file SHALL do nothing. Left and Right SHALL move to the previous and next photo or video in the current filtered order, skipping other files, and Escape SHALL close the viewer and return to the grid without hiding the window.

#### Scenario: View image
- **WHEN** the user clicks a PNG item
- **THEN** the viewer shows the full image fitted to the pane

#### Scenario: Play video
- **WHEN** the user clicks an MP4 item
- **THEN** the viewer plays the video and the user can pause and seek

#### Scenario: Other file is not viewable
- **WHEN** the user clicks `notes.txt`
- **THEN** no viewer opens

#### Scenario: Navigate
- **WHEN** the viewer shows the newest item and the user presses Right
- **THEN** the viewer shows the next older photo or video in the current filter, skipping other files

#### Scenario: Escape closes viewer only
- **WHEN** the viewer is open and the user presses Escape
- **THEN** the viewer closes and the gallery grid is shown; the window stays open

### Requirement: Viewer thumbnail strip
The viewer SHALL show a horizontal strip of thumbnails below the media, containing every photo and video of the current filter in the same order as Left/Right navigation; other files are not in the strip. The item being viewed SHALL be highlighted and scrolled into the center of the strip whenever it changes. Clicking a thumbnail in the strip SHALL show that item.

#### Scenario: Active item highlighted
- **WHEN** the viewer opens on an item
- **THEN** the strip shows that item highlighted and scrolled into view

#### Scenario: Keyboard keeps strip in sync
- **WHEN** the user presses Right in the viewer
- **THEN** the next item is shown and the strip highlights it and scrolls it into view

#### Scenario: Jump from strip
- **WHEN** the user clicks a thumbnail in the strip
- **THEN** the viewer shows that item

#### Scenario: Strip follows filter
- **WHEN** the filter is Videos and the viewer is open
- **THEN** the strip contains only video items

### Requirement: Item context menu

Right-clicking an item in the grid, or the content in the viewer, SHALL show the actions Open with default app, Show in Explorer, Pin, Copy image and Delete. Pin and Copy image SHALL be shown only for photo items; other files get Open with default app, Show in Explorer and Delete.

#### Scenario: Open with default app

- **WHEN** the user chooses Open with default app on an MP4 item
- **THEN** the file opens in the Windows default app for MP4

#### Scenario: Show in Explorer

- **WHEN** the user chooses Show in Explorer
- **THEN** Explorer opens the output folder with the file selected

#### Scenario: Copy image

- **WHEN** the user chooses Copy image on a PNG or JPG item
- **THEN** the image is placed on the clipboard

#### Scenario: Pin photo

- **WHEN** the user right-clicks a PNG or JPG item
- **THEN** the menu has a Pin action

#### Scenario: No copy for video

- **WHEN** the user right-clicks an MP4 item
- **THEN** the menu has no Copy image action

#### Scenario: No pin for video

- **WHEN** the user right-clicks an MP4 item
- **THEN** the menu has no Pin action

#### Scenario: Other file menu

- **WHEN** the user right-clicks `notes.txt` and chooses Open with default app
- **THEN** the file opens in the Windows default app for txt

#### Scenario: Other file has no pin

- **WHEN** the user right-clicks an other file
- **THEN** the menu has no Pin action

### Requirement: Delete moves to the Recycle Bin after confirmation
Delete SHALL ask for confirmation and, when confirmed, move the file to the Windows Recycle Bin and remove it from the gallery. Deleting from the viewer SHALL show the next item, or the previous one when it was the last, or close the viewer when nothing is left in the filter. A failed delete SHALL keep the item and show an error.

#### Scenario: Confirmed delete
- **WHEN** the user chooses Delete and confirms
- **THEN** the file is in the Recycle Bin and the item is gone from the grid

#### Scenario: Cancelled delete
- **WHEN** the user chooses Delete and cancels
- **THEN** the file and item are unchanged

#### Scenario: Delete in viewer
- **WHEN** the viewer shows an item that has an older item and the user deletes it
- **THEN** the viewer shows that older item

#### Scenario: Delete fails
- **WHEN** the file is locked by another process and the user confirms Delete
- **THEN** the item stays and an error is shown

### Requirement: File actions are limited to the output folder

Thumbnails, viewer content, open, reveal, copy, pin and delete SHALL only act on files directly inside the resolved output folder. Viewer content SHALL only be served for photo and video files. Pin SHALL only act on photo files. A request for any other path SHALL be refused without touching the file.

#### Scenario: Path outside the folder

- **WHEN** a request names a path outside the output folder, directly or through `..`
- **THEN** the request is refused and nothing is read, opened or deleted

#### Scenario: Pin outside the folder

- **WHEN** a pin request names a path outside the output folder, directly or through `..`
- **THEN** the request is refused and no pin appears

#### Scenario: Pin a non-photo file

- **WHEN** a pin request names a video or other file inside the output folder
- **THEN** the request is refused and no pin appears

#### Scenario: Viewer content for other file

- **WHEN** viewer content is requested for `notes.txt` inside the output folder
- **THEN** the request is refused

#### Scenario: Output folder changed

- **WHEN** the user changes the output folder in General
- **THEN** the gallery lists the new folder and files of the old folder can no longer be read

### Requirement: Gallery updates live
While the window is open, a screenshot or recording saved by snapr SHALL appear in the gallery without reopening the window. The gallery SHALL reload its list each time the window is opened.

#### Scenario: Capture while open
- **WHEN** the gallery is shown and the user takes an area capture
- **THEN** the new item appears under today's header

#### Scenario: External change
- **WHEN** a file is added to the folder from Explorer and the user reopens the window
- **THEN** the gallery shows the new file

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
