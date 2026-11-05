## ADDED Requirements

### Requirement: Automatic redaction of fullscreen and window captures
While automatic redaction is on in the redaction settings and text recognition is available, a fullscreen or window capture SHALL be scanned for the enabled categories and custom patterns before it is saved, and every match SHALL be covered in the default redaction style. Area, Draw and text captures SHALL NOT be redacted automatically.

#### Scenario: Fullscreen capture
- **WHEN** automatic redaction is on and the user takes a fullscreen capture while `alice@example.com` is on screen
- **THEN** the saved PNG, the clipboard image and the preview show a black box over the email

#### Scenario: Window capture
- **WHEN** automatic redaction is on, the default style is Pixelate and the user captures a window showing `192.168.1.20`
- **THEN** the IP is pixelated in the saved PNG

#### Scenario: Off by default
- **WHEN** the redaction settings were never changed and the user takes a fullscreen capture
- **THEN** the capture is saved unredacted

#### Scenario: Area capture
- **WHEN** automatic redaction is on and the user takes an area capture showing an email
- **THEN** the capture is saved unredacted

#### Scenario: No text recognition
- **WHEN** automatic redaction is on but no OCR is available
- **THEN** fullscreen and window captures are saved unredacted without an error

#### Scenario: Settings apply to the next capture
- **WHEN** the user turns the email category off and takes a fullscreen capture
- **THEN** email addresses are not covered

### Requirement: Only the redacted image leaves the capture
When a capture is redacted automatically, the saved file, the clipboard copy and the preview toast SHALL all show the redacted image and the unredacted image SHALL NOT be written anywhere. When recognition fails the capture SHALL be saved unredacted and the tray tooltip SHALL say that redaction failed.

#### Scenario: Nothing unredacted on disk
- **WHEN** automatic redaction covers an email in a fullscreen capture
- **THEN** the output folder holds only the redacted PNG

#### Scenario: Recognition fails
- **WHEN** automatic redaction is on and OCR returns an error
- **THEN** the capture is saved unredacted and the tooltip reads "Snapr: redaction failed: …"

### Requirement: Redaction prefers Windows OCR
With the redaction Text recognition choice on Windows OCR (the default), automatic redaction and Draw's Detect SHALL use Windows OCR whenever a Windows OCR language is available, whichever engine text capture uses, and the ready PaddleOCR model only when Windows OCR is unavailable or fails. Text capture and the gallery SHALL keep using the selected engine.

#### Scenario: PaddleOCR selected
- **WHEN** the OCR engine is PaddleOCR Medium, a Windows OCR language is installed and automatic redaction is on
- **THEN** a fullscreen capture is redacted with Windows OCR and saved without waiting for PaddleOCR

#### Scenario: No Windows OCR language
- **WHEN** no Windows OCR language is installed and a PaddleOCR model is ready
- **THEN** redaction uses PaddleOCR

### Requirement: PaddleOCR can be forced for redaction
With the Text recognition choice on PaddleOCR, automatic redaction and Draw's Detect SHALL use the downloaded PaddleOCR model of the selected size even when text capture uses Windows OCR, and Windows OCR only when PaddleOCR fails.

#### Scenario: PaddleOCR forced for redaction
- **WHEN** text capture uses Windows OCR, the PaddleOCR Medium model is downloaded and Text recognition in the redaction settings is PaddleOCR
- **THEN** a fullscreen capture is redacted with PaddleOCR Medium
