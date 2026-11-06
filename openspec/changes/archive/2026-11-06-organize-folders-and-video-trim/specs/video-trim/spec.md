## ADDED Requirements

### Requirement: Trim video from the gallery
While the viewer shows an MP4 item, its header SHALL offer a Trim video button, and the context menu of an MP4 item SHALL offer Trim video. Both SHALL open the trim dialog for that video. Photos and other files SHALL NOT offer it.

#### Scenario: Viewer shows a video
- **WHEN** the viewer shows an MP4 item and the user clicks Trim video
- **THEN** the trim dialog opens for that video

#### Scenario: Context menu
- **WHEN** the user right-clicks an MP4 item and chooses Trim video
- **THEN** the trim dialog opens for that video

#### Scenario: Photo
- **WHEN** the viewer shows a PNG item or the user right-clicks it
- **THEN** no Trim video action is offered

### Requirement: Trim dialog picks a range
The trim dialog SHALL show the video file name, a playable preview of the video and a two-thumb range slider over the whole duration in 100 ms steps, starting at the full video. Moving a thumb SHALL seek the preview to that thumb's time. The dialog SHALL show the start, end and resulting duration. Trim SHALL be disabled while the range is the whole video. Cancel or Escape SHALL close the dialog without queuing anything. Trim SHALL close the dialog and queue a trim task for that range.

#### Scenario: Seek on drag
- **WHEN** the user drags the start thumb to 0:12.3
- **THEN** the preview shows the frame at 0:12.3 and the dialog shows the new start and duration

#### Scenario: Whole video
- **WHEN** the dialog opens and the user has not moved a thumb
- **THEN** Trim is disabled

#### Scenario: Queue
- **WHEN** the user sets 0:05 to 0:40 and clicks Trim
- **THEN** the dialog closes and one trim task for that range is added to the background queue

### Requirement: Trim writes a new video
A trim task SHALL write an MP4 containing only the chosen range of the source, starting at time zero, at the source's resolution, with the source's audio for that range when the source has audio. It SHALL be named `<source name>_trimmed.mp4` with a `_N` suffix when the name is taken, in the source video's folder. The source video and its subtitle files SHALL NOT be changed. A range that does not lie inside the video, or that is empty, SHALL be refused and nothing SHALL be queued. A cancelled or failed task SHALL leave no partial file.

#### Scenario: Output
- **WHEN** a trim of `rec.mp4` from 0:05 to 0:40 finishes
- **THEN** `rec_trimmed.mp4` exists next to `rec.mp4`, lasts about 35 seconds, starts with the frame at 0:05 and has the matching audio, and `rec.mp4` is unchanged

#### Scenario: Name taken
- **WHEN** `rec_trimmed.mp4` already exists and another trim of `rec.mp4` finishes
- **THEN** the new file is `rec_trimmed_1.mp4`

#### Scenario: Silent source
- **WHEN** the source has no audio track
- **THEN** the trimmed video has no audio track

#### Scenario: Invalid range
- **WHEN** a trim is requested with a start at or after the end, or an end past the video's duration
- **THEN** the request is refused and nothing is queued

#### Scenario: Cancel
- **WHEN** the user cancels a running trim in the queue sheet
- **THEN** no `_trimmed` file and no partial file remain

### Requirement: Trim keeps the subtitle tracks
When a trim finishes, every subtitle track of the source (the transcript `<name>.srt` and each `<name>.<code>.srt` translation) SHALL be written for the new video under the new video's name. Each track SHALL keep only the cues that overlap the range, clipped to it and shifted so the range start is time zero, renumbered from 1. A track with no cue left SHALL still be written, empty. The new video SHALL get the source's stored language. A source without subtitle tracks SHALL produce no subtitle files.

#### Scenario: Transcript and translation
- **WHEN** `rec.mp4` has `rec.srt` and `rec.vi.srt` and is trimmed from 0:05 to 0:40 into `rec_trimmed.mp4`
- **THEN** `rec_trimmed.srt` and `rec_trimmed.vi.srt` exist, and a cue from 0:03 to 0:07 in the source appears from 0:00 to 0:02

#### Scenario: Name with suffix
- **WHEN** the trimmed video is named `rec_trimmed_1.mp4`
- **THEN** its tracks are named `rec_trimmed_1.srt` and `rec_trimmed_1.<code>.srt`

#### Scenario: Cue outside the range
- **WHEN** a source cue ends before the range start or starts after the range end
- **THEN** it is not in the trimmed track

#### Scenario: Gallery shows the transcript
- **WHEN** a trim of a transcribed video finishes
- **THEN** the gallery shows the new video with its transcript and subtitle language badges

### Requirement: Trim runs in the background queue
A trim SHALL be a background queue task of kind Trim video, shown in the queue sheet with its progress, cancellable and retryable like the other kinds, and the gallery SHALL refresh when it finishes.

#### Scenario: Progress
- **WHEN** a trim is running
- **THEN** the queue sheet shows it as Trim video with a percentage and a cancel button

#### Scenario: Retry
- **WHEN** a trim failed and the user clicks Retry in the queue sheet
- **THEN** the same trim is queued again
