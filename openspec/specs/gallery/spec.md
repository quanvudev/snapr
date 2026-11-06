# gallery Specification

## Purpose

Lets users browse, view and manage the screenshots and recordings in the output folder from inside snapr, grouped by day like a phone photo library.

## Requirements

### Requirement: Gallery lists every file in the output folder
The gallery SHALL list every file directly inside the resolved output folder, its `Images` subfolder and its `Videos` subfolder, and classify it by extension, case-insensitive: photos are `png`, `jpg`, `jpeg`, `gif`, `webp` and `bmp`; videos are `mp4` and `webm`; every other file is an other file. Any other subfolder, and anything nested deeper inside `Images` or `Videos`, SHALL be ignored. Files are not required to follow snapr's naming, and a file is listed whichever of the three folders it is in. An `srt` file whose name without extension equals that of a video in the same folder is that video's transcript and SHALL NOT be listed. An `srt` file named `<video name>.<code>.srt`, case-insensitive, where `<code>` is a supported translation target, for a video in the same folder is that video's translation and SHALL NOT be listed either.

#### Scenario: Mixed folder
- **WHEN** the output folder holds `a.png`, `notes.txt` and subfolders `Images` containing `c.jpg`, `Videos` containing `b.MP4` and `clip.mov`, and `old` containing `d.png`
- **THEN** the gallery shows `a.png` and `c.jpg` as photos, `b.MP4` as a video, `notes.txt` and `clip.mov` as other files, and nothing from `old`

#### Scenario: Nested folder
- **WHEN** `Images` contains a subfolder `x` holding `e.png`
- **THEN** the gallery does not show `e.png`

#### Scenario: Empty or missing folder
- **WHEN** the output folder, `Images` and `Videos` have no file, or do not exist
- **THEN** the gallery shows an empty state message instead of a grid

#### Scenario: Transcript beside its video
- **WHEN** `Videos` holds `a.mp4`, `a.srt` and `b.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.srt` as an other file, and does not show `a.srt`

#### Scenario: Transcript in another folder
- **WHEN** `Videos` holds `a.mp4` and the output folder root holds `a.srt`
- **THEN** the gallery shows `a.srt` as an other file and `a.mp4` has no transcript

#### Scenario: Translation beside its video
- **WHEN** `Videos` holds `a.mp4`, `a.en.srt` and `b.en.srt`
- **THEN** the gallery shows `a.mp4` as a video and `b.en.srt` as an other file, and does not show `a.en.srt`

#### Scenario: Several translations
- **WHEN** `Videos` holds `a.mp4`, `a.srt`, `a.vi.srt`, `a.zh-Hant.srt` and `a.xx.srt`
- **THEN** the gallery shows `a.mp4` and `a.xx.srt` (not a supported code), and hides the others

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

Right-clicking an item in the grid, or the content in the viewer, SHALL show the actions Open with default app, Show in Explorer, Pin, Copy image, Extract text, Export GIF, Compress video and Delete. Pin and Copy image SHALL be shown only for photo items, Extract text only for photo items while Windows has an OCR language, and Export GIF and Compress video only for MP4 items; other files get Open with default app, Show in Explorer and Delete.

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

#### Scenario: Extract text from a grid item

- **WHEN** an OCR language is installed and the user chooses Extract text on a PNG item in the grid
- **THEN** the viewer opens on that item with the text pane open

#### Scenario: No extract text for video

- **WHEN** the user right-clicks an MP4 item
- **THEN** the menu has no Extract text action

#### Scenario: No extract text without OCR

- **WHEN** no OCR language is installed and the user right-clicks a PNG item
- **THEN** the menu has no Extract text action

#### Scenario: Export GIF from a video

- **WHEN** the user right-clicks an MP4 item and chooses Export GIF
- **THEN** the GIF export dialog opens for that video

#### Scenario: No export GIF for photos or other files

- **WHEN** the user right-clicks a PNG item or `notes.txt`
- **THEN** the menu has no Export GIF action

#### Scenario: Compress a video

- **WHEN** the user right-clicks an MP4 item and chooses Compress video
- **THEN** the compress dialog opens for that video

#### Scenario: No compress for photos or other files

- **WHEN** the user right-clicks a PNG item or `notes.txt`
- **THEN** the menu has no Compress video action

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

Thumbnails, viewer content, open, reveal, copy, extract text, pin, delete and every video task (export GIF, compress, transcribe, translate, export with subtitles, trim) SHALL only act on files directly inside the resolved output folder, its `Images` subfolder or its `Videos` subfolder. Viewer content SHALL only be served for photo and video files. Pin and extract text SHALL only act on photo files. A request for any other path SHALL be refused without touching the file.

