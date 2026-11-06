## Purpose

Enlarge gallery photos by two or four times on this computer with a downloadable super-resolution model, saving a sharper PNG copy through the background task queue.

## ADDED Requirements

### Requirement: Upscaling writes an enlarged copy
Upscaling a gallery photo SHALL run as a background task (kind "Upscale") on the computer, at the scale the user chose, ×2 or ×4, and SHALL write a PNG exactly that many times wider and taller than the photo. The file SHALL be named `<stem>_x2.png` or `<stem>_x4.png`, with a `_N` suffix when the name is taken, and saved in the `Images` subfolder whatever folder the photo is in. The original SHALL NOT be changed.

#### Scenario: Upscale a crop twice
- **WHEN** the model is downloaded and the user upscales the 400×260 `dialog.png` by ×2
- **THEN** `Images/dialog_x2.png` appears in the gallery at 800×520 and `dialog.png` is unchanged

#### Scenario: Four times
- **WHEN** the user upscales `dialog.png` by ×4
- **THEN** `Images/dialog_x4.png` is 1600×1040

#### Scenario: Name taken
- **WHEN** `dialog_x2.png` already exists and the user upscales `dialog.png` by ×2 again
- **THEN** the new file is `dialog_x2_1.png` and the old one is unchanged

### Requirement: Upscale tasks report progress and can be cancelled
A running upscale SHALL report its progress in the background tasks list and SHALL be cancellable; a cancelled or failed upscale SHALL leave no partial file.

#### Scenario: Cancel
- **WHEN** the user cancels a running upscale
- **THEN** no new file appears in the output folder

### Requirement: Upscaled text stays readable and correct
The enlarged image SHALL be sharper than a classic resize of the same photo, and text in it SHALL keep the same characters as the original, including Vietnamese diacritics and text as small as 9 px; the model SHALL NOT turn one letter or accent into another.

#### Scenario: Small text
- **WHEN** the user upscales a screenshot crop containing `user@example.com` in 9 px Segoe UI by ×4
- **THEN** the result reads `user@example.com`, with sharper edges than the photo enlarged by Lanczos

#### Scenario: Vietnamese diacritics
- **WHEN** the user upscales a crop containing "Lọc tiếng ồn"
- **THEN** every accent in the result is the same as in the original

### Requirement: Transparency is kept
A photo with transparent pixels SHALL keep them: the result's alpha SHALL be the photo's alpha enlarged to the result size, so a transparent area stays transparent and soft edges stay soft.

#### Scenario: Padded screenshot
- **WHEN** the user upscales a screenshot saved with transparent padding
- **THEN** the padding is transparent in the result too

### Requirement: Results have a size limit
An upscale whose result would be longer than 8192 px on its longer side SHALL be refused before anything is queued, with a message saying the photo is too large for that scale.

#### Scenario: Large photo at ×4
- **WHEN** the user upscales a 2560×1440 screenshot by ×4
- **THEN** an error says the photo is too large to upscale ×4 and nothing is queued

#### Scenario: Same photo at ×2
- **WHEN** the user upscales the same 2560×1440 screenshot by ×2
- **THEN** the upscale is queued and the result is 5120×2880

### Requirement: Upscaling needs the model
An upscale SHALL be refused, with nothing queued, while the upscale model is not downloaded, and the refusal SHALL offer to open AI models. The model SHALL never be downloaded without the user asking.

#### Scenario: Model missing
- **WHEN** the model is not downloaded and the user chooses Upscale ×2
- **THEN** an error says the model is needed, with an action that opens AI models, and no task is queued

### Requirement: Upscale model download
The upscale model (Real-ESRGAN General x4v3, BSD-3-Clause, about 4.6 MB) SHALL be downloaded from a fixed, pinned source and verified by size and checksum before use, with progress, Cancel, and Retry after a failure, together with the shared ONNX Runtime when it is missing. A failed or cancelled download SHALL leave no model files. The model SHALL be stored in the app's local data folder and SHALL be deletable, except while an upscale is running or the model is downloading.

#### Scenario: Download
- **WHEN** the user clicks Download on the upscale card
- **THEN** the progress is shown and, when finished, the card shows the model as downloaded

#### Scenario: Corrupted download
- **WHEN** the downloaded file does not match its checksum
- **THEN** the card shows an error with Retry and no model file is kept

#### Scenario: Delete while running
- **WHEN** an upscale is running and the user deletes the model
- **THEN** the delete is refused with a message
