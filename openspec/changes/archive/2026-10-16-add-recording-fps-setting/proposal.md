# Proposal

## Why

Recordings are fixed at about 30 fps. That looks choppy for fast UI animation, scrolling or games. Users want 60 fps when smoothness matters, and 30 fps when smaller files matter more, so the rate should be a setting rather than a new fixed value.

## What Changes

- New **Recording** group in the settings sidebar, between Preview and Hotkeys. It has one **Frame rate** row with a choice of 30 or 60 fps.
- New config field `record_fps`. It defaults to 30, so behavior and file size are unchanged for existing users. Values other than 30 or 60 are treated as 30.
- A recording uses the frame rate configured when it starts. Changing the setting during a recording affects only the next recording.
- The setup page is unchanged.

Out of scope: other frame rates, bitrate or quality settings, audio.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `screen-recording`: The recording frame rate is the configured 30 or 60 fps instead of a fixed about-30 fps.
- `settings-window`: The sidebar adds a Recording group with the frame-rate setting.

## Impact

- Backend:
  - `config.rs`: the `record_fps` field, its default and clamping;
  - `recorder.rs`: the throttle interval and encoder frame rate come from the fps;
  - `lib.rs`: passes the configured fps to `Recorder::start`.
- Frontend:
  - `lib/api.ts`: the `Config` type;
  - `lib/i18n.ts`: labels in `en` and `vi`;
  - `hooks/use-settings.ts`: `SectionId`;
  - a new `components/settings/recording/` folder with the group and its row;
  - `components/settings/sections.ts`: an entry for the new group.
- Config: new field, backward compatible through serde defaults. No new dependencies.