#### Scenario: Path outside the folder

- **WHEN** a request names a path outside those three folders, directly or through `..`
- **THEN** the request is refused and nothing is read, opened or deleted

#### Scenario: File in a subfolder

- **WHEN** a request names `Videos\a.mp4` or `Images\b.png` inside the output folder
- **THEN** it is accepted

#### Scenario: File in another subfolder

- **WHEN** a request names `old\d.png` or `Images\x\e.png` inside the output folder
- **THEN** the request is refused

#### Scenario: Pin outside the folder

- **WHEN** a pin request names a path outside those three folders, directly or through `..`
- **THEN** the request is refused and no pin appears

#### Scenario: Pin a non-photo file

- **WHEN** a pin request names a video or other file inside the output folder
- **THEN** the request is refused and no pin appears

#### Scenario: Extract text outside the folder

- **WHEN** an extract text request names a path outside those three folders or a video inside them
- **THEN** the request is refused and nothing is recognized

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

While selection mode is active or the selection is not empty, the gallery SHALL show a bar with the selected count, Select all, Delete, Copy, Pin, Export GIF, Compress video and Cancel. Delete SHALL be enabled for any non-empty selection. Copy and Pin SHALL be enabled only when at least one selected item is a photo. Export GIF and Compress video SHALL be enabled only when at least one selected item is an MP4. Cancel SHALL leave selection mode and clear the selection.

#### Scenario: Bar and count

- **WHEN** the selection becomes non-empty
- **THEN** the bar shows the selected count and the actions

#### Scenario: Copy and Pin need a photo

- **WHEN** the selection has no photos
- **THEN** Copy and Pin are disabled; when at least one photo is selected they are enabled

#### Scenario: Video actions need a video

- **WHEN** the selection has no MP4 items
- **THEN** Export GIF and Compress video are disabled; when at least one MP4 is selected they are enabled

#### Scenario: Mixed selection

- **WHEN** the selection has photos and videos
- **THEN** Delete acts on all of them, Copy or Pin act on the photos only, and Export GIF or Compress video act on the videos only

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

Every bulk delete, copy and pin path SHALL point to a file directly inside the resolved output folder, its `Images` subfolder or its `Videos` subfolder, and be of a type the action accepts. If any path is invalid, the whole request SHALL be refused before any file or clipboard change. Delete accepts photos, videos and other files; copy and pin accept photos only.

#### Scenario: Outside the folder

- **WHEN** a bulk request includes a path outside those three folders
- **THEN** no file is deleted, copied or pinned and the clipboard is unchanged

#### Scenario: Mixed valid and invalid delete

- **WHEN** a bulk delete has one valid path and one outside path
- **THEN** no file is deleted

#### Scenario: Files from both subfolders

- **WHEN** a bulk delete includes `Images\a.png` and `Videos\b.mp4`
- **THEN** both are accepted

#### Scenario: Non-photo copy

- **WHEN** a bulk copy request includes a video
- **THEN** the request is refused and the clipboard is unchanged

#### Scenario: Non-photo pin

- **WHEN** a bulk pin request includes an other file
- **THEN** no pins are created

#### Scenario: Video delete

- **WHEN** a bulk delete request includes a video inside the output folder
- **THEN** it is accepted

### Requirement: Viewer text pane
While viewing a photo with an OCR language installed, the viewer SHALL offer a toggle that opens a pane to the right of the image showing the photo's recognized content: each decoded QR code, marked as QR, then the recognized text, keeping its lines. The text SHALL be selectable and copyable but not editable, and a Copy all action SHALL copy all of it. The pane SHALL NOT change the clipboard or show a notification on its own.

#### Scenario: Open the pane

- **WHEN** the user views a PNG containing two lines of text and turns on the text toggle
- **THEN** a pane on the right shows a loading indicator, then the two lines, and the clipboard is unchanged

#### Scenario: Read-only text

- **WHEN** the text pane shows recognized text
- **THEN** the user can select it and copy it with Ctrl+C, but cannot type into or change it

#### Scenario: Copy all

- **WHEN** the user chooses Copy all in the text pane
- **THEN** the clipboard holds every QR content and the recognized text shown in the pane

#### Scenario: QR code in the photo

- **WHEN** the viewed photo contains a QR code and a caption
- **THEN** the pane shows the QR content marked as QR followed by the caption text

#### Scenario: No text

