## MODIFIED Requirements

### Requirement: Captures are saved in order with unique names
Captures SHALL be saved one at a time in the order they were taken, into the `Images` subfolder of the configured folder, as PNG files with the default compression, named by local timestamp with a `_N` suffix when the name is taken in that subfolder.

#### Scenario: Saved in Images
- **WHEN** the user takes a capture
- **THEN** the PNG is in `<output>\Images` and nothing new appears directly in the output folder

#### Scenario: Two captures in the same second
- **WHEN** the user takes two captures within the same second
- **THEN** both files are kept, the second named with a `_1` suffix

#### Scenario: Clipboard holds the latest capture
- **WHEN** copy to clipboard is enabled and several captures are saved one after another
- **THEN** after the last save finishes, the clipboard holds the most recent capture
