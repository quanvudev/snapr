# ai-acceleration Specification

## Purpose

Lets snapr run its local AI models (PaddleOCR, Whisper, HY-MT) on the graphics card when the user opts in, by detecting the hardware, downloading a GPU build of the model runtime on request and falling back to the CPU when the GPU cannot be used.

## Requirements

### Requirement: Hardware acceleration setting
Hardware acceleration SHALL be a setting with the values Auto, CPU, DirectML and CUDA, defaulting to Auto. A config written before this setting existed SHALL load with Auto. Auto SHALL use CUDA when its runtime is downloaded and an NVIDIA card is present, else DirectML when its runtime is downloaded and a usable card is present, else the CPU. Auto SHALL never start a download.

#### Scenario: Older config
- **WHEN** snapr starts with a config that has no hardware acceleration setting and no GPU runtime downloaded
- **THEN** the setting is Auto, AI tasks run on the CPU and nothing is downloaded

#### Scenario: Auto picks a downloaded runtime
- **WHEN** the setting is Auto, an NVIDIA card is present and only the DirectML runtime is downloaded
- **THEN** AI tasks use DirectML

#### Scenario: CPU forced
- **WHEN** the setting is CPU and the CUDA runtime is downloaded
- **THEN** AI tasks run on the CPU

### Requirement: Graphics card detection
snapr SHALL list the graphics cards, ignoring software and virtual adapters and adapters with less than 2 GB of dedicated memory, and SHALL pick the usable card with the most dedicated memory. It SHALL recommend CUDA for an NVIDIA card, DirectML for any other usable card and the CPU when no card is usable. CUDA SHALL be offered only when an NVIDIA card is usable.

#### Scenario: Discrete NVIDIA card with an integrated GPU and a virtual adapter
- **WHEN** the computer has an RTX 5060 Ti (16 GB), a Radeon integrated GPU (512 MB) and an emulator's virtual display adapter
- **THEN** the detected card is the RTX 5060 Ti and CUDA is recommended

#### Scenario: AMD discrete card
- **WHEN** the only usable card is an AMD card with 8 GB
- **THEN** DirectML is recommended and CUDA is not offered

#### Scenario: No usable card
- **WHEN** the computer has only an integrated GPU with 512 MB of dedicated memory
- **THEN** the CPU is recommended and only Auto and CPU can be selected

### Requirement: GPU runtimes are downloaded on request
The DirectML runtime (about 40 MB) and the CUDA runtime (about 1.3 GB on disk, 1.0 GB to download) SHALL be downloaded only when the user selects DirectML or CUDA, or presses Download. Each file SHALL be pinned and verified by SHA-256 and size. A failed or cancelled download SHALL leave no partial files, and only one GPU runtime download SHALL run at a time.

#### Scenario: Select DirectML
- **WHEN** the user selects DirectML and its runtime is not downloaded
- **THEN** the setting is saved and the DirectML download starts with progress and Cancel

#### Scenario: Verification fails
- **WHEN** a downloaded runtime file does not match its pinned SHA-256
- **THEN** the download fails with an error and Retry, no file of that runtime is kept and AI tasks keep using the current runtime

#### Scenario: Cancel
- **WHEN** the user cancels the CUDA download at 40%
- **THEN** the download stops, its partial files are removed and the setting still says CUDA with Download offered

### Requirement: GPU runtimes can be deleted
Each downloaded GPU runtime SHALL be deletable with a confirmation, except the one loaded in this session, which SHALL show that a restart is needed first. Deleting it SHALL free its disk space and, when the setting pointed to it, leave AI tasks on the next best runtime.

#### Scenario: Delete an unused runtime
- **WHEN** the CUDA runtime is downloaded but this session loaded DirectML and the user deletes CUDA and confirms
- **THEN** the CUDA files are removed and the total of Downloaded models no longer includes them

#### Scenario: Delete the loaded runtime
- **WHEN** this session loaded the CUDA runtime and the user tries to delete it
- **THEN** it is not deleted and the page says to switch away from CUDA and restart snapr first

