# Requirements Document

## Introduction

MIDI Keyboard Integration extends the existing Circle of Fifths web application with real-time physical MIDI keyboard support via the Web MIDI API. The feature enables users to connect a hardware MIDI keyboard to the browser, see their played notes reflected live on the existing Piano_Panel, receive chord and key recognition feedback, and engage in structured practice and play-along modes. All MIDI processing runs entirely in the browser (Rust/WASM + JS interop via web-sys/js-sys) with no backend required. The feature integrates into the existing Elm-like unidirectional state architecture by dispatching new `AppAction` variants into the existing reducer.

## Glossary

- **App**: The Circle of Fifths web application.
- **MIDI_Device**: A physical or virtual MIDI keyboard connected to the host computer and accessible via the Web MIDI API.
- **MIDI_Access**: The browser-granted permission object returned by `navigator.requestMIDIAccess()`.
- **NoteOn_Event**: A MIDI message indicating a key was pressed, carrying a note number (0–127) and velocity (1–127).
- **NoteOff_Event**: A MIDI message indicating a key was released, carrying a note number (0–127).
- **MIDI_Note_Number**: An integer 0–127 uniquely identifying a pitch across octaves (middle C = 60).
- **PitchClass**: One of the 12 pitch classes (C, C♯/D♭, D, … B), derived from a MIDI_Note_Number modulo 12.
- **Octave**: The octave number derived from a MIDI_Note_Number divided by 12, minus 1 (middle C = octave 4).
- **Velocity**: An integer 1–127 representing how hard a MIDI key was pressed; higher values indicate harder presses.
- **Held_Notes**: The set of MIDI_Note_Numbers currently depressed (NoteOn received, NoteOff not yet received).
- **Piano_Panel**: The existing scrollable piano keyboard UI component.
- **Velocity_Intensity**: A visual highlight intensity on the Piano_Panel proportional to the Velocity of a Held_Note.
- **Chord_Recognition**: The process of identifying a named chord (e.g. "Am", "Cmaj7") from a set of PitchClasses.
- **Roman_Numeral**: A Roman numeral label (I, ii, iii, IV, V, vi, vii°) denoting a chord's scale degree and quality within a Key.
- **Diatonic**: Belonging to the notes and chords of the currently selected Key.
- **Borrowed_Chord**: A chord whose root or quality does not belong to the currently selected Key's diatonic set (modal interchange).
- **Key_Detection**: The process of inferring the most likely musical key(s) from a set of recently played notes.
- **Rolling_Window**: A fixed-duration time window (e.g. 10 seconds) over which recently played notes are accumulated for Key_Detection.
- **Practice_Mode**: A structured exercise mode where the App presents a target chord or progression and evaluates the user's MIDI input against it.
- **Play_Along_Mode**: A mode where the App plays a chord progression via the Audio_Engine and the user follows along on the MIDI keyboard in real time.
- **Target_Chord**: The chord the user is expected to play in Practice_Mode.
- **Audio_Engine**: The existing WebAssembly audio synthesis component.
- **Circle**: The interactive circle of fifths SVG diagram.
- **Key**: A musical key (e.g. C major, A minor).
- **MIDI_Status**: The current connection state of the MIDI subsystem: `Unavailable`, `PermissionDenied`, `NoDevices`, or `Connected`.

---

## Requirements

### Requirement 1: MIDI Access and Device Management

**User Story:** As a piano learner, I want the app to connect to my MIDI keyboard automatically, so that I can start playing without manual configuration.

#### Acceptance Criteria

1. WHEN the App loads, THE App SHALL request MIDI access from the browser via `navigator.requestMIDIAccess()` without requiring sysex permissions.
2. IF the browser does not support the Web MIDI API, THEN THE App SHALL display a notice informing the user that MIDI input requires a Chromium-based browser, and all non-MIDI features SHALL continue to function normally.
3. IF the user denies the MIDI access permission request, THEN THE App SHALL display a notice explaining that MIDI input is unavailable and provide instructions for re-enabling permission, and all non-MIDI features SHALL continue to function normally.
4. WHEN MIDI_Access is granted and at least one MIDI_Device is present, THE App SHALL display the name of the connected MIDI_Device in the UI.
5. WHEN multiple MIDI_Device inputs are present, THE App SHALL listen to all connected MIDI_Device inputs simultaneously.
6. WHEN a new MIDI_Device is connected after the App has loaded, THE App SHALL detect the new device and begin listening to it without requiring a page reload.
7. WHEN a MIDI_Device is disconnected, THE App SHALL update the displayed device name and remove Held_Notes that originated from that device.
8. THE App SHALL display the current MIDI_Status in the UI at all times.

