# Spec Delta

## Purpose

Lets the user choose the engine that recognizes text (the OCR built into Windows or the offline PaddleOCR v6 models) and manages the PaddleOCR models on disk: download, verification, storage and deletion.

## ADDED Requirements

### Requirement: OCR engine setting
The OCR engine SHALL be a setting with the values Windows and PaddleOCR, defaulting to Windows. A config written before this setting existed SHALL load with Windows. A change SHALL take effect for the next recognition without restarting snapr.

#### Scenario: Default engine
- **WHEN** snapr starts with a config that has no OCR engine setting
- **THEN** text is recognized with Windows OCR and nothing is downloaded

#### Scenario: Switch back to Windows
- **WHEN** the engine is PaddleOCR with a ready model and the user selects Windows
- **THEN** the next recognition uses Windows OCR and the downloaded models stay on disk

### Requirement: PaddleOCR replaces Windows OCR when ready
While the engine is PaddleOCR and the selected model size is ready, every text recognition (text hotkey, pin Copy text, gallery Extract text) SHALL use PaddleOCR, offline, and SHALL NOT use Windows OCR. Results SHALL follow the same row, clipboard, QR and notification rules as Windows OCR.

#### Scenario: Recognize with PaddleOCR
- **WHEN** the engine is PaddleOCR, the Small model is ready and the user extracts text from an area with two lines of English text
- **THEN** the clipboard holds those two lines and a notification says two lines were copied

#### Scenario: Works without a Windows OCR language
- **WHEN** Windows has no OCR language installed and the engine is PaddleOCR with a ready model
- **THEN** the text hotkey, pin Copy text and gallery Extract text are available and recognize text

#### Scenario: Offline
- **WHEN** the selected model is ready and the network is disconnected
- **THEN** text recognition works

### Requirement: Fallback to Windows OCR until the model is ready
While the engine is PaddleOCR but the selected size is not ready (downloading, failed, cancelled or deleted), recognition SHALL use Windows OCR when it is available, and text capture SHALL be unavailable when it is not.

#### Scenario: During download
- **WHEN** the user selects PaddleOCR and extracts text while the model is still downloading
- **THEN** the text is recognized with Windows OCR

#### Scenario: No engine ready
- **WHEN** Windows has no OCR language and the selected PaddleOCR size is not downloaded
- **THEN** the text hotkey is not registered and Copy text / Extract text are not offered

#### Scenario: Ready without restart
- **WHEN** Windows has no OCR language and the selected model finishes downloading
- **THEN** text capture becomes available and the text hotkey works without restarting snapr

### Requirement: Three model sizes
PaddleOCR SHALL offer the sizes Tiny, Small and Medium, defaulting to Small. Each size SHALL use its own PP-OCRv6 detection and recognition models. Tiny SHALL be described as fastest and smallest, without Japanese; Small as balanced; Medium as most accurate, slowest and largest.

#### Scenario: Change size
- **WHEN** the engine is PaddleOCR and the user changes the size from Small to Medium
- **THEN** recognition uses Medium once it is ready and Small until then

#### Scenario: Older config
- **WHEN** a config with no model size loads
- **THEN** the size is Small

### Requirement: Model download on selection
Selecting PaddleOCR, or a size that is not on disk while PaddleOCR is selected, SHALL start downloading that size in the background, together with the ONNX Runtime library if it is missing. Only one download SHALL run at a time. Progress SHALL be reported to the settings window. Nothing SHALL be downloaded while the engine is Windows.

#### Scenario: First activation
- **WHEN** the user selects PaddleOCR for the first time
- **THEN** the Small model and the ONNX Runtime library download in the background and the settings window shows progress

#### Scenario: Size already on disk
- **WHEN** the user selects a size that was downloaded before
- **THEN** nothing is downloaded and the size is used immediately

#### Scenario: Cancel
- **WHEN** the user cancels a running download
- **THEN** the download stops, its partial files are removed and the size is shown as not downloaded

#### Scenario: Window closed during download
- **WHEN** the settings window is hidden while a download runs
- **THEN** the download continues and the size becomes ready when it finishes

### Requirement: Downloaded files are verified
Every downloaded file SHALL be checked against a SHA-256 hash pinned in snapr before it is used. A file that fails the check, or an interrupted download, SHALL NOT be used and SHALL leave no file that counts as ready. A failure SHALL be shown in the settings window with a retry action.

#### Scenario: Hash mismatch
- **WHEN** a downloaded model file does not match its pinned hash
- **THEN** the file is discarded, the size stays not downloaded and the settings window shows the error with Retry

#### Scenario: Interrupted by quit
- **WHEN** snapr quits during a download and is started again
- **THEN** the size is shown as not downloaded and recognition does not use partial files

### Requirement: Models are stored locally
Models and the ONNX Runtime library SHALL be stored in snapr's local (non-roaming) app data folder, separate from the config and the output folder.

#### Scenario: Location
- **WHEN** a model finishes downloading
- **THEN** its files are under `%LOCALAPPDATA%\com.quanvudev.snapr\paddle\` and nothing new appears in `%APPDATA%` or the output folder

### Requirement: Downloaded sizes are kept and deleted manually
Downloaded sizes SHALL be kept when the user changes size or engine. The user SHALL be able to delete each downloaded size, after a confirmation. Deleting the size in use SHALL make recognition fall back to Windows OCR. The ONNX Runtime library SHALL be deleted when no size remains.

#### Scenario: Keep on size change
- **WHEN** Small is downloaded and the user switches to Tiny
- **THEN** Small stays on disk and switching back to Small does not download again

#### Scenario: Delete a size in use
- **WHEN** the engine is PaddleOCR on Small and the user confirms deleting Small
- **THEN** Small's files are removed, the engine stays PaddleOCR and recognition uses Windows OCR until a size is ready again

#### Scenario: Delete the last size
- **WHEN** the user deletes the only downloaded size
- **THEN** the ONNX Runtime library is removed too and the paddle folder holds no model files
