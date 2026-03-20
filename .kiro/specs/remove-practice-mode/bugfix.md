# Bugfix Requirements Document

## Introduction

Practice Mode is currently implemented across multiple layers of the Circle of Fifths app but is not working correctly. Rather than debug it in place, the user wants it removed cleanly so it can be re-introduced later as a standalone, well-tested feature. The "bug" is that broken practice-mode code is woven into the core app: it adds dead weight to the state shape, pollutes the reducer, exposes a broken UI entry point in the nav bar, and entangles `PianoPanel` with practice-specific coloring logic. The fix removes every practice artifact while leaving all other features (circle, key info, progressions, piano, audio, MIDI, metronome, play-along, theme, storage) completely intact.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the codebase is compiled, THEN the system includes `PracticePanel`, `PracticeState`, `PracticeScore`, `EnterPractice`, `ExitPractice`, `PracticeAdvance`, `AppMode::Practice`, and `practice_state` as first-class symbols entangled with core app code

1.2 WHEN `AppState` is constructed, THEN the system allocates `practice_state: Option<PracticeState>` and `app_mode` fields that are partially shared with play-along mode, making the state shape harder to reason about

1.3 WHEN the nav bar renders, THEN the system displays a "Practice" button (when MIDI is connected) that navigates to a broken feature

1.4 WHEN `app.rs` renders, THEN the system includes an `AppMode::Practice` branch that renders `PracticePanel` — a component that does not work correctly

1.5 WHEN `PianoPanel` receives a `practice_target` prop, THEN the system applies `midi-correct` / `midi-incorrect` coloring that is only meaningful in practice mode

1.6 WHEN `src/components/mod.rs` is compiled, THEN the system exposes `pub mod practice_panel` as part of the public component surface

### Expected Behavior (Correct)

2.1 WHEN the codebase is compiled, THEN the system SHALL contain no references to `PracticePanel`, `PracticeState`, `PracticeScore`, `EnterPractice`, `ExitPractice`, `PracticeAdvance`, or `AppMode::Practice`

2.2 WHEN `AppState` is constructed, THEN the system SHALL NOT include a `practice_state` field; `app_mode` SHALL only represent `Normal` and `PlayAlong` states

2.3 WHEN the nav bar renders, THEN the system SHALL NOT display a "Practice" button or the "Connect a MIDI device to use Practice mode" hint

2.4 WHEN `app.rs` renders, THEN the system SHALL NOT include an `AppMode::Practice` render branch or `PracticePanel` usage

2.5 WHEN `PianoPanel` is rendered, THEN the system SHALL NOT receive a `practice_target` prop; the `midi-correct` / `midi-incorrect` coloring logic SHALL be removed

2.6 WHEN `src/components/mod.rs` is compiled, THEN the system SHALL NOT declare `pub mod practice_panel`

### Unchanged Behavior (Regression Prevention)

3.1 WHEN `cargo test` is run after the removal, THEN the system SHALL pass all remaining tests with no failures

3.2 WHEN the app is compiled and served, THEN the system SHALL continue to render the circle, key info panel, progression panel, piano panel, MIDI status bar, and nav bar correctly

3.3 WHEN a user interacts with MIDI input, THEN the system SHALL continue to handle note-on/note-off events, chord recognition, key suggestions, and play-along mode without change

3.4 WHEN the metronome is toggled, THEN the system SHALL continue to schedule clicks and persist `metronome_active` to localStorage

3.5 WHEN the app persists state, THEN the system SHALL continue to save and restore `theme`, `muted`, `favorites`, and `metronome_active` via localStorage

3.6 WHEN the nav bar renders, THEN the system SHALL continue to display the BPM slider, theme toggle, mute toggle, and metronome toggle

3.7 WHEN `AppAction` variants unrelated to practice are dispatched, THEN the system SHALL produce identical state transitions as before the removal

3.8 WHEN play-along mode is active, THEN the system SHALL continue to function correctly — `AppMode::PlayAlong`, `PlayAlongState`, `EnterPlayAlong`, `ExitPlayAlong`, `PlayAlongTick`, `RecordPlayAlongChordResult` are all preserved

---

## Bug Condition

```pascal
FUNCTION isBugCondition(X)
  INPUT: X — the compiled source tree
  OUTPUT: boolean

  RETURN X contains ANY of:
    file  src/components/practice_panel.rs
    token "pub mod practice_panel"         in src/components/mod.rs
    type  PracticeState                    in src/state/mod.rs
    field practice_state                   in AppState
    variant EnterPractice                  in AppAction
    variant ExitPractice                   in AppAction
    variant PracticeAdvance                in AppAction
    variant AppMode::Practice              in AppMode enum
    type  PracticeScore                    in src/midi/mod.rs
    prop  practice_target                  in PianoPanelProps
    prop  on_enter_practice                in NavBarProps
    button "Practice"                      in NavBar HTML
    hint "Connect a MIDI device"           in NavBar HTML
    import PracticePanel                   in src/components/app.rs
    callback on_enter_practice             in app.rs
    callback on_practice_exit              in app.rs
    callback on_practice_advance           in app.rs
    branch AppMode::Practice               in app.rs render
    field practice_target                  in app.rs derived values
END FUNCTION
```
