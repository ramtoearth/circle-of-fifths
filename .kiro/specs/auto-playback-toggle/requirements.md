# Requirements Document

## Introduction

The Auto-Playback Toggle feature adds a persistent on/off switch that controls whether clicking
a circle segment or selecting a chord progression triggers audio playback and sequential
animation. When the toggle is **off**, clicking a segment immediately shows the full static
scale highlight on the piano without playing any audio or running any animation. When the
toggle is **on** (the default), existing behaviour is preserved exactly. The toggle state
persists across browser sessions, consistent with how mute state is already persisted.

## Glossary

- **App**: The Circle of Fifths web application.
- **Auto_Playback**: The combined behaviour of triggering audio playback and a sequential
  piano highlight animation when the user clicks a circle segment or selects a chord
  progression.
- **Playback_Toggle**: The persistent UI control (button or switch) that enables or disables
  Auto_Playback globally.
- **Toggle_State**: The current value of the Playback_Toggle — either `enabled` (Auto_Playback
  runs as normal) or `disabled` (Auto_Playback is suppressed).
- **Static_Highlight**: The immediate, non-animated piano key highlight that shows all notes
  of the selected scale or chord simultaneously, with no sequential stepping.
- **Playback_Session**: A single triggered run of scale or progression audio combined with its
  synchronized sequential piano highlight animation (as defined in the cancellable-playback
  spec).
- **Audio_Engine**: The WebAssembly audio synthesis component responsible for sound playback.
- **Piano_Panel**: The scrollable piano keyboard UI component displayed below the circle.
- **Idle_State**: The state of the Piano_Panel when no Playback_Session is active — highlights
  reflect only the statically selected key or chord.
- **PersistedState**: The subset of AppState written to and read from localStorage.

---

## Requirements

### Requirement 1: Playback Toggle Control

**User Story:** As a piano learner, I want a toggle button that turns auto-playback on or off,
so that I can choose whether clicking a note shows it silently or plays the full scale.

#### Acceptance Criteria

1. THE App SHALL display the Playback_Toggle in a persistent, reachable position in the UI at
   all times (not only during playback).
2. THE Playback_Toggle SHALL have two states: `enabled` and `disabled`, with a visible label
   or icon that reflects the current Toggle_State.
3. WHEN the user activates the Playback_Toggle, THE App SHALL switch the Toggle_State from
   `enabled` to `disabled`, or from `disabled` to `enabled`.
4. THE Playback_Toggle SHALL be accessible via keyboard focus and SHALL have a descriptive
   accessible label that reflects the current Toggle_State.

---

### Requirement 2: Suppressed Playback When Toggle Is Disabled

**User Story:** As a piano learner, I want clicking a circle segment to show the scale notes
on the piano immediately without playing audio, so that I can study the notes quietly.

#### Acceptance Criteria

1. WHILE the Toggle_State is `disabled` and the user clicks a circle segment, THE App SHALL
   NOT start a Playback_Session (no audio, no sequential animation).
2. WHILE the Toggle_State is `disabled` and the user clicks a circle segment, THE Piano_Panel
   SHALL immediately display the Static_Highlight for all notes of the selected scale.
3. WHILE the Toggle_State is `disabled` and the user clicks a circle segment, THE Audio_Engine
   SHALL NOT play any audio.
4. WHILE the Toggle_State is `disabled` and the user clicks an already-selected segment, THE
   App SHALL deselect the key and clear the Piano_Panel highlights, consistent with existing
   deselection behaviour.

---

### Requirement 3: Suppressed Playback for Chord Progressions When Toggle Is Disabled

**User Story:** As a piano learner, I want selecting a chord progression to show the chord
notes silently when auto-playback is off, so that I can read the chord without hearing it.

#### Acceptance Criteria

1. WHILE the Toggle_State is `disabled` and the user selects a chord progression, THE App
   SHALL NOT start a Playback_Session (no audio, no sequential animation).
2. WHILE the Toggle_State is `disabled` and the user selects a chord progression, THE
   Piano_Panel SHALL immediately display the Static_Highlight for the first chord of the
   progression.
3. WHILE the Toggle_State is `disabled` and the user selects the already-active progression,
   THE App SHALL cancel any active Playback_Session and transition to Idle_State, consistent
   with existing cancellation behaviour.

---

### Requirement 4: Preserved Behaviour When Toggle Is Enabled

**User Story:** As a piano learner, I want the existing playback behaviour to be unchanged
when auto-playback is on, so that I can still hear scales and progressions as before.

#### Acceptance Criteria

1. WHILE the Toggle_State is `enabled`, THE App SHALL behave identically to the pre-toggle
   implementation for all circle segment clicks, chord progression selections, and Stop
   control interactions.
2. WHILE the Toggle_State is `enabled`, THE App SHALL start a Playback_Session (audio +
   sequential animation) when the user clicks a circle segment or selects a chord progression,
   as defined by the cancellable-playback spec.

---

### Requirement 5: Cancellation Interaction with Toggle

**User Story:** As a piano learner, I want switching the toggle off during active playback to
stop the current session immediately, so that the piano reflects the silent mode right away.

#### Acceptance Criteria

1. WHEN the user switches the Toggle_State from `enabled` to `disabled` while a
   Playback_Session is active, THE App SHALL cancel the active Playback_Session immediately
   (equivalent to pressing Stop).
2. WHEN the user switches the Toggle_State from `enabled` to `disabled` while a
   Playback_Session is active, THE Piano_Panel SHALL transition to Idle_State, showing the
   Static_Highlight for the currently selected key or chord.
3. WHEN the user switches the Toggle_State from `disabled` to `enabled`, THE App SHALL NOT
   start any new Playback_Session automatically; the next user interaction triggers playback.

---

### Requirement 6: Persistence of Toggle State

**User Story:** As a piano learner, I want the auto-playback toggle setting to be remembered
between sessions, so that I do not have to reconfigure it every time I open the app.

#### Acceptance Criteria

1. THE App SHALL persist the Toggle_State to localStorage using the key `cof_auto_playback`.
2. WHEN the App loads, THE App SHALL restore the Toggle_State from localStorage; IF no stored
   value is found, THEN THE App SHALL default the Toggle_State to `enabled`.
3. FOR ALL valid Toggle_State values, serialising then deserialising SHALL produce the
   original Toggle_State value (round-trip property).
4. IF the stored value in localStorage is unrecognised or corrupt, THEN THE App SHALL default
   the Toggle_State to `enabled`.

---

### Requirement 7: Toggle Does Not Affect Mute State

**User Story:** As a piano learner, I want the auto-playback toggle and the mute button to be
independent controls, so that I can combine them freely.

#### Acceptance Criteria

1. THE App SHALL treat the Toggle_State and the mute state as independent boolean flags that
   do not affect each other.
2. WHEN the user activates the Playback_Toggle, THE App SHALL NOT change the mute state.
3. WHEN the user activates the mute control, THE App SHALL NOT change the Toggle_State.
