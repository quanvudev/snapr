## MODIFIED Requirements

### Requirement: Remove background from the gallery
For a photo, Remove background SHALL be offered inside the AI tools group: the AI tools submenu of the context menu, the AI tools button of the viewer header and the AI tools button of the bulk action bar. The bulk action bar's AI tools button SHALL be enabled when the selection contains a photo, and Remove background there SHALL queue one background removal per selected photo in visible order and ignore other items. Each SHALL queue the photo's background removal and show the usual "Added to queue" toast. MP4 and other items SHALL NOT offer it. When the result finishes, the gallery SHALL show the new file.

#### Scenario: Viewer button
- **WHEN** the viewer shows a PNG and the user opens AI tools and clicks Remove background
- **THEN** the photo's background removal is queued

#### Scenario: Context menu
- **WHEN** the user right-clicks a PNG item and chooses AI tools, then Remove background
- **THEN** its background removal is queued

#### Scenario: Bulk
- **WHEN** two PNGs and an MP4 are selected and the user chooses AI tools, then Remove background
- **THEN** two background removals are queued, in visible order

#### Scenario: Video
- **WHEN** the viewer shows an MP4 item
- **THEN** the header has no AI tools button and its context menu has no AI tools submenu

## ADDED Requirements

### Requirement: AI tools group
Photo actions that run a local AI model SHALL be grouped as AI tools: one AI tools submenu in a photo's context menu, one AI tools button in the viewer header and one in the bulk action bar, each listing Remove background, then a separator, then Upscale ×2 and Upscale ×4. MP4 and other items SHALL NOT offer AI tools.

#### Scenario: One entry point per place
- **WHEN** the viewer shows a PNG
- **THEN** the header has a single AI tools button whose menu lists Remove background, Upscale ×2 and Upscale ×4

#### Scenario: Context menu group
- **WHEN** the user right-clicks a PNG item and opens AI tools
- **THEN** it lists Remove background, Upscale ×2 and Upscale ×4

### Requirement: Upscale from the gallery
Choosing Upscale ×2 or Upscale ×4 in a photo's AI tools (context menu or viewer) SHALL queue that upscale at once with the usual "Added to queue" toast; a refusal (model missing, too large) SHALL show its error toast and queue nothing. When a result finishes, the gallery SHALL show the new file.

#### Scenario: Context menu
- **WHEN** the user right-clicks a PNG item and chooses AI tools, then Upscale ×2
- **THEN** its ×2 upscale is queued

#### Scenario: Viewer button
- **WHEN** the viewer shows a PNG and the user opens AI tools and chooses Upscale ×4
- **THEN** the photo's ×4 upscale is queued

### Requirement: Bulk upscale from the gallery
Upscale ×2 and Upscale ×4 in the bulk action bar's AI tools SHALL queue one upscale per selected photo in visible order and ignore other items; a photo that is refused SHALL show its error toast without stopping the others.

#### Scenario: Bulk
- **WHEN** two PNGs and an MP4 are selected and the user chooses AI tools, then Upscale ×2
- **THEN** two ×2 upscales are queued, in visible order

#### Scenario: Bulk with one photo too large
- **WHEN** a small and a 2560×1440 PNG are selected and the user chooses AI tools, then Upscale ×4
- **THEN** the small one is queued and an error toast says the large one is too large