- **WHEN** the viewed photo has no QR code and no recognizable text
- **THEN** the pane says no text was found

#### Scenario: Recognition fails

- **WHEN** the photo cannot be decoded or recognition fails and no QR code was found
- **THEN** the pane shows the error

### Requirement: Text pane follows the viewer
The text pane SHALL stay open while the user moves between photos with Left/Right or the thumbnail strip and SHALL show the content of the photo being viewed, recognizing each photo at most once while the gallery is shown. It SHALL be hidden on videos. Escape SHALL close the pane before it closes the viewer, and closing the viewer SHALL close the pane.

#### Scenario: Next photo

- **WHEN** the text pane is open and the user presses Right to another photo
- **THEN** the pane shows that photo's content

#### Scenario: Back to a seen photo

- **WHEN** the user returns to a photo whose text was already shown
- **THEN** the pane shows it immediately without recognizing it again

#### Scenario: Video

- **WHEN** the text pane is open and the user moves to a video
- **THEN** the pane is hidden, and it reappears on the next photo

#### Scenario: Escape

- **WHEN** the text pane is open and the user presses Escape
- **THEN** the pane closes and the viewer stays open

### Requirement: Export GIF from the viewer
While the viewer shows an MP4 item, its header SHALL offer an Export GIF button that opens the GIF export dialog for that video. The button SHALL NOT be shown for photos.

#### Scenario: Viewer shows a video
- **WHEN** the viewer shows an MP4 item and the user clicks Export GIF
- **THEN** the GIF export dialog opens for that video

#### Scenario: Viewer shows a photo
- **WHEN** the viewer shows a PNG item
- **THEN** the header has no Export GIF button

### Requirement: Transcribe from the gallery
The context menu of an MP4 item SHALL offer Transcribe, which queues its transcription. The bulk action bar SHALL offer Transcribe, enabled when the selection contains an MP4, which queues one transcription per selected MP4 in visible order and ignores other items. Photos SHALL NOT offer it.

#### Scenario: Context menu
- **WHEN** the user right-clicks an MP4 and chooses Transcribe
- **THEN** one transcription of that video is added to the queue

#### Scenario: Bulk
- **WHEN** two MP4s and a PNG are selected and the user chooses Transcribe
- **THEN** two transcriptions are queued, in the order shown, and the PNG is ignored

#### Scenario: Photo
- **WHEN** the user right-clicks a PNG
- **THEN** the menu has no Transcribe

### Requirement: Deleting a video removes its transcript
Deleting an MP4, from the grid, the viewer or a bulk delete, SHALL also move its `<video name>.srt` and every `<video name>.<code>.srt` translation to the Recycle Bin when they exist. A failure to remove any of them SHALL NOT fail the video's delete.

#### Scenario: Delete a transcribed video
- **WHEN** the user deletes a video that has a transcript and confirms
- **THEN** both the video and its `.srt` are in the Recycle Bin

#### Scenario: Delete a translated video
- **WHEN** the user deletes a video that has `.srt`, `.en.srt` and `.ja.srt` and confirms
- **THEN** the video and all three subtitle files are in the Recycle Bin

#### Scenario: Delete a video without transcript
- **WHEN** the user deletes a video that has no `.srt` and confirms
- **THEN** the video is in the Recycle Bin and no error is shown

### Requirement: Compress video from the viewer
While the viewer shows an MP4 item, its header SHALL offer a Compress video button that opens the compress dialog for that video. The button SHALL NOT be shown for photos.

#### Scenario: Viewer shows a video
- **WHEN** the viewer shows an MP4 item and the user clicks Compress video
- **THEN** the compress dialog opens for that video

#### Scenario: Viewer shows a photo
- **WHEN** the viewer shows a PNG item
- **THEN** the header has no Compress video button

### Requirement: Video language is known
Snapr SHALL remember the spoken language of each MP4: a finished transcription records the language Whisper detected, or the fixed transcript language when one is set. The record SHALL survive a restart and SHALL be dropped when the video is deleted.

#### Scenario: Recorded by transcription
- **WHEN** transcript language is Auto-detect and a Vietnamese video finishes transcribing
- **THEN** the video's language is Vietnamese, also after snapr restarts

### Requirement: Video language is detected on demand
For an MP4 with no recorded language, opening the transcript pane SHALL detect it from the first 30 seconds of audio with the selected Whisper model when that model is downloaded, without blocking the viewer, and record it. A video with no audio track or no downloaded model SHALL stay unknown.

