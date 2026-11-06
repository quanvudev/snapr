## ADDED Requirements

### Requirement: AI models shows the upscale model
AI models SHALL show an upscale card for Real-ESRGAN General x4v3 with a one-line description (enlarges photos and screenshots ×2 or ×4 on this computer, keeping text sharp), its size, the licence name (BSD-3-Clause) with a link, its status line (Download / progress with Cancel / error with Retry / downloaded) and a "Used for" row naming Upscale in the gallery. Downloaded models SHALL list it with its size and Delete like the other models, and its total SHALL include it.

#### Scenario: Card
- **WHEN** the user opens AI models and the model is not downloaded
- **THEN** the upscale card shows the description, about 4.6 MB, "BSD-3-Clause" with a link, and Download

#### Scenario: Downloaded list
- **WHEN** the upscale model is downloaded
- **THEN** Downloaded models lists Real-ESRGAN General x4v3 with its size, and the total includes it

#### Scenario: Delete
- **WHEN** the user deletes Real-ESRGAN General x4v3 from Downloaded models and confirms
- **THEN** the card shows it as not downloaded and its disk space is freed

### Requirement: About names the upscale model
Once the upscale model is downloaded, About SHALL name it and its licence with a link, like the other models.

#### Scenario: About
- **WHEN** the upscale model is downloaded and the user opens About
- **THEN** About names Real-ESRGAN General x4v3 and BSD-3-Clause with a link
