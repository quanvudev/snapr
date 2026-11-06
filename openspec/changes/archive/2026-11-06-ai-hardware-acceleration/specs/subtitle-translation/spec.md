# Spec Delta

## ADDED Requirements

### Requirement: Cues are translated in batches
Translation SHALL process up to 8 cues per model run on every runtime, each cue with its own request. A cue's translation SHALL NOT contain text from another cue and SHALL match translating that cue alone, apart from rare wording differences caused by numeric precision. Progress SHALL count finished cues, and a cancel SHALL stop the translation within one generated token of the running batch.

#### Scenario: Same result as one by one
- **WHEN** a 20-cue transcript is translated to English in batches and the same cues are translated one at a time
- **THEN** at least 19 of the 20 cues have the same translation in both runs and no cue contains text from another cue

#### Scenario: Cues of different lengths
- **WHEN** a batch holds a one-word cue and a 40-word cue
- **THEN** the one-word cue's translation is short and complete, and the 40-word cue's translation is complete

#### Scenario: Progress per batch
- **WHEN** a 40-cue transcript is translated and the first batch of 8 cues finishes
- **THEN** the queue shows it at 20%

#### Scenario: Cancel during a batch
- **WHEN** the user cancels while a batch is generating
- **THEN** the translation stops promptly, no partial `.srt` is left and an existing translation file is unchanged

#### Scenario: Faster than one by one
- **WHEN** a 100-cue transcript is translated on the CPU
- **THEN** it finishes faster than translating the cues one at a time on the same computer
