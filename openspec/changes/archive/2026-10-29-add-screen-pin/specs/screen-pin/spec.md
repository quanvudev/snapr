# Spec Delta

## Purpose

Turns a saved screenshot into an always-on-top desktop window that stays visible while the user works, without taking focus, and keeps Snapr's own captures and recordings free of pins.

## ADDED Requirements

### Requirement: A screenshot can be pinned from the preview toast

While a screenshot's preview toast is shown, hovering it SHALL reveal a Pin action. Choosing Pin SHALL close the toast and pin the saved image without opening Explorer. Clicking the toast elsewhere SHALL keep the existing open-in-Explorer behavior. A recording toast SHALL NOT offer Pin.

#### Scenario: Pin from the toast

- **WHEN** a screenshot toast is shown, the pointer enters it and the user chooses Pin
- **THEN** the toast closes, the saved image is pinned, and Explorer does not open

#### Scenario: Toast click unchanged

- **WHEN** the user clicks the screenshot toast outside the Pin action
- **THEN** Explorer opens on the saved file

#### Scenario: Recording toast has no pin

- **WHEN** a recording's preview toast is shown
- **THEN** it has no Pin action

### Requirement: A gallery photo can be pinned

The gallery's Pin action SHALL pin a photo item and hide the settings window. A pin request SHALL be limited to photo files directly inside the resolved output folder. If the selected image cannot be decoded, no pin SHALL appear and the error SHALL be reported in the tray tooltip.

#### Scenario: Pin a gallery photo

- **WHEN** the user chooses Pin on a PNG or JPG item in the gallery
- **THEN** the settings window hides and the photo appears as a pin

#### Scenario: Undecodable gallery photo

- **WHEN** the user chooses Pin on a file that cannot be decoded as an image
- **THEN** no pin appears and the tray tooltip reports the error

### Requirement: A pin appears at a moderate size near the cursor

A pin SHALL show the image at 1:1 when its larger side is at most 250 logical px; a larger image SHALL be scaled down, keeping aspect ratio, so its larger side is about 250 logical px. The pin SHALL be centered on the cursor on the cursor's monitor, clamped inside the work area, frameless, always on top of normal windows, skipped from the taskbar, and SHALL NOT take keyboard focus or change the foreground window.

#### Scenario: Small image fits

- **WHEN** a 150x100 image is pinned
- **THEN** it appears at 150x100, centered on the cursor

#### Scenario: Clamped at the edge

- **WHEN** the cursor is near an edge of the monitor
- **THEN** the pin is shifted so the whole pin stays inside the work area

#### Scenario: Large image is capped

- **WHEN** a 1920x1080 image is pinned on a 100% monitor
- **THEN** the image is about 250x141 with its aspect ratio kept

#### Scenario: Focus is not stolen

- **WHEN** a pin appears while another application is active
- **THEN** that application keeps the foreground and keyboard focus

#### Scenario: Secondary monitor

- **WHEN** the cursor is on a secondary monitor
- **THEN** the pin appears on that monitor and is clamped to its work area

### Requirement: A pin preserves image transparency

A pin SHALL render the image with per-pixel transparency, so transparent areas of a padded capture show the desktop beneath instead of an opaque background.

#### Scenario: Pinned padded capture

- **WHEN** a padded screenshot with transparent padding is pinned
- **THEN** the transparent padding shows the desktop beneath the image

### Requirement: A pin is visibly framed

Every pin SHALL draw a visible border around the image, outside the image pixels, so the pin's bounds are distinguishable when the image edges match the desktop behind it. The border SHALL NOT appear in the copied image.

#### Scenario: Image matches the desktop

- **WHEN** a pinned image's edges have the same colour as the desktop behind it
- **THEN** a visible border outlines the pin

#### Scenario: Copy excludes the frame

- **WHEN** the user copies a pin
- **THEN** the clipboard image has no border

### Requirement: A pin can be moved

