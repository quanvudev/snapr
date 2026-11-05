## ADDED Requirements

### Requirement: Capture shows redaction settings
The Capture page SHALL show a Redaction card with: an "Automatically redact fullscreen and window captures" switch (off by default), a Black / Pixelate default style choice (Black by default), a Text recognition choice, one switch each for Email, Phone, Card number, IP address and Secrets (all on by default), and the custom pattern list. Every change SHALL save immediately. A config without these settings SHALL load with the defaults.

#### Scenario: Defaults
- **WHEN** the Capture page is shown for a config without redaction settings
- **THEN** automatic redaction is off, the style is Black and all five categories are on, with no custom pattern

#### Scenario: Turn a category off
- **WHEN** the user turns Phone off
- **THEN** the config is saved with phone detection off and the next detection does not cover phone numbers

### Requirement: Redaction text recognition choice
The Redaction card SHALL offer Windows OCR (fast, the default) or PaddleOCR (more accurate) for redaction. PaddleOCR SHALL be disabled, with a link to AI models, while the selected PaddleOCR size is not downloaded.

#### Scenario: PaddleOCR not downloaded
- **WHEN** the selected PaddleOCR size is not downloaded
- **THEN** the PaddleOCR choice is disabled and an AI models link is shown

#### Scenario: Choose PaddleOCR
- **WHEN** the model is downloaded and the user picks PaddleOCR
- **THEN** the config is saved with `engine = "paddle"` and the row says each capture takes longer

### Requirement: Custom pattern list
The Redaction card SHALL list the custom patterns with a remove button each, plus an input and an Add button. A pattern SHALL be added only when it is a valid regular expression for snapr's matcher, not empty, at most 200 characters and not already listed; otherwise an inline message SHALL say why and nothing is saved. At most 20 patterns SHALL be stored; Add is disabled at the limit.

#### Scenario: Add a pattern
- **WHEN** the user types `EMP-\d{6}` and presses Add
- **THEN** the pattern is listed, the input is cleared and the config is saved

#### Scenario: Invalid pattern
- **WHEN** the user types `(abc` and presses Add
- **THEN** an inline error says the pattern is invalid, the input keeps the text and nothing is saved

#### Scenario: Remove a pattern
- **WHEN** the user clicks remove on a listed pattern
- **THEN** it disappears from the list and the config is saved without it

#### Scenario: Limit
- **WHEN** 20 patterns are listed
- **THEN** the Add button is disabled
