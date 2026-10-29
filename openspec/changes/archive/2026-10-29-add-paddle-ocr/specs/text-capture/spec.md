# Spec Delta

## MODIFIED Requirements

### Requirement: Recognition uses the Windows profile languages
Text SHALL be recognized offline with the engine selected by the `ocr-engine` capability. With the Windows engine, or while PaddleOCR is selected but not ready, text SHALL be recognized with the OCR engine built into Windows, using the user's Windows profile languages. With PaddleOCR ready, the languages SHALL be those of the PaddleOCR model, independent of the Windows profile.

#### Scenario: Vietnamese profile
- **WHEN** the engine is Windows, the Windows profile language is Vietnamese with its OCR language installed and the area contains Vietnamese text
- **THEN** the recognized text keeps its Vietnamese diacritics

#### Scenario: Japanese without a Windows language pack
- **WHEN** the engine is PaddleOCR with the Small model ready, no Japanese OCR language is installed in Windows and the area contains Japanese text
- **THEN** the Japanese text is recognized

### Requirement: Text capture is available only with an OCR language
Text capture SHALL be available only when Windows can create an OCR engine for the user's profile languages, or the engine is PaddleOCR and the selected model size is ready. Availability SHALL be checked at startup, each time the settings window opens, when the engine or model size changes, and when a model download finishes or a model is deleted.

#### Scenario: No OCR language
- **WHEN** no OCR language is installed in Windows and no PaddleOCR model is ready for the selected engine
- **THEN** the text hotkey is not registered and pressing it does nothing

#### Scenario: Language installed while running
- **WHEN** the user installs an OCR language while snapr runs and then opens the settings window
- **THEN** text capture becomes available and the text hotkey works without restarting snapr

#### Scenario: PaddleOCR model becomes ready
- **WHEN** no OCR language is installed in Windows, the engine is PaddleOCR and the selected model finishes downloading
- **THEN** text capture becomes available and the text hotkey works without restarting snapr

### Requirement: Text can be extracted from an existing image
Text SHALL also be extractable from a pin or a gallery photo, recognizing the whole full-resolution image with the selected engine. From a pin, the QR-then-OCR, clipboard and notification rules of the text hotkey SHALL apply. From the gallery, the result SHALL be shown in the viewer's text pane as described by the `gallery` capability. No file SHALL be created or changed. The action SHALL be offered only while text capture is available, regardless of the text capture switch.

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
- **WHEN** no OCR language is installed and no PaddleOCR model is ready for the selected engine
- **THEN** neither Copy text on pins nor Extract text on gallery photos is offered

#### Scenario: PaddleOCR without a Windows language
- **WHEN** no OCR language is installed in Windows and the engine is PaddleOCR with a ready model
- **THEN** Extract text on a gallery photo shows the text recognized by PaddleOCR

#### Scenario: Source file unchanged
- **WHEN** the user extracts text from a gallery photo
- **THEN** the file keeps its content and timestamps and no new file appears in the output folder