Dragging a pin with the left mouse button SHALL move it without changing its size or taking focus. A pin SHALL be movable across monitors. A press on a toolbar action SHALL NOT start a drag.

#### Scenario: Drag the image

- **WHEN** the user presses the left button on the pinned image and drags
- **THEN** the pin follows the pointer without changing size

#### Scenario: Cross a monitor boundary

- **WHEN** the user drags a pin onto another monitor
- **THEN** the pin stays at the same physical pixel size and remains on top

#### Scenario: Press a toolbar action

- **WHEN** the user presses the left button on a toolbar action
- **THEN** the pin does not move

#### Scenario: Drag keeps focus

- **WHEN** the user drags a pin while another application is active
- **THEN** the other application remains the foreground window

### Requirement: A hover toolbar copies or closes a pin

Hovering a pin SHALL show Copy and Close actions. Copy SHALL put the full-resolution pinned image on the clipboard and keep the pin open, briefly confirming success. Close SHALL remove only that pin. A failed copy SHALL keep the pin and report the error in the tray tooltip. The actions SHALL stay reachable for pins of any image size.

#### Scenario: Copy the image

- **WHEN** the user chooses Copy on a pin
- **THEN** the clipboard holds the full-resolution image, the pin remains, and the action briefly confirms

#### Scenario: Copy fails

- **WHEN** the clipboard cannot be written
- **THEN** the pin remains and the tray tooltip reports the error

#### Scenario: Close one pin

- **WHEN** the user chooses Close on one of several pins
- **THEN** only that pin disappears and the others stay

#### Scenario: Small image

- **WHEN** the pinned image is smaller than the toolbar
- **THEN** both actions are still visible and usable

### Requirement: Pins coexist and are ephemeral

Snapr SHALL allow multiple pins at the same time. A new pin SHALL appear above existing pins. Pins SHALL be in-memory only: not saved to the output folder, not listed in the gallery, not restored after a restart, and not modifying their source files. All pins SHALL disappear when Snapr quits.

#### Scenario: Multiple pins

- **WHEN** the user pins two images
- **THEN** both are visible and can be moved independently

#### Scenario: New pin on top

- **WHEN** a new pin is created while other pins exist
- **THEN** the new pin appears above them

#### Scenario: Quit clears pins

- **WHEN** Snapr quits
- **THEN** all pins disappear and their source files are unchanged

#### Scenario: Gallery unaffected

- **WHEN** a pin is created or closed
- **THEN** the gallery list does not change

### Requirement: Pins are hidden from Snapr's captures and recordings

Pins SHALL be hidden while a capture overlay is open, while a fullscreen grab is in progress, and while a recording is running, and SHALL reappear when none of those is active. Saved captures and recordings SHALL NOT contain pins. Pins SHALL remain visible to other applications, including screen shares, whenever no Snapr capture or recording is active.

#### Scenario: Capture with pins open

- **WHEN** the user starts an area, window, text, draw or record capture while pins are visible
- **THEN** the pins hide before the desktop is captured, stay hidden while the overlay is open, and return after the capture is saved or cancelled, and the saved image has no pins

#### Scenario: Fullscreen capture

- **WHEN** the user takes a fullscreen capture while pins are visible
- **THEN** the saved image has no pins and the pins return right after the grab

#### Scenario: Recording

- **WHEN** a recording starts and later ends or fails
- **THEN** pins hide for the whole recording and return afterwards, and the video has no pins

#### Scenario: Screenshot during a recording

- **WHEN** the user takes a screenshot while a recording is running
- **THEN** the pins stay hidden after the screenshot interaction ends and return only when the recording ends

#### Scenario: Capture or recording fails to start

- **WHEN** a capture or recording fails to start
- **THEN** the pins return

#### Scenario: Pin created while hidden

- **WHEN** a pin is created while pins are hidden
- **THEN** it stays hidden until the last hiding condition ends

#### Scenario: Other applications

- **WHEN** another application captures or shares the screen and no Snapr capture or recording is active
- **THEN** the pins are visible
