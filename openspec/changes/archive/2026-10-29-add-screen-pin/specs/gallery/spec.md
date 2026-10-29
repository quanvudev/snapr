# Spec Delta

## MODIFIED Requirements

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