#### Scenario: Detected on demand
- **WHEN** a video was transcribed by an older snapr, the Small model is downloaded and the user opens its transcript pane
- **THEN** after a moment the video's language is known and the pane and context menu reflect it

#### Scenario: Model missing
- **WHEN** no Whisper model is downloaded and the user opens the transcript pane of a video with no recorded language
- **THEN** the language stays unknown and no error is shown

### Requirement: Translate to English from the gallery
While the translation model is not downloaded, the context menu of an MP4 whose known language is not English SHALL offer Translate to English, which queues one English translation of its speech with Whisper. A finished translation SHALL be written as `<video name>.en.srt` beside the video, replacing an older one, and SHALL NOT change `<video name>.srt`. It SHALL be refused, queueing nothing, when the selected Whisper model is not downloaded or the video is known to be English. Photos, English videos and videos of unknown language SHALL NOT offer it.

#### Scenario: Translate a Vietnamese recording
- **WHEN** the translation model is not downloaded and the user right-clicks a Vietnamese MP4 that has `a.srt` and chooses Translate to English
- **THEN** a translate item appears in the queue, and when it finishes `a.en.srt` holds English text with timings while `a.srt` is unchanged

#### Scenario: Replaced by the translation model
- **WHEN** the translation model is downloaded
- **THEN** the menu offers Translate subtitles… instead of Translate to English

#### Scenario: English video
- **WHEN** the user right-clicks an MP4 whose language is English
- **THEN** the menu has no Translate to English

#### Scenario: Unknown language
- **WHEN** the user right-clicks an MP4 whose language is not known yet
- **THEN** the menu has no Translate to English

#### Scenario: Model missing
- **WHEN** the selected Whisper model is not downloaded and the user chooses Translate to English on a Vietnamese video
- **THEN** nothing is queued and an error toast says the model is not downloaded

#### Scenario: Photo
- **WHEN** the user right-clicks a PNG
- **THEN** the menu has no Translate to English

#### Scenario: Cancel keeps the old translation
- **WHEN** a video already has `a.en.srt` and the user cancels a new translation of it
- **THEN** `a.en.srt` is unchanged and no partial file is left

### Requirement: Transcript pane tracks
The transcript pane SHALL offer a track choice listing Original (`.srt`) and every translation the video has, each named by its language. Each track SHALL have its own loading, running, error and segment states. Editing, Copy all and seeking SHALL act on the shown track only. The chosen language SHALL persist across videos that have it; otherwise Original is shown. Without the translation model the English entry follows the Whisper fallback rules.

#### Scenario: English video has no switch
- **WHEN** the translation model is not downloaded and the user opens the transcript pane of an English video with only `a.srt`
- **THEN** no track choice is shown and the original transcript is listed

#### Scenario: Switch appears after detection
- **WHEN** the translation model is not downloaded, the pane of a video with unknown language is open and detection finds Vietnamese
- **THEN** the track choice appears with Original and English

#### Scenario: Switch to English
- **WHEN** a video has `a.srt` and `a.en.srt` and the user picks English in the track choice
- **THEN** the English segments are listed, clicking one seeks the video and the active segment follows playback

#### Scenario: Edit the English track
- **WHEN** the user edits an English segment and presses Enter
- **THEN** `a.en.srt` is rewritten with the new text and `a.srt` is unchanged

#### Scenario: Several translations
- **WHEN** a video has `a.srt`, `a.en.srt` and `a.ja.srt`
- **THEN** the track choice lists Original, English and Japanese, and picking Japanese lists the Japanese segments with seeking and the active segment

#### Scenario: Translate from the pane
- **WHEN** the translation model is downloaded and the pane shows a video with a transcript and a known language
- **THEN** a Translate action opens the target picker, and a queued translation shows its progress under that language in the track choice

#### Scenario: No translation yet
- **WHEN** the translation model is not downloaded, a Vietnamese video has only `a.srt` and the user picks English
- **THEN** the pane offers Translate to English, and choosing it queues a Whisper translation and shows its progress

#### Scenario: Stays on the finished translation
- **WHEN** the pane shows the running Japanese translation and it reaches 100%
- **THEN** the pane keeps showing Japanese and lists the translated segments, without switching back to Original

#### Scenario: Follows a new translation
- **WHEN** the user picks Japanese in the target picker for the video shown in the viewer
- **THEN** the transcript pane switches to the Japanese track and shows its progress

#### Scenario: Original names its language
- **WHEN** the video's language is Vietnamese
- **THEN** the Original entry reads "Original (Vietnamese)" in the track choices of the pane and of Export with subtitles

