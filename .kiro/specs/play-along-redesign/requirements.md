# Requirements Document

## Introduction

Play-Along Redesign replaces the existing BPM-timed evaluation model in Play-Along Mode with a
beginner-friendly, wait-based interaction model. The current implementation advances through chords
on a fixed timer and marks each chord as correct or incorrect based on what the user was holding at
the beat boundary, producing an unintuitive experience for learners who are not yet comfortable with
strict timing. The redesign removes the timer entirely: the app displays the target chord, shows a
hand-position overlay on the existing Piano_Panel to guide finger placement, and waits indefinitely
for the user to play the correct chord before advancing. The progression loops back to the first
chord after the last one, giving learners unlimited repetitions without re-entering the mode. The
per-chord result list that was spamming the screen with ✗ marks is removed.

## Glossary

- **App**: The Circle of Fifths web application.
- **Play_Along_Mode**: The UI mode where the App guides the user through a chord progression on the
  MIDI keyboard, waiting for each chord to be played correctly before advancing.
- **Target_Chord**: The chord currently expected from the user in Play_Along_Mode.
- **Chord_Detection**: The process of comparing Held_Notes against the Target_Chord's PitchClasses
  to determine whether the chord has been played correctly.
- **Held_Notes**: The set of MIDI note numbers currently depressed on the connected MIDI keyboard.
- **PitchClass**: One of the 12 chromatic pitch classes (C, C♯/D♭, D, … B), octave-agnostic.
- **Piano_Panel**: The existing scrollable piano keyboard UI component.
- **Hand_Position_Overlay**: A visual layer rendered over the Piano_Panel that shows numbered
  finger-position indicators above the keys belonging to the Target_Chord.
- **Finger_Indicator**: A single circular UI element within the Hand_Position_Overlay showing the
  finger number (1 = thumb, 3 = middle, 5 = pinky) above the corresponding piano key.
- **Root_Position_Fingering**: The standard beginner right-hand fingering for a triad: finger 1 on
  the root, finger 3 on the third, finger 5 on the fifth.
- **Progression_Loop**: The behavior of wrapping back to the first chord in the progression after
  the last chord has been played correctly.
- **MIDI_Device**: A physical or virtual MIDI keyboard connected via the Web MIDI API.
- **Audio_Engine**: The existing WebAssembly audio synthesis component.
- **Chord_Advance_Debounce**: A brief hold period (300 ms) during which all target PitchClasses
  must remain in Held_Notes before the chord is accepted, preventing accidental advances from
  momentary overlap of adjacent chord shapes.

---

## Requirements

### Requirement 1: Wait-Based Chord Advancement

**User Story:** As a beginner piano learner, I want the play-along mode to wait until I play the
correct chord before moving on, so that I can take as long as I need without being penalised for
slow transitions.

#### Acceptance Criteria

1. WHEN Play_Along_Mode is active and the Target_Chord is displayed, THE App SHALL NOT advance to
   the next chord until all PitchClasses of the Target_Chord are simultaneously present in
   Held_Notes (Chord_Detection succeeds).
2. WHEN Chord_Detection succeeds and all target PitchClasses remain in Held_Notes for at least
   300 ms (Chord_Advance_Debounce), THE App SHALL advance to the next chord.
3. THE App SHALL NOT use a fixed-interval timer to advance chords in Play_Along_Mode; all
   advancement SHALL be triggered exclusively by Chord_Detection events.
4. WHEN Play_Along_Mode is active and the user releases notes before the Chord_Advance_Debounce
   period elapses, THE App SHALL cancel the pending advance and wait for Chord_Detection to
   succeed again.
5. THE App SHALL accept the Target_Chord when the Held_Notes contain all target PitchClasses
   regardless of octave — octave-agnostic matching SHALL be used.

---

### Requirement 2: Hand Position Overlay

**User Story:** As a beginner piano learner, I want to see numbered finger-position guides above
the piano keys I need to press, so that I know exactly where to place each finger without reading
music notation.

#### Acceptance Criteria

1. WHILE Play_Along_Mode is active, THE Piano_Panel SHALL display a Hand_Position_Overlay showing
   Finger_Indicators above each key belonging to the Target_Chord.
2. THE Hand_Position_Overlay SHALL use Root_Position_Fingering: Finger_Indicator "1" (thumb) on
   the root, Finger_Indicator "3" (middle finger) on the third, and Finger_Indicator "5" (pinky)
   on the fifth.
