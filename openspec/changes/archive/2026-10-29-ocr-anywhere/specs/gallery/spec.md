# Spec Delta

## MODIFIED Requirements

### Requirement: Item context menu

Right-clicking an item in the grid, or the content in the viewer, SHALL show the actions Open with default app, Show in Explorer, Pin, Copy image, Extract text and Delete. Pin and Copy image SHALL be shown only for photo items, and Extract text only for photo items while Windows has an OCR language; other files get Open with default app, Show in Explorer and Delete.

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

### Requirement: File actions are limited to the output folder

Thumbnails, viewer content, open, reveal, copy, extract text, pin and delete SHALL only act on files directly inside the resolved output folder. Viewer content SHALL only be served for photo and video files. Pin and extract text SHALL only act on photo files. A request for any other path SHALL be refused without touching the file.

#### Scenario: Path outside the folder

- **WHEN** a request names a path outside the output folder, directly or through `..`
- **THEN** the request is refused and nothing is read, opened or deleted

#### Scenario: Pin outside the folder

- **WHEN** a pin request names a path outside the output folder, directly or through `..`
- **THEN** the request is refused and no pin appears

#### Scenario: Pin a non-photo file

- **WHEN** a pin request names a video or other file inside the output folder
- **THEN** the request is refused and no pin appears

#### Scenario: Extract text outside the folder

- **WHEN** an extract text request names a path outside the output folder or a video inside it
- **THEN** the request is refused and nothing is recognized

#### Scenario: Viewer content for other file

- **WHEN** viewer content is requested for `notes.txt` inside the output folder
- **THEN** the request is refused

#### Scenario: Output folder changed

- **WHEN** the user changes the output folder in General
- **THEN** the gallery lists the new folder and files of the old folder can no longer be read

## ADDED Requirements

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