#### Scenario: Tracks run independently
- **WHEN** a translation of a video is running and the user switches to Original
- **THEN** the original transcript is listed and editable, and switching back shows the running progress

#### Scenario: Edit a translation
- **WHEN** the user edits a Japanese segment and presses Enter
- **THEN** `a.ja.srt` is rewritten with the new text and `a.srt` is unchanged

#### Scenario: Edit refused while translating
- **WHEN** a translation of a video into a language is waiting or running
- **THEN** editing is hidden on that track, and a save of that track is refused

### Requirement: Export with subtitles chooses a track
The Export with subtitles dialog SHALL offer a Track choice among Original and every translation the video has when there is more than one, preselecting the track shown in the transcript pane when opened from it and Original otherwise. With one track the choice SHALL be hidden and that track used. The preview cue SHALL come from the chosen track. A translation export SHALL be written as `<video name>_subtitled_<code>.mp4` (`_N` when taken), an original one as `<video name>_subtitled.mp4`.

#### Scenario: Export English subtitles
- **WHEN** a video has both tracks and the user picks English and exports
- **THEN** a subtitle item is queued and the result `a_subtitled_en.mp4` shows the English cues

#### Scenario: Export Japanese subtitles
- **WHEN** a video has `a.srt` and `a.ja.srt` and the user picks Japanese and exports
- **THEN** the result is `a_subtitled_ja.mp4` with the Japanese cues

#### Scenario: Only original
- **WHEN** a video has `a.srt` and no translation and the user opens Export with subtitles
- **THEN** no Track choice is shown and the export uses `a.srt`

#### Scenario: Only English
- **WHEN** a video has `a.en.srt` and no `a.srt`
- **THEN** the context menu offers Export with subtitles, the dialog shows no Track choice and the export uses `a.en.srt`

### Requirement: Translate subtitles from the gallery
While the translation model is downloaded, the context menu of an MP4 that has a transcript and a known language SHALL offer Translate subtitles…, which opens a target picker preselecting the default target and queues one translation on confirm. A video without a transcript or with an unknown language SHALL NOT offer it.

#### Scenario: Translate to Japanese
- **WHEN** the user right-clicks a Vietnamese video with `a.srt`, chooses Translate subtitles…, picks Japanese and confirms
- **THEN** a translation item for Japanese is queued and later `a.ja.srt` exists

#### Scenario: Default preselected
- **WHEN** the picker opens for an English video
- **THEN** Vietnamese is preselected and English is not offered

#### Scenario: No transcript
- **WHEN** a video has no `a.srt`
- **THEN** the menu has no Translate subtitles…

### Requirement: Tiles show subtitle languages
A video tile in the gallery grid SHALL show one small badge per subtitle track at its bottom-left: the original transcript as the video's language code (or `CC` when the language is unknown), then each translation's code, upper-case, at most three badges followed by `+N` for the rest. A video without subtitles SHALL show none. The viewer thumbnail strip SHALL NOT show them.

#### Scenario: Several subtitles
- **WHEN** a Vietnamese video has `a.srt`, `a.en.srt` and `a.ja.srt`
- **THEN** its tile shows `VI`, `EN` and `JA`

#### Scenario: Unknown language
- **WHEN** a video has `a.srt` and its language is not known
- **THEN** its tile shows `CC`

#### Scenario: Many subtitles
- **WHEN** a video has the original and five translations
- **THEN** its tile shows three badges and `+3`

### Requirement: Remove background from the gallery
While the viewer shows a photo, its header SHALL offer a Remove background button; the context menu of a photo item SHALL offer Remove background; the bulk action bar SHALL offer Remove background, enabled when the selection contains a photo, which queues one background removal per selected photo in visible order and ignores other items. Each SHALL queue the photo's background removal and show the usual "Added to queue" toast. MP4 and other items SHALL NOT offer it. When the result finishes, the gallery SHALL show the new file.

#### Scenario: Viewer button
- **WHEN** the viewer shows a PNG and the user clicks Remove background
- **THEN** the photo's background removal is queued

#### Scenario: Context menu
- **WHEN** the user right-clicks a PNG item and chooses Remove background
- **THEN** its background removal is queued

#### Scenario: Bulk
- **WHEN** two PNGs and an MP4 are selected and the user chooses Remove background
- **THEN** two background removals are queued, in visible order

#### Scenario: Video
- **WHEN** the viewer shows an MP4 item
- **THEN** the header has no Remove background button and its context menu has no Remove background action
