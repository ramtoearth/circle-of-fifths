# Implementation Plan: Remove Practice Mode

## Overview

Pure deletion across 7 files. No new logic is introduced. Tasks 1–2 establish the baseline. Tasks 3.1–3.6 are fully parallel (different files). Task 3.7–3.8 verify the fix.

## Tasks

- [x] 1. Confirm bug condition — practice symbols present before fix
  - Run grep checks to confirm all practice symbols exist in the unfixed codebase:
    - `grep -r "PracticePanel\|PracticeState\|PracticeScore\|EnterPractice\|ExitPractice\|PracticeAdvance\|AppMode::Practice\|practice_state\|on_enter_practice\|on_practice_exit\|on_practice_advance\|pub mod practice_panel" src/`
  - All should return matches — document the hits
  - **RESULT**: 44 matches across 7 files — `isBugCondition` = true. Hits:
    - `app.rs`: PracticePanel import, on_enter_practice, on_practice_exit, on_practice_advance callbacks, practice_state branch, AppMode::Practice render branch (14 hits)
    - `components/mod.rs`: pub mod practice_panel (1 hit)
    - `nav_bar.rs`: on_enter_practice prop, local binding, Practice button (3 hits)
    - `practice_panel.rs`: PracticeScore, PracticePanelProps, PracticePanel component (7 hits)
    - `midi/mod.rs`: PracticeScore struct (1 hit)
    - `midi/tests.rs`: PracticeScore::default() (1 hit)
    - `state/mod.rs`: PracticeState, practice_state field, EnterPractice/ExitPractice/PracticeAdvance variants + reducer arms, AppMode::Practice (17 hits)
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [x] 2. Record preservation baseline — run cargo test on unfixed code
  - Run `cargo test` and confirm all tests pass before any changes
  - Record which test suites pass (MIDI reducer, storage, music theory, piano panel, practice panel)
  - **RESULT**: 120 passed, 0 failed on unfixed code. Baseline confirmed across all suites.
  - _Requirements: 3.1–3.8_

- [ ] 3. Remove all practice-mode code

  - [x] 3.1 Delete `src/components/practice_panel.rs`
    - Delete the entire file
    - _Requirements: 2.1, 2.6_
    - _Depends on: None_

  - [x] 3.2 Remove `practice_panel` from `src/components/mod.rs`
    - Remove the line: `pub mod practice_panel;`
    - _Requirements: 2.6_
    - _Depends on: None_

  - [ ] 3.3 Remove practice types and reducer arms from `src/state/mod.rs`
    - Remove type: `PracticeState` struct (the entire struct definition)
    - Remove variant from `AppMode` enum: `Practice` — keep `Normal` and `PlayAlong`
    - Remove from `AppState`: `practice_state: Option<PracticeState>` field
    - Remove from `AppState::default()`: `practice_state: None`
    - Remove from `AppAction`: `EnterPractice`, `ExitPractice`, `PracticeAdvance` variants
    - Remove reducer arms for `AppAction::EnterPractice`, `AppAction::ExitPractice`, `AppAction::PracticeAdvance`
    - Note: the `EnterPlayAlong` reducer arm has a guard `if state.app_mode != AppMode::Normal` — this still compiles correctly with only two variants (`Normal`, `PlayAlong`)
    - _Requirements: 2.1, 2.2_
    - _Depends on: None_

  - [x] 3.4 Remove `PracticeScore` from `src/midi/mod.rs`
    - Remove the `PracticeScore` struct definition and its `#[derive]` line
    - Keep: `ChordResult`, `PlayAlongScore`, `HeldNote`, `MidiStatus`, `RecognizedChord`, `KeySuggestion`, `MidiEvent`, `MidiEngine`, all functions — untouched
    - _Requirements: 2.1_
    - _Depends on: None_

  - [ ] 3.5 Remove practice props from `src/components/nav_bar.rs`
    - Remove from `NavBarProps`: `on_enter_practice: Callback<()>`
    - Remove from component body: `let on_enter_practice = props.on_enter_practice.reform(|_: MouseEvent| ());`
    - Remove from rendered HTML: the entire `if props.midi_status == MidiStatus::Connected { <button ...>Practice</button> } else { <span>Connect a MIDI device to use Practice mode</span> }` block
    - Keep: `midi_status` prop (still used by nothing after this removal — but check if it's still needed for any other purpose; if not, it can stay as dead prop or be removed too)
    - Note: `midi_status` is still passed from `app.rs` to `NavBar` — it's fine to keep the prop even if the Practice button is gone, since it costs nothing
    - _Requirements: 2.3_
    - _Depends on: None_

  - [ ] 3.6 Remove practice wiring from `src/components/app.rs`
    - Remove import: `use crate::components::practice_panel::PracticePanel;`
    - Remove `PracticeAdvance` from `use crate::state::{AppAction, AppMode, AppState, ProgressionId, SessionResult, Theme}` import (remove `PracticeAdvance` if listed; `SessionResult` may already be removed by the quiz removal task)
    - Remove callback: `on_enter_practice` (the `Callback::from(move |_| state.dispatch(AppAction::EnterPractice))` definition)
    - Remove callback: `on_practice_exit` (the `Callback::from(move |_: ()| state.dispatch(AppAction::ExitPractice))` definition)
    - Remove callback: `on_practice_advance` (the `Callback::from(move |_: ()| state.dispatch(AppAction::PracticeAdvance))` definition)
    - Simplify the `practice_target` derived value: remove the `else { state.practice_state.as_ref().map(|ps| ps.target_chord.notes.to_vec()) }` branch — keep only the play-along branch:
      ```rust
      let practice_target: Option<Vec<crate::music_theory::PitchClass>> =
          if let Some(ref pa) = state.play_along_state {
              crate::data::find_progression(pa.progression_id).and_then(|prog| {
                  let chords = crate::music_theory::diatonic_chords(prog.key);
                  prog.chords
                      .get(pa.current_chord_index)
                      .and_then(|&d| chords.iter().find(|c| c.degree == d))
                      .map(|c| c.notes.to_vec())
              })
          } else {
              None
          };
      ```
    - Remove from `<NavBar>` usage: `on_enter_practice={on_enter_practice}`
    - Remove render branch: `if state.app_mode == AppMode::Practice { if let Some(ref ps) = state.practice_state { <PracticePanel ... /> } }`
    - Update the play-along render condition: change `} else if state.app_mode == AppMode::PlayAlong {` — this can stay as `else if` since the `Practice` branch above it is gone; or restructure as needed so it compiles
    - Keep: `practice_target` derived value (now play-along only), `practice_target={practice_target}` prop on `<PianoPanel>` — play-along still uses this for note coloring
    - _Requirements: 2.4, 2.5_
    - _Depends on: None_

  - [ ] 3.7 Verify fix — re-run grep checks from task 1
    - Re-run the same grep command from task 1 on the fixed code
    - Expected: zero matches for all practice symbols
    - Run `cargo check` — expect zero compilation errors
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 3.8 Verify preservation — run cargo test
    - Run `cargo test` on the fixed codebase
    - Expected: all remaining tests pass (MIDI reducer, storage, music theory, piano panel tests)
    - Note: practice_panel tests are gone (file deleted), but all other test suites must be green
    - _Requirements: 3.1–3.8_

- [ ] 4. Checkpoint — confirm zero test failures and zero compilation errors
  - Run `cargo test` and confirm exit code 0
  - Confirm no quiz or practice symbols remain in `src/`
  - Ask the user if any questions arise
