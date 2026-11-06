# Spec Delta

## ADDED Requirements

### Requirement: AI models shows hardware acceleration
AI models SHALL start with a Hardware acceleration card showing the detected card and its memory (or "No supported graphics card"), the choice Auto / CPU / DirectML / CUDA (CUDA only with a usable NVIDIA card), the runtime in use this session, one status line (download progress with Cancel, an error with Retry, or the runtime's size with Download), and a GPU error when there is one.

#### Scenario: Card on an NVIDIA computer
- **WHEN** the user opens AI models on a computer with an RTX 5060 Ti and no GPU runtime downloaded
- **THEN** the card shows "NVIDIA GeForce RTX 5060 Ti · 16 GB", Auto selected, "In use: CPU" and a hint recommending CUDA

#### Scenario: Download progress
- **WHEN** the CUDA runtime is downloading
- **THEN** the card shows a progress bar with the percentage and Cancel

#### Scenario: GPU error shown
- **WHEN** the last task could not use the card
- **THEN** the card shows the error under the status line

### Requirement: AI models offers a restart when needed
When the selected runtime differs from the one loaded this session, the acceleration card SHALL say the change applies after a restart and show Restart snapr. A refused restart SHALL show its reason inline.

#### Scenario: Restart needed
- **WHEN** this session loaded the CPU runtime and the user selects downloaded DirectML
- **THEN** the card shows "Applies after restart" and Restart snapr

#### Scenario: Restart refused
- **WHEN** a recording is running and the user presses Restart snapr
- **THEN** the card shows that snapr cannot restart while recording

### Requirement: Downloaded models lists GPU files
Downloaded models SHALL also list each downloaded GPU runtime and each downloaded Whisper GPU variant with its size and Delete, and its total SHALL include them. The CPU runtime SHALL stay unlisted.

#### Scenario: GPU entries listed
- **WHEN** the CUDA runtime and the Small GPU variant are downloaded
- **THEN** Downloaded models lists "CUDA runtime" (about 1.3 GB) and "Whisper Small (GPU)" (about 490 MB), and the total includes both

#### Scenario: Loaded runtime cannot be deleted
- **WHEN** this session loaded the CUDA runtime
- **THEN** its Delete action is disabled with a note that a restart on another runtime is needed first
