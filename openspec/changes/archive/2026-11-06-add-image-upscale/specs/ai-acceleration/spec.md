## MODIFIED Requirements

### Requirement: Models used on the GPU
With CUDA loaded, PaddleOCR, Whisper, HY-MT, the background removal model and the upscale model SHALL run on the NVIDIA card. With DirectML loaded, PaddleOCR, the background removal model and the upscale model SHALL run on the detected card; Whisper and HY-MT SHALL stay on the CPU, because DirectML gives wrong Whisper results. Windows OCR is not affected.

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

#### Scenario: Upscale with DirectML
- **WHEN** DirectML is loaded and the user upscales a photo
- **THEN** the model runs on the detected card and the result matches the CPU result

#### Scenario: Card fails during an upscale
- **WHEN** the card fails while an upscale runs
- **THEN** the upscale runs again on the processor, finishes, and AI models shows why the card was not used