### Requirement: One runtime per session
snapr SHALL load one model runtime per session, at the first AI task, from the setting at that moment. A change made before any AI task ran SHALL apply to the next task. A change made after SHALL apply after snapr restarts, and the page SHALL say a restart is needed.

#### Scenario: Change before any AI task
- **WHEN** snapr has not run any AI task yet, the DirectML runtime is downloaded and the user selects DirectML
- **THEN** the next transcription runs on DirectML without a restart

#### Scenario: Change after an AI task
- **WHEN** a transcription already ran on the CPU in this session and the user selects DirectML
- **THEN** later tasks keep running on the CPU and the page shows that DirectML applies after a restart, with Restart snapr

### Requirement: Restart to apply
Restart snapr SHALL quit and start snapr again with the same hotkeys and settings. It SHALL be refused, with the reason shown, while a recording is running or counting down, or while a background task is waiting or running.

#### Scenario: Restart
- **WHEN** a restart is needed, nothing is recording and the queue is empty, and the user presses Restart snapr
- **THEN** snapr quits, starts again, and the next AI task uses the selected runtime

#### Scenario: Restart refused during a task
- **WHEN** a transcription is running and the user presses Restart snapr
- **THEN** snapr keeps running and the page says to wait for the background tasks to finish

### Requirement: Models used on the GPU
With CUDA loaded, PaddleOCR, Whisper, HY-MT and the background removal model SHALL run on the NVIDIA card. With DirectML loaded, PaddleOCR and the background removal model SHALL run on the detected card; Whisper and HY-MT SHALL stay on the CPU, because DirectML gives wrong Whisper results. Windows OCR is not affected.

#### Scenario: Recognize text with DirectML
- **WHEN** DirectML is loaded, the OCR engine is PaddleOCR and the user captures text
- **THEN** PaddleOCR runs on the detected card and the text matches the CPU result

#### Scenario: Transcribe with DirectML
- **WHEN** DirectML is loaded and a recording is transcribed
- **THEN** the Whisper model runs on the CPU and the transcript is correct

#### Scenario: Translate with DirectML
- **WHEN** DirectML is loaded and a transcript is translated
- **THEN** HY-MT runs on the CPU

#### Scenario: Translate with CUDA
- **WHEN** CUDA is loaded and a transcript is translated
- **THEN** HY-MT runs on the NVIDIA card

#### Scenario: Remove background with DirectML
- **WHEN** DirectML is loaded and the user removes the background of a photo
- **THEN** the model runs on the detected card and the cutout matches the CPU result

### Requirement: Whisper GPU variants
Each Whisper model SHALL have a GPU variant: fp16 for Base and Small, q4f16 for Large v3 Turbo. With CUDA loaded or selected, the selected model's GPU variant SHALL download after its CPU files are ready. Whisper SHALL run on the card only with its GPU variant; until it is ready, transcription SHALL use the CPU files on the CPU. Downloaded models SHALL list each GPU variant with its size and Delete.

#### Scenario: Variant downloads after switching
- **WHEN** Small is downloaded and the user selects CUDA
- **THEN** the Small GPU variant (about 490 MB) downloads with its progress on the Whisper card

#### Scenario: Variant not ready yet
- **WHEN** CUDA is loaded and the Small GPU variant is still downloading when a transcription starts
- **THEN** the transcription runs with the Small CPU files on the CPU and finishes normally

#### Scenario: Same transcript quality
- **WHEN** the same recording is transcribed with the CPU files and with the GPU variant
- **THEN** both transcripts are usable and in the same language; small wording differences are allowed

### Requirement: Fallback to the CPU
When the GPU cannot be used for a model (driver too old, CUDA libraries missing, device removed or out of memory), that task SHALL run on the CPU of the loaded runtime and finish normally. The AI models page SHALL show the GPU error until the next successful GPU task.

#### Scenario: Old NVIDIA driver
- **WHEN** CUDA is loaded and the NVIDIA driver is too old for it
- **THEN** transcriptions still finish on the CPU and the acceleration card shows the CUDA error

#### Scenario: Error clears
- **WHEN** a GPU error is shown and a later task runs on the card successfully
- **THEN** the error is no longer shown