3. EACH Finger_Indicator SHALL be a visually distinct circular element containing the finger number,
   positioned above the corresponding piano key and clearly visible against both white and black
   keys.
4. WHEN a target key's PitchClass is present in Held_Notes (the user is pressing that note),
   THE corresponding Finger_Indicator SHALL transition to a "held" visual state (e.g., filled or
   highlighted) to confirm the note is active.
5. WHEN Play_Along_Mode is not active, THE Hand_Position_Overlay SHALL NOT be rendered.
6. THE Hand_Position_Overlay SHALL update immediately when the Target_Chord changes (on
   Progression_Loop or chord advance).

---

### Requirement 3: Continuous Progression Loop

**User Story:** As a beginner piano learner, I want the progression to loop back to the first chord
after I finish the last one, so that I can keep practising without stopping to restart.

#### Acceptance Criteria

1. WHEN the user correctly plays the last chord in the progression (Chord_Detection succeeds for
   the chord at the final index), THE App SHALL advance the current chord index back to 0
   (Progression_Loop), rather than ending the session.
2. THE App SHALL NOT display a completion summary screen in the new play-along mode; the
   progression SHALL loop indefinitely until the user explicitly stops.
3. WHEN a Progression_Loop occurs, THE App SHALL briefly indicate to the user that the progression
   has looped (e.g., a "Loop!" label or brief visual cue) before presenting the first chord again.
4. THE loop indicator SHALL be visible for no more than 1.5 seconds before the normal Target_Chord
   display resumes.

---

### Requirement 4: Clean UI — Removal of Result List

**User Story:** As a beginner piano learner, I want a clean, uncluttered play-along screen that
focuses on the current chord, so that I am not overwhelmed by historical result data.

#### Acceptance Criteria

1. THE Play_Along_Mode screen SHALL NOT display a per-chord result list (✓/✗ entries) at any
   point during or after the session.
2. THE Play_Along_Mode screen SHALL display: the Target_Chord name and Roman numeral, the current
   chord position within the progression (e.g., "Chord 2 of 4"), and a Stop button.
3. THE Play_Along_Mode screen SHALL NOT display a BPM control; the BPM field is irrelevant in
   wait-based mode and SHALL be hidden from the play-along panel.
4. WHEN Play_Along_Mode is active, THE App SHALL NOT log or accumulate per-chord accuracy scores.
5. THE Play_Along_Mode screen SHALL remain stable and uncluttered as the user plays; no new list
   items SHALL appear during the session.

---

### Requirement 5: Entry, Exit, and MIDI Prerequisites

**User Story:** As a piano learner, I want play-along mode to be gated on a connected MIDI device
and to stop cleanly when I press Stop, so that the mode only runs when it can work properly.

#### Acceptance Criteria

1. THE App SHALL allow entry into Play_Along_Mode only when a MIDI_Device is connected
   (`midi_status == Connected`) and an active Progression is selected.
2. IF no MIDI_Device is connected when the user attempts to start Play_Along_Mode, THE App SHALL
   display a message instructing the user to connect a MIDI keyboard, consistent with existing
   behaviour.
3. WHEN the user presses the Stop button, THE App SHALL exit Play_Along_Mode immediately, clear
   the Hand_Position_Overlay, and return to the normal Progression view.
4. WHEN Play_Along_Mode is exited for any reason, THE App SHALL clear any pending
   Chord_Advance_Debounce timer.
5. WHEN a MIDI_Device is disconnected while Play_Along_Mode is active, THE App SHALL exit
   Play_Along_Mode and display the disconnection notice.

---

### Requirement 6: Audio Feedback on Chord Advance

**User Story:** As a beginner piano learner, I want to hear the next chord played back when I
advance, so that I can train my ear to recognize the target chord before I play it.

#### Acceptance Criteria

1. WHEN Play_Along_Mode advances to a new Target_Chord (including the initial chord on entry),
   THE Audio_Engine SHALL play the Target_Chord notes simultaneously as an audio preview, unless
   the App is muted.
2. WHEN Play_Along_Mode is muted, THE App SHALL NOT play the audio preview but SHALL still
   advance on Chord_Detection.
3. THE audio preview SHALL use the same chord playback method as the existing "play chord"
   feature — all notes simultaneously, not sequentially.