---

### Requirement 2: Real-Time Note Input and Piano Panel Highlighting

**User Story:** As a piano learner, I want the notes I play on my MIDI keyboard to light up on the piano panel in real time, so that I can see exactly what I'm playing.

#### Acceptance Criteria

1. WHEN a NoteOn_Event is received, THE App SHALL add the corresponding MIDI_Note_Number to the Held_Notes set and highlight the matching key on the Piano_Panel within 20ms of the event timestamp.
2. WHEN a NoteOff_Event is received, THE App SHALL remove the corresponding MIDI_Note_Number from the Held_Notes set and remove the highlight from the matching Piano_Panel key.
3. THE App SHALL derive the PitchClass and Octave from each MIDI_Note_Number and highlight the Piano_Panel key at the correct PitchClass and Octave position.
4. THE App SHALL render the highlight intensity of each Held_Note on the Piano_Panel proportional to its Velocity, using a visually distinct gradient or opacity scale between the minimum (Velocity = 1) and maximum (Velocity = 127) values.
5. WHEN a NoteOn_Event is received with Velocity = 0, THE App SHALL treat it as a NoteOff_Event for that note.
6. WHILE Held_Notes are present, THE Piano_Panel SHALL scroll automatically to keep the lowest Held_Note in view if it falls outside the currently visible range.
7. WHEN all keys are released (Held_Notes becomes empty), THE Piano_Panel SHALL revert to displaying only the scale highlights of the currently selected Key, if any.

---

### Requirement 3: Chord Recognition

**User Story:** As a piano learner, I want the app to identify the chord I'm playing, so that I can learn chord shapes and their names.

#### Acceptance Criteria

1. WHEN the Held_Notes set contains 3 or more distinct PitchClasses, THE App SHALL identify the best-matching named chord and display its name (e.g. "Am", "Cmaj7", "G7") in the MIDI status area.
2. WHEN the Held_Notes set contains fewer than 3 distinct PitchClasses, THE App SHALL clear the recognized chord display.
3. WHILE a Key is selected and a chord is recognized, THE App SHALL display the Roman_Numeral of the recognized chord within the selected Key (e.g. "vi").
4. WHILE a Key is selected and a chord is recognized, THE App SHALL indicate whether the recognized chord is Diatonic or a Borrowed_Chord relative to the selected Key.
5. WHEN the Held_Notes set changes, THE App SHALL update the chord recognition result within 20ms.
6. IF no standard chord name matches the Held_Notes set, THEN THE App SHALL display the note names of the Held_Notes without a chord label.

---

### Requirement 4: Key and Scale Detection

**User Story:** As a piano learner, I want the app to suggest which key I'm playing in based on my recent notes, so that I can discover the key of a melody or improvisation.

#### Acceptance Criteria

1. THE App SHALL accumulate all NoteOn_Event PitchClasses received within a Rolling_Window of 10 seconds for Key_Detection analysis.
2. WHEN the accumulated PitchClass set within the Rolling_Window contains 4 or more distinct PitchClasses, THE App SHALL compute and display the top 3 candidate Keys ranked by how many of the accumulated PitchClasses belong to each Key's scale.
3. WHEN the accumulated PitchClass set within the Rolling_Window contains fewer than 4 distinct PitchClasses, THE App SHALL not display key suggestions.
4. THE App SHALL highlight the top candidate Key's Segment on the Circle with a visually distinct indicator that does not override the user's manually selected Key highlight.
5. WHEN the Rolling_Window advances and the accumulated PitchClass set changes, THE App SHALL recompute Key_Detection and update the suggestions within 100ms.
6. THE App SHALL provide a control to clear the Rolling_Window and reset Key_Detection suggestions.

---

