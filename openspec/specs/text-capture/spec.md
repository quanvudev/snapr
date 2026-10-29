# text-capture Specification

## Purpose

Lets the user select an area of the screen and get the text in it on the clipboard, using the OCR engine built into Windows, without saving a screenshot.

## Requirements

### Requirement: Text hotkey copies the text of a selected area
Pressing the text hotkey SHALL open the area picker. Selecting an area SHALL close the picker, recognize the text in that area and copy it to the clipboard as plain text, one visual row per line, with the text pieces of a row joined left to right by a space. No image file SHALL be saved and no image preview SHALL be shown. Cancelling the picker SHALL do nothing.

#### Scenario: Extract text from an area
- **WHEN** the user presses the text hotkey and selects an area containing two lines of text
- **THEN** the clipboard holds those two lines as plain text and no new file appears in the output folder

#### Scenario: Spaced items on one row
- **WHEN** the selected area is a single row of widely spaced items such as a navigation bar
- **THEN** the clipboard holds one line with the items in left-to-right order separated by spaces

#### Scenario: Cancel the picker
- **WHEN** the user presses the text hotkey and then presses Escape or right-clicks
- **THEN** the picker closes, the clipboard is unchanged and no notification is shown

#### Scenario: Clipboard setting does not apply
- **WHEN** Copy to clipboard is off and the user extracts text from an area
- **THEN** the text is still copied to the clipboard

### Requirement: Recognition uses the Windows profile languages
Text SHALL be recognized with the OCR engine built into Windows, using the user's Windows profile languages, offline.

#### Scenario: Vietnamese profile
- **WHEN** the Windows profile language is Vietnamese with its OCR language installed and the area contains Vietnamese text
- **THEN** the recognized text keeps its Vietnamese diacritics

### Requirement: Text capture reports its result with a notification
After recognition, a Windows notification SHALL report the result in the app language: the number of copied lines on success, "No text found" when nothing was recognized, or the error when recognition fails. When nothing is recognized or recognition fails, the clipboard SHALL NOT change.

#### Scenario: Success
- **WHEN** three lines of text are recognized
- **THEN** a notification says three lines were copied

#### Scenario: No text
- **WHEN** the selected area contains no recognizable text
- **THEN** a notification says no text was found and the clipboard keeps its previous content

#### Scenario: Language follows the app
- **WHEN** the app language is Vietnamese and text is copied
- **THEN** the notification is in Vietnamese

### Requirement: Text capture is available only with an OCR language
Text capture SHALL be available only when Windows can create an OCR engine for the user's profile languages. Availability SHALL be checked at startup and each time the settings window opens.

#### Scenario: No OCR language
- **WHEN** no OCR language is installed in Windows
- **THEN** the text hotkey is not registered and pressing it does nothing

#### Scenario: Language installed while running
- **WHEN** the user installs an OCR language while snapr runs and then opens the settings window
- **THEN** text capture becomes available and the text hotkey works without restarting snapr

### Requirement: Text capture can be turned off
Text capture SHALL be controlled by an enable setting that defaults to on. The text hotkey SHALL be registered only while the setting is on and text capture is available. While it is not registered, the text hotkey SHALL be ignored by hotkey validation.

#### Scenario: Turned off
- **WHEN** the user turns text capture off
- **THEN** the text hotkey stops working immediately and its key can be used by another snapr hotkey without a duplicate error

#### Scenario: Turned back on
- **WHEN** the user turns text capture on and OCR is available
- **THEN** the text hotkey works immediately

### Requirement: Text capture works during a recording
While a recording is running, the text hotkey SHALL keep working as usual.

#### Scenario: Extract text while recording
- **WHEN** a recording is running and the user extracts text from an area
- **THEN** the text is copied and the recording continues

### Requirement: QR codes are read before text
Every text recognition SHALL first look for QR codes in the image. When one or more QR codes decode, their contents SHALL be copied to the clipboard as plain text, one code per line in top-to-bottom order, instead of the recognized text. When no QR code decodes, recognition SHALL continue with the OCR engine as before. A decoded link SHALL NOT be opened.

#### Scenario: Area with a QR code
- **WHEN** the user extracts text from an area containing one QR code that encodes `https://example.com`
- **THEN** the clipboard holds `https://example.com` and a notification says a QR code was copied

#### Scenario: Two QR codes
- **WHEN** the selected image contains two QR codes stacked vertically
- **THEN** the clipboard holds both contents, the upper one on the first line

#### Scenario: QR code and text
- **WHEN** the selected area contains a QR code and a caption under it
- **THEN** the clipboard holds only the QR content

#### Scenario: Unreadable QR code
- **WHEN** the area contains a QR code that cannot be decoded and some text
- **THEN** the clipboard holds the recognized text as if no QR code were present

#### Scenario: Link not opened
- **WHEN** a decoded QR code is a URL
- **THEN** no browser or app is opened

### Requirement: Text can be extracted from an existing image
Text SHALL also be extractable from a pin or a gallery photo, recognizing the whole full-resolution image. From a pin, the QR-then-OCR, clipboard and notification rules of the text hotkey SHALL apply. From the gallery, the result SHALL be shown in the viewer's text pane as described by the `gallery` capability. No file SHALL be created or changed. The action SHALL be offered only while Windows can create an OCR engine, regardless of the text capture switch.

#### Scenario: Same result as the hotkey
- **WHEN** the user copies text from a pin of a screenshot containing two lines of text
- **THEN** the clipboard holds those two lines and a notification says two lines were copied

#### Scenario: No text in a pin
- **WHEN** the user copies text from a pin with no recognizable text
- **THEN** a notification says no text was found and the clipboard keeps its previous content

#### Scenario: Text capture switched off
- **WHEN** text capture is turned off and an OCR language is installed
- **THEN** Copy text is still offered on pins and Extract text on gallery photos

#### Scenario: No OCR language
- **WHEN** no OCR language is installed
- **THEN** neither Copy text on pins nor Extract text on gallery photos is offered

#### Scenario: Source file unchanged
- **WHEN** the user extracts text from a gallery photo
- **THEN** the file keeps its content and timestamps and no new file appears in the output folder
