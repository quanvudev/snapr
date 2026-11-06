## MODIFIED Requirements

### Requirement: Dark blurred appearance
The settings window SHALL use a dark theme over the native Windows blurred backdrop, independent of the system light/dark setting. The setup page SHALL use a frosted-glass backdrop that blurs what is behind the window, under a translucent background. The first-run splash SHALL NOT use a backdrop: it has a fully transparent background.

#### Scenario: Light system theme
- **WHEN** Windows is set to light mode
- **THEN** the window still renders the dark theme over the blurred backdrop

#### Scenario: Setup is frosted glass
- **WHEN** the setup page is shown over other windows
- **THEN** those windows show through it blurred

#### Scenario: Splash has no backdrop
- **WHEN** the first-run splash is visible
- **THEN** only the logo and spinner are drawn and the desktop shows through everywhere else
