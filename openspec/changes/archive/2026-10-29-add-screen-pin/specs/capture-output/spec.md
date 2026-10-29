# Spec Delta

## MODIFIED Requirements

### Requirement: Preview toast appears after the file is saved

The preview toast SHALL appear only after the capture's PNG file has been written, and only if the preview duration is greater than 0. A screenshot toast SHALL offer a Pin action on hover; choosing it SHALL pin the saved image and close the toast. Clicking the toast outside the Pin action SHALL open the saved file in Explorer.

#### Scenario: Toast after save

- **WHEN** a capture's file has been written and the preview duration is greater than 0
- **THEN** the toast showing that capture appears in the bottom-right of the work area

#### Scenario: Toast does not wait for the clipboard

- **WHEN** copy to clipboard is enabled and a capture's file has been written
- **THEN** the toast appears without waiting for the clipboard copy to finish

#### Scenario: Preview disabled

- **WHEN** a capture's file has been written and the preview duration is 0
- **THEN** no toast appears

#### Scenario: Toast click

- **WHEN** the user clicks the toast outside the Pin action
- **THEN** Explorer opens on the saved file, which exists

#### Scenario: Toast pin

- **WHEN** a screenshot toast is hovered and the user chooses Pin
- **THEN** the toast closes and the saved image is pinned, and Explorer does not open

#### Scenario: Overlay open when save finishes

- **WHEN** a capture's file has been written while a capture overlay is open
- **THEN** no toast is shown for that capture
