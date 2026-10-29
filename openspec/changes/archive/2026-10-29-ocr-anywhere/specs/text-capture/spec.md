# Spec Delta

## ADDED Requirements

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
