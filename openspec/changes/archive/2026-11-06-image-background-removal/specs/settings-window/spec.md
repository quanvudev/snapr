## ADDED Requirements

### Requirement: AI models shows the background removal model
AI models SHALL show a background removal card for BEN2 Base with a one-line description (removes the background of photos of people, pets and objects on this computer, not suited to UI screenshots), its size, the licence name (MIT) with a link, its status line (Download / progress with Cancel / error with Retry / downloaded) and a "Used for" row naming Remove background in the gallery. Downloaded models SHALL list it with its size and Delete like the other models, and its total SHALL include it.

#### Scenario: Card
- **WHEN** the user opens AI models and the model is not downloaded
- **THEN** the background removal card shows the description, about 223 MB, "MIT" with a link, and Download

#### Scenario: Downloaded list
- **WHEN** the background removal model is downloaded
- **THEN** Downloaded models lists BEN2 Base with its size, and the total includes it

#### Scenario: Delete
- **WHEN** the user deletes BEN2 Base from Downloaded models and confirms
- **THEN** the card shows it as not downloaded and its disk space is freed
