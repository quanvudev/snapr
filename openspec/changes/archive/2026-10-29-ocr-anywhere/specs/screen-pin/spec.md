# Spec Delta

## MODIFIED Requirements

### Requirement: A hover toolbar copies or closes a pin

Hovering a pin SHALL show Copy, Copy text and Close actions. Copy text SHALL be shown only while Windows has an OCR language and SHALL extract the text of the full-resolution pinned image as described by the `text-capture` capability, keeping the pin open. Copy SHALL put the full-resolution pinned image on the clipboard and keep the pin open, briefly confirming success. Close SHALL remove only that pin. A failed copy SHALL keep the pin and report the error in the tray tooltip. The actions SHALL stay reachable for pins of any image size.

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
- **THEN** all actions are still visible and usable

#### Scenario: Copy text from a pin

- **WHEN** an OCR language is installed and the user chooses Copy text on a pin of a screenshot containing text
- **THEN** the clipboard holds the recognized text, a text-capture notification is shown and the pin remains

#### Scenario: No OCR language

- **WHEN** no OCR language is installed and the user hovers a pin
- **THEN** only Copy and Close are shown
