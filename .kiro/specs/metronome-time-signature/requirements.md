# Requirements Document

## Introduction

The metronome currently supports only BPM control with a single undifferentiated click sound. This feature adds time signature configuration (numerator and denominator) and an accent sound on the first beat of each bar, allowing musicians to hear bar boundaries and practice in compound and simple meters such as 4/4, 3/4, 6/8, and others.

## Glossary

- **Metronome**: The component responsible for scheduling and playing rhythmic click sounds at a configured tempo.
- **Time_Signature**: A pair of values — beats per bar (numerator) and beat unit (denominator) — that define the rhythmic grouping of the metronome.
- **Numerator**: The number of beats in one bar (e.g., 3 in 3/4). Valid range: 1–16.
- **Denominator**: The note value that represents one beat (e.g., 4 for a quarter note, 8 for an eighth note). Valid values: 1, 2, 4, 8, 16.
- **Beat**: A single rhythmic pulse scheduled by the Metronome.
- **Bar**: A group of beats equal in count to the Numerator.
- **Accent_Click**: A distinct audio click played on beat 1 of each bar to mark the bar boundary.
- **Regular_Click**: The standard audio click played on beats 2 through Numerator of each bar.
- **Beat_Index**: A zero-based counter tracking the current beat position within the active bar (0 = first beat).
- **AppState**: The top-level application state managed by the Yew reducer.
- **AudioEngine**: The WebAudio-based engine responsible for scheduling sounds.
- **NavBar**: The top navigation bar component that hosts metronome controls.

## Requirements

### Requirement 1: Time Signature Data Model

**User Story:** As a musician, I want the app to store a time signature alongside BPM, so that the metronome can use it to group beats into bars.

#### Acceptance Criteria

1. THE AppState SHALL contain a `time_signature` field of type `TimeSignature` with a default value of 4/4 (numerator = 4, denominator = 4).
2. THE TimeSignature SHALL enforce a numerator in the range 1 to 16 inclusive.
3. THE TimeSignature SHALL enforce a denominator that is one of the values: 1, 2, 4, 8, or 16.
4. IF a `SetTimeSignature` action is dispatched with a numerator outside the range 1–16, THEN THE AppState SHALL reject the update and retain the previous time signature.
5. IF a `SetTimeSignature` action is dispatched with a denominator not in {1, 2, 4, 8, 16}, THEN THE AppState SHALL reject the update and retain the previous time signature.
6. THE AppState SHALL persist the `time_signature` field to localStorage alongside BPM, theme, and mute state.

---

### Requirement 2: Time Signature UI Controls

**User Story:** As a musician, I want to configure the time signature from the navigation bar, so that I can set the meter without leaving the main view.

#### Acceptance Criteria

1. WHILE the Metronome is displayed in the NavBar, THE NavBar SHALL display a numerator selector and a denominator selector adjacent to the BPM control.
2. THE NavBar SHALL offer numerator options covering the integers 1 through 16.
3. THE NavBar SHALL offer denominator options covering the values 1, 2, 4, 8, and 16.
4. WHEN the user selects a new numerator, THE NavBar SHALL emit a `SetTimeSignature` action with the updated numerator and the current denominator.
5. WHEN the user selects a new denominator, THE NavBar SHALL emit a `SetTimeSignature` action with the current numerator and the updated denominator.
6. THE NavBar SHALL display the current time signature in the conventional "numerator/denominator" format (e.g., "4/4") as a label adjacent to the selectors.

---

### Requirement 3: Beat Counting and Bar Tracking

**User Story:** As a musician, I want the metronome to count beats within each bar, so that the accent always falls on beat 1.

#### Acceptance Criteria

1. THE Metronome SHALL maintain a `beat_index` counter that increments by 1 on each scheduled beat.
2. WHEN `beat_index` reaches the value equal to the Numerator, THE Metronome SHALL reset `beat_index` to 0.
3. WHEN the time signature changes while the Metronome is active, THE Metronome SHALL reset `beat_index` to 0 on the next scheduled beat.
4. WHEN the Metronome is stopped and restarted, THE Metronome SHALL reset `beat_index` to 0.
5. FOR ALL numerator values N in the range 1–16, after exactly N beats the `beat_index` SHALL equal 0 (modular wrap property).

---

### Requirement 4: Accent Sound on Beat 1

**User Story:** As a musician, I want to hear a distinct accent click on the first beat of each bar, so that I can clearly identify bar boundaries while practicing.

#### Acceptance Criteria

1. WHEN `beat_index` equals 0, THE AudioEngine SHALL schedule an Accent_Click at the computed beat start time.
2. WHEN `beat_index` is greater than 0, THE AudioEngine SHALL schedule a Regular_Click at the computed beat start time.
3. THE Accent_Click SHALL use a higher pitch than the Regular_Click to produce an audible distinction.
4. THE Accent_Click SHALL use a frequency of 1800 Hz and the Regular_Click SHALL use a frequency of 1200 Hz, both with the same triangle oscillator waveform and 30 ms duration used by the existing metronome click.
5. WHILE the Metronome is muted, THE AudioEngine SHALL schedule neither Accent_Click nor Regular_Click sounds.
6. THE AudioEngine SHALL expose a `schedule_metronome_click_accented(start: f64, is_accent: bool)` function that selects the correct frequency based on the `is_accent` parameter.

---

### Requirement 5: Beat Interval Calculation

**User Story:** As a musician, I want the beat interval to reflect the denominator of the time signature, so that 6/8 at 120 BPM sounds different from 6/4 at 120 BPM.

#### Acceptance Criteria

1. THE Metronome SHALL compute the beat interval in milliseconds as `(60_000 / bpm) * (4 / denominator)`, where BPM is the current tempo and denominator is the current time signature denominator.
2. WHEN the denominator is 4, THE Metronome SHALL produce a beat interval equal to `60_000 / bpm` milliseconds (identical to the current behavior).
3. WHEN the denominator is 8, THE Metronome SHALL produce a beat interval equal to `30_000 / bpm` milliseconds (eighth-note pulse).
4. WHEN the denominator is 2, THE Metronome SHALL produce a beat interval equal to `120_000 / bpm` milliseconds (half-note pulse).
5. WHEN the BPM or time signature changes while the Metronome is active, THE Metronome SHALL apply the new beat interval starting from the next scheduled beat.

---

### Requirement 6: State Persistence

**User Story:** As a musician, I want my time signature setting to be remembered between sessions, so that I do not have to reconfigure it every time I open the app.

#### Acceptance Criteria

1. WHEN the app saves state to localStorage, THE Storage SHALL serialize the `time_signature` field as part of the persisted state object.
2. WHEN the app loads state from localStorage and a valid `time_signature` is present, THE Storage SHALL deserialize and restore the `time_signature` into AppState.
3. IF the app loads state from localStorage and no `time_signature` field is present, THEN THE AppState SHALL use the default time signature of 4/4.
4. IF the app loads state from localStorage and the stored `time_signature` contains an invalid numerator or denominator, THEN THE AppState SHALL use the default time signature of 4/4.
