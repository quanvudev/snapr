## ADDED Requirements

### Requirement: Background removal writes a transparent copy
Removing the background of a gallery photo SHALL run as a background task (kind "Remove background") on the computer and SHALL write a PNG of the same pixel size as the photo, where the main subject keeps its pixels and the background is transparent, with soft edges kept as partial transparency. The file SHALL be named `<stem>_cutout.png`, with a `_N` suffix when that name is taken, and saved in the `Images` subfolder of the output folder whatever folder the photo is in. The original photo SHALL NOT be changed. While running, the task SHALL report progress and SHALL be cancellable, and a cancelled or failed task SHALL leave no partial file.

#### Scenario: Product screenshot
- **WHEN** the model is downloaded and the user removes the background of `shoe.png`
- **THEN** `Images/shoe_cutout.png` appears in the gallery with the shoe on a transparent background and the same size as `shoe.png`

#### Scenario: Name taken
- **WHEN** `shoe_cutout.png` already exists and the user removes the background of `shoe.png` again
- **THEN** the new file is `shoe_cutout_1.png` and the old one is unchanged

#### Scenario: Cancel
- **WHEN** the user cancels a running background removal
- **THEN** no new file appears in the output folder

### Requirement: Background removal needs the model
A background removal SHALL be refused, with nothing queued, while the background removal model is not downloaded, and the refusal SHALL offer to open AI models. The model SHALL never be downloaded without the user asking.

#### Scenario: Model missing
- **WHEN** the model is not downloaded and the user chooses Remove background
- **THEN** an error says the model is needed, with an action that opens AI models, and no task is queued

### Requirement: Background removal model download
The background removal model (BEN2 Base, MIT licence, about 223 MB) SHALL be downloaded from a fixed, pinned source and verified by size and checksum before use, with progress, Cancel, and Retry after a failure. A failed or cancelled download SHALL leave no model files. The model SHALL be stored in the app's local data folder and SHALL be deletable from Downloaded models, except while a background removal is running.

#### Scenario: Download
- **WHEN** the user clicks Download on the background removal card
- **THEN** the progress is shown and, when finished, the card shows the model as downloaded

#### Scenario: Corrupted download
- **WHEN** the downloaded file does not match its checksum
- **THEN** the card shows an error with Retry and no model file is kept

#### Scenario: Delete while running
- **WHEN** a background removal is running and the user deletes the model
- **THEN** the delete is refused with a message
