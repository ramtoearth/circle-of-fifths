# Remove Practice Mode — Design

## Overview

Practice mode is entangled across six source files. The fix is a pure deletion — no new logic is introduced. Every practice symbol is removed from `src/components/practice_panel.rs` (deleted), `src/components/mod.rs`, `src/state/mod.rs`, `src/midi/mod.rs`, `src/components/piano_panel.rs`, `src/components/nav_bar.rs`, and `src/components/app.rs`. Play-along mode, MIDI, metronome, and all other features are left completely intact.

## Files to Modify

### DELETE: `src/components/practice_panel.rs`
Delete the entire file.

### MODIFY: `src/components/mod.rs`
- Remove: `pub mod practice_panel;`

### MODIFY: `src/state/mod.rs`
- Remove type: `PracticeState` struct
- Remove variant from `AppMode` enum: `Practice` — keep `Normal` and `PlayAlong`
- Remove from `AppState`: `practice_state: Option<PracticeState>` field
- Remove from `AppState::default()`: `practice_state: None`
- Remove from `AppAction`: `EnterPractice`, `ExitPractice`, `PracticeAdvance` variants
- Remove reducer arms for `EnterPractice`, `ExitPractice`, `PracticeAdvance`
- Update `EnterPlayAlong` reducer arm: remove the guard `if state.app_mode != AppMode::Normal` — or keep it checking only `PlayAlong` (since `Practice` variant is gone, the guard `!= Normal` still works with just two variants)
- Remove tests: none specific to practice in state tests (the quiz tests were the ones removed; practice reducer arms have no dedicated unit tests in state/mod.rs)

### MODIFY: `src/midi/mod.rs`
- Remove type: `PracticeScore` struct
- Keep: `ChordResult`, `PlayAlongScore`, `HeldNote`, `MidiStatus`, `RecognizedChord`, `KeySuggestion`, `MidiEvent`, `MidiEngine`, chord recognition, key detection — all untouched

### MODIFY: `src/components/piano_panel.rs`
- Remove from `PianoPanelProps`: `practice_target: Option<Vec<PitchClass>>` prop
- Remove the `midi-correct` / `midi-incorrect` coloring logic that uses `practice_target`
- Keep: `held_notes` prop and `midi-held` opacity coloring (used by play-along too)
- Keep: all other piano rendering logic

### MODIFY: `src/components/nav_bar.rs`
- Remove from `NavBarProps`: `on_enter_practice: Callback<()>`
- Remove from component body: `let on_enter_practice = props.on_enter_practice.reform(|_: MouseEvent| ());`
- Remove from rendered HTML: the `if props.midi_status == MidiStatus::Connected { <button ...>Practice</button> } else { <span>Connect a MIDI device...</span> }` block entirely

### MODIFY: `src/components/app.rs`
- Remove import: `use crate::components::practice_panel::PracticePanel;`
- Remove import: `use crate::components::play_along_panel::PlayAlongPanel;` — wait, play-along stays. Keep that import.
- Remove `PracticeAdvance` from `use crate::state::{...}` import (if listed explicitly)
- Remove derived value: `let practice_target: Option<Vec<PitchClass>> = if let Some(ref pa) = state.play_along_state { ... } else { state.practice_state.as_ref().map(...) };` — simplify to only the play-along branch, or pass `None` directly to `PianoPanel` if `practice_target` prop is removed entirely
- Remove callback: `on_enter_practice`
- Remove callback: `on_practice_exit`
- Remove callback: `on_practice_advance`
- Remove from `<NavBar>` usage: `on_enter_practice={on_enter_practice}`
- Remove from `<PianoPanel>` usage: `practice_target={practice_target}` prop
- Remove render branch: `if state.app_mode == AppMode::Practice { if let Some(ref ps) = state.practice_state { <PracticePanel ... /> } }`
- Update the render condition: change `} else if state.app_mode == AppMode::PlayAlong {` to be the primary branch (no longer needs to be `else if`)

## Correctness Properties

**Property 1: Bug Condition — Practice Symbols Absent After Fix**
After applying the fix, `isBugCondition` SHALL return `false`: no practice symbol from the enumerated list appears anywhere in the source tree, and `cargo test` compiles and passes with zero errors and zero test failures.

**Property 2: Preservation — Non-Practice Behavior Unchanged**
All behaviors that do NOT involve practice symbols (MIDI handling, play-along mode, metronome, storage, nav bar controls, circle rendering) SHALL produce exactly the same behavior as before the fix. All existing non-practice tests must continue to pass.

## Notes

- `AppMode` will have only two variants after this fix: `Normal` and `PlayAlong`. The `EnterPlayAlong` reducer guard `if state.app_mode != AppMode::Normal` still works correctly.
- `practice_target` in `app.rs` currently has two branches: one for play-along (derives target from `play_along_state`) and one for practice (derives from `practice_state`). The play-along branch must be KEPT — `PianoPanel` uses `practice_target` to show `midi-correct`/`midi-incorrect` coloring during play-along too. So the prop stays on `PianoPanel`, but its name should be clarified or kept as-is. Only the practice branch of the derived value is removed; the play-along branch remains.
- `PracticeScore` lives in `src/midi/mod.rs` (not `src/state/mod.rs`) — make sure to remove it from there.
- Because `practice_target` is also used by play-along, `PianoPanelProps.practice_target` and `practice_key_class()` are KEPT. Only the practice-specific tests in `piano_panel.rs` that test practice-mode behavior can stay (they test a pure function that play-along also uses). The prop rename is optional — keeping it as `practice_target` is fine since play-along uses the same coloring logic.
