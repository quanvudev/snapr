## ADDED Requirements

### Requirement: Remove background from the gallery
While the viewer shows a photo, its header SHALL offer a Remove background button; the context menu of a photo item SHALL offer Remove background; the bulk action bar SHALL offer Remove background, enabled when the selection contains a photo, which queues one background removal per selected photo in visible order and ignores other items. Each SHALL queue the photo's background removal and show the usual "Added to queue" toast. MP4 and other items SHALL NOT offer it. When the result finishes, the gallery SHALL show the new file.

#### Scenario: Viewer button
- **WHEN** the viewer shows a PNG and the user clicks Remove background
- **THEN** the photo's background removal is queued

#### Scenario: Context menu
- **WHEN** the user right-clicks a PNG item and chooses Remove background
- **THEN** its background removal is queued

#### Scenario: Bulk
- **WHEN** two PNGs and an MP4 are selected and the user chooses Remove background
- **THEN** two background removals are queued, in visible order

#### Scenario: Video
- **WHEN** the viewer shows an MP4 item
- **THEN** the header has no Remove background button and its context menu has no Remove background action