### Requirement 5: Practice Mode

**User Story:** As a piano learner, I want a practice mode that shows me target chords to play and tells me if I got them right, so that I can build muscle memory for chord shapes.

#### Acceptance Criteria

1. THE App SHALL provide a Practice_Mode accessible from the main navigation when a MIDI_Device is connected.
2. WHEN Practice_Mode is entered, THE App SHALL display a Target_Chord or target progression on screen for the user to play.
3. WHILE Practice_Mode is active, THE App SHALL compare the Held_Notes set against the Target_Chord notes and color-code each Piano_Panel key: green for a correct note that is part of the Target_Chord, red for an incorrect note not part of the Target_Chord, and uncolored for Target_Chord notes not yet played.
4. WHEN the user plays all notes of the Target_Chord (all Target_Chord PitchClasses are present in the Held_Notes set), THE App SHALL register the chord as correctly played and advance to the next Target_Chord.
5. THE App SHALL compute and display a per-chord accuracy score as the ratio of correct notes played to total notes played for that chord attempt.
6. WHEN a target progression is completed, THE App SHALL display a summary showing the accuracy score for each chord and the overall progression accuracy.
7. IF no MIDI_Device is connected when the user attempts to enter Practice_Mode, THEN THE App SHALL display a message instructing the user to connect a MIDI keyboard.

---

### Requirement 6: Chord Play-Along Mode

**User Story:** As a piano learner, I want to play along with a chord progression while the app plays it back, so that I can practice timing and chord transitions.

#### Acceptance Criteria

1. THE App SHALL provide a Play_Along_Mode accessible from the Progression panel when a MIDI_Device is connected and a Progression is active.
2. WHEN Play_Along_Mode is started, THE Audio_Engine SHALL begin playing the active Progression from the first chord, advancing one chord per beat at the tempo defined by `AppState.bpm` (range 40–200 BPM).
3. WHILE Play_Along_Mode is active, THE App SHALL display the current expected chord and highlight its notes on the Piano_Panel as the target.
4. WHILE Play_Along_Mode is active, THE App SHALL compare the Held_Notes set against the expected chord notes and highlight Piano_Panel keys: green for correct notes and red for incorrect notes, updated in real time.
5. THE App SHALL record whether the user played the correct PitchClasses for each chord within the chord's time window and compute a per-chord accuracy score.
6. WHEN Play_Along_Mode completes the full Progression, THE App SHALL display a results summary showing per-chord accuracy and overall accuracy.
7. THE App SHALL provide a control to stop Play_Along_Mode at any time and return to the normal Progression view.
8. IF no MIDI_Device is connected when the user attempts to start Play_Along_Mode, THEN THE App SHALL display a message instructing the user to connect a MIDI keyboard.
9. WHEN Play_Along_Mode is started, THE App SHALL automatically enable the Metronome if it is not already active, and SHALL restore the Metronome to its previous state (active or inactive) when Play_Along_Mode stops.

---

### Requirement 7: Metronome

**User Story:** As a piano learner, I want a metronome that clicks at the current BPM, so that I can practice with a steady beat.

#### Acceptance Criteria

1. THE App SHALL provide a Metronome toggle button in the NavBar, adjacent to the BPM slider.
2. WHEN the Metronome is toggled on, THE Audio_Engine SHALL begin scheduling a short high-pitched click sound on every beat at the tempo defined by `AppState.bpm`.
3. WHEN the Metronome is toggled off, THE Audio_Engine SHALL stop scheduling click sounds.
4. THE Metronome SHALL use `AppState.bpm` as its tempo source; no separate BPM field exists for the Metronome.
5. WHEN `AppState.bpm` changes while the Metronome is active, THE Metronome SHALL update its click interval to match the new BPM without requiring the user to toggle the Metronome off and on.
6. THE Metronome click sound SHALL be audibly distinct from chord and scale playback sounds (a short high-pitched oscillator burst, not a sine-wave note).
7. THE App SHALL persist `metronome_active` in localStorage so that the Metronome state is restored on page reload.
8. THE NavBar BPM slider SHALL have a range of 40–200 BPM.
9. WHEN the Metronome is active and the App is muted, THE Metronome SHALL also be silent.

