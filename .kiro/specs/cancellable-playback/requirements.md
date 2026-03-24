# Requirements Document

## Introduction

The Cancellable Playback feature adds an immediate stop mechanism to the Circle of Fifths app. Currently, clicking a circle segment or a chord progression triggers both audio playback and a sequential piano key highlight animation. These run to completion with no way to interrupt them — even muting audio leaves the highlight animation running. This feature introduces a stop control and ensures that any in-progress playback or animation is cancelled instantly when the user requests it, when a new segment is clicked, or when a new progression is selected.

## Glossary

- **App**: The Circle of Fifths web application.
- **Playback_Session**: A single triggered run of scale or progression audio combined with its synchronized piano highlight animation.
- **Scale_Animation**: The sequential per-note piano key highlight that runs when a circle segment is clicked, advancing one note every (60 000 / BPM) ms.
- **Progression_Animation**: The sequential per-chord piano key highlight that runs when a chord progression is selected, advancing one chord per second.
- **Animation_Handle**: A cancellable reference to one or more pending JS timeout callbacks that drive a Scale_Animation or Progression_Animation.
- **Stop_Control**: The UI button that immediately cancels the active Playback_Session.
- **Audio_Engine**: The WebAssembly audio synthesis component responsible for sound playback.
- **Piano_Panel**: The scrollable piano keyboard UI component displayed below the circle.
- **Idle_State**: The state of the Piano_Panel when no Playback_Session is active — highlights reflect only the statically selected key or chord, not any animation frame.

---

## Requirements

### Requirement 1: Stop Control Visibility

**User Story:** As a piano learner, I want a visible stop button while playback or animation is running, so that I know I can cancel it at any time.

#### Acceptance Criteria

1. WHILE a Playback_Session is active, THE App SHALL display the Stop_Control in a visible, reachable position in the UI.
2. WHILE no Playback_Session is active, THE App SHALL hide or disable the Stop_Control so it does not clutter the interface.
3. THE Stop_Control SHALL be accessible via keyboard focus and SHALL have a descriptive accessible label.

---

### Requirement 2: Immediate Cancellation via Stop Control

**User Story:** As a piano learner, I want to press a stop button and have both audio and the highlight animation stop immediately, so that I am not forced to wait for the animation to finish.

#### Acceptance Criteria

1. WHEN the user activates the Stop_Control, THE App SHALL cancel all pending Animation_Handles belonging to the active Playback_Session within one animation frame (≤ 16 ms).
2. WHEN the user activates the Stop_Control, THE Audio_Engine SHALL suspend audio output immediately.
3. WHEN the user activates the Stop_Control, THE Piano_Panel SHALL transition to Idle_State immediately, showing only the static scale or chord highlight for the currently selected key or chord.
4. WHEN the user activates the Stop_Control, THE App SHALL mark the Playback_Session as inactive so the Stop_Control is hidden.

---

### Requirement 3: Cancellation on New Segment Click

**User Story:** As a piano learner, I want clicking a new circle segment to cancel any running animation before starting the new one, so that highlights never overlap or stack.

#### Acceptance Criteria

1. WHEN the user clicks a circle segment while a Playback_Session is active, THE App SHALL cancel all pending Animation_Handles of the current Playback_Session before starting the new Scale_Animation.
2. WHEN the user clicks a circle segment while a Playback_Session is active, THE Audio_Engine SHALL stop the current audio before starting the new scale playback.
3. WHEN the user clicks an already-selected segment while a Playback_Session is active, THE App SHALL cancel the active Playback_Session and transition to Idle_State without starting a new Playback_Session.

---

### Requirement 4: Cancellation on New Progression Selection

**User Story:** As a piano learner, I want selecting a chord progression to cancel any running animation, so that the new progression starts cleanly from its first chord.

#### Acceptance Criteria

1. WHEN the user selects a chord progression while a Playback_Session is active, THE App SHALL cancel all pending Animation_Handles of the current Playback_Session before starting the new Progression_Animation.
2. WHEN the user selects a chord progression while a Playback_Session is active, THE Audio_Engine SHALL stop the current audio before starting the new progression playback.
3. WHEN the user selects the already-active progression while a Playback_Session is active, THE App SHALL cancel the active Playback_Session and transition to Idle_State without restarting playback.

---

### Requirement 5: Animation Handle Lifecycle

**User Story:** As a developer, I want animation timeouts to be tracked and cancelled cleanly, so that no stale callbacks fire after a Playback_Session is cancelled.

#### Acceptance Criteria

1. THE App SHALL store all Timeout handles created for a Playback_Session in a single Animation_Handle collection that can be cancelled atomically.
2. WHEN a Playback_Session is cancelled, THE App SHALL drop all Timeout handles in the Animation_Handle collection, preventing any pending callbacks from firing.
3. WHEN a Playback_Session completes naturally (all notes or chords have been highlighted), THE App SHALL clear the Animation_Handle collection and transition to Idle_State.
4. IF a new Playback_Session starts while an Animation_Handle collection is non-empty, THEN THE App SHALL cancel the existing collection before creating new handles.

---

### Requirement 6: Mute Does Not Block Cancellation

**User Story:** As a piano learner, I want the stop control to cancel the highlight animation even when audio is muted, so that I never have to wait for a silent animation to finish.

#### Acceptance Criteria

1. WHILE the Audio_Engine is muted and a Playback_Session is active, THE App SHALL still display the Stop_Control.
2. WHEN the user activates the Stop_Control while the Audio_Engine is muted, THE App SHALL cancel all pending Animation_Handles and transition the Piano_Panel to Idle_State.
3. WHEN a new segment is clicked while the Audio_Engine is muted and a Scale_Animation is running, THE App SHALL cancel the running Scale_Animation before starting the new one.

---

### Requirement 7: Idle State Correctness After Cancellation

**User Story:** As a piano learner, I want the piano to show the correct static highlights after I stop playback, so that the visual state is always consistent with my current selection.

#### Acceptance Criteria

1. WHEN a Playback_Session is cancelled while a Key is selected, THE Piano_Panel SHALL highlight all notes of that Key's scale in Idle_State.
2. WHEN a Playback_Session is cancelled while a chord is highlighted, THE Piano_Panel SHALL highlight the notes of that chord (root, third, fifth) in Idle_State.
3. WHEN a Playback_Session is cancelled and no Key is selected, THE Piano_Panel SHALL display no highlights in Idle_State.
4. THE Piano_Panel SHALL NOT display any animation-frame highlight (playing_note) after a Playback_Session is cancelled.
