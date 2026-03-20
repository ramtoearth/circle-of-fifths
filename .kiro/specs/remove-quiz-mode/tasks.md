# Implementation Plan

- [x] 1. Write bug condition exploration test
  - **Property 1: Bug Condition** - Quiz Symbols Present Before Fix
  - **CRITICAL**: This test MUST FAIL on fixed code - passing confirms the bug exists on unfixed code
  - **DO NOT attempt to fix the test or the code when it fails after the fix is applied**
  - **GOAL**: Confirm that `isBugCondition` returns `true` on the unfixed codebase
  - **Scoped grep approach**: For each symbol in the enumerated list, confirm it is present in the source tree
  - Run the following checks on UNFIXED code — all should find matches:
    - `grep -r "QuizPanel" src/` — expect hits in `app.rs`, `components/mod.rs`, `quiz_panel.rs`
    - `grep -r "QuestionType\|BestScores\|SessionResult\|quiz_active" src/` — expect hits in `state/mod.rs`, `storage/mod.rs`, `app.rs`, `quiz_panel.rs`
    - `grep -r "EnterQuiz\|ExitQuiz\|RecordQuizResult" src/` — expect hits in `state/mod.rs`, `app.rs`
    - `grep -r "KEY_BEST_SCORES\|serialize_best_scores\|deserialize_best_scores" src/` — expect hits in `storage/mod.rs`
    - `grep -r "on_enter_quiz\|Quiz Mode\|quiz_panel" src/` — expect hits in `nav_bar.rs`, `app.rs`, `components/mod.rs`
  - **EXPECTED OUTCOME**: All grep commands return matches (confirms `isBugCondition` is `true`)
  - **RESULT**: All five grep checks returned matches — `isBugCondition` is `true`. Counterexamples:
    - `QuizPanel`: `app.rs:13` (import), `app.rs:408` (render), `quiz_panel.rs:129,143,144`
    - `QuestionType|BestScores|SessionResult|quiz_active`: `state/mod.rs:19,32,81,82,109,110,128,131,150,331,336`, `storage/mod.rs:3,22,32`, `app.rs:16`, `quiz_panel.rs` (many)
    - `EnterQuiz|ExitQuiz|RecordQuizResult`: `state/mod.rs:148-150,330-340,569,575-576`, `app.rs:267,319,325-326`
    - `KEY_BEST_SCORES|serialize_best_scores|deserialize_best_scores`: `storage/mod.rs:12,70,74,118-119,139,208,218,253-254`
    - `on_enter_quiz|Quiz Mode|quiz_panel`: `components/mod.rs:10`, `nav_bar.rs:17,28,79-80`, `app.rs:13,265,361`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Non-Quiz Behavior Unchanged
  - **IMPORTANT**: Follow observation-first methodology — run existing tests on UNFIXED code first
  - Observe: `cargo test` passes all non-quiz tests on unfixed code (MIDI reducer properties, storage round-trips, state reducer tests)
  - Confirm the following test groups pass on UNFIXED code:
    - `prop_note_on_off_round_trip`, `prop_velocity_zero_is_note_off`, `prop_clear_rolling_window`, `prop_empty_devices_clears_held_notes`, `prop_bpm_clamping`, `prop_exit_play_along_restores_metronome` in `src/state/mod.rs`
    - `theme_round_trip_dark`, `theme_round_trip_light`, `muted_round_trip_*`, `favorites_round_trip_*`, `metronome_active_round_trip_*`, `load_state_returns_defaults_in_native_target` in `src/storage/mod.rs`
    - `select_key_sets_selected_key`, `select_key_twice_deselects`, `favorite_toggle_round_trip`, `mute_toggle_round_trip`, `theme_toggle_round_trip`, `octave_clamp_*` in `src/state/mod.rs`
  - Record baseline: all listed tests PASS on unfixed code
  - **EXPECTED OUTCOME**: All preservation tests PASS on unfixed code (establishes baseline to preserve)
  - **RESULT**: `cargo test` — 139 passed, 0 failed on unfixed code. Baseline confirmed:
    - MIDI reducer props (7): `prop_note_on_off_round_trip`, `prop_velocity_zero_is_note_off`, `prop_clear_rolling_window`, `prop_empty_devices_clears_held_notes`, `prop_set_bpm_clamped`, `prop_exit_play_along_resets_mode`, `prop_metronome_toggle_round_trip` — all ✅
    - Storage round-trips (9): `theme_round_trip_{dark,light}`, `muted_round_trip_{true,false}`, `favorites_round_trip_{empty,nonempty}`, `metronome_active_round_trip_{true,false}`, `load_state_returns_defaults_in_native_target` — all ✅
    - State reducer units (7): `select_key_sets_selected_key`, `select_key_twice_deselects`, `favorite_toggle_round_trip`, `mute_toggle_round_trip`, `theme_toggle_round_trip`, `octave_clamp_{at_min,at_max}` — all ✅
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 3. Remove all quiz-mode code

  - [x] 3.1 Delete src/components/quiz_panel.rs
    - Delete the entire file `src/components/quiz_panel.rs`
    - _Bug_Condition: isBugCondition(X) where X contains file src/components/quiz_panel.rs_
    - _Expected_Behavior: file does not exist after fix_
    - _Preservation: no other component depends on quiz_panel.rs_
    - _Requirements: 2.1, 2.6_

  - [x] 3.2 Remove quiz_panel from src/components/mod.rs
    - Remove the line `pub mod quiz_panel;`
    - _Bug_Condition: isBugCondition(X) where X contains "pub mod quiz_panel" in components/mod.rs_
    - _Expected_Behavior: quiz_panel is not part of the public component surface_
    - _Requirements: 2.6_

  - [ ] 3.3 Remove quiz types and reducer arms from src/state/mod.rs
    - Remove types: `QuestionType` enum, `Question` struct, `BestScores` struct, `SessionResult` struct (the entire "Quiz / app-level types" section and its contents)
    - Remove from `AppState`: `quiz_active: bool` and `best_scores: BestScores` fields
    - Remove from `AppState::default()`: `quiz_active: false` and `best_scores: BestScores::default()`
    - Remove from `AppAction`: `EnterQuiz`, `ExitQuiz`, `RecordQuizResult(SessionResult)` variants
    - Remove reducer arms for `AppAction::EnterQuiz`, `AppAction::ExitQuiz`, `AppAction::RecordQuizResult`
    - Remove tests: `enter_quiz_sets_quiz_active` and `exit_quiz_clears_quiz_active`
    - _Bug_Condition: isBugCondition(X) where X contains QuestionType/Question/BestScores/SessionResult/quiz_active/EnterQuiz/ExitQuiz/RecordQuizResult_
    - _Expected_Behavior: AppState has no quiz_active or best_scores fields; AppAction has no quiz variants_
    - _Preservation: all non-quiz AppAction variants and their reducer arms remain unchanged_
    - _Requirements: 2.1, 2.2_

  - [ ] 3.4 Remove quiz storage from src/storage/mod.rs
    - Remove `BestScores` from the `use crate::state::{...}` import
    - Remove `KEY_BEST_SCORES` constant
    - Remove `best_scores: BestScores` field from `PersistedState` struct
    - Remove `best_scores: BestScores::default()` from `PersistedState::default()`
    - Remove `serialize_best_scores()` and `deserialize_best_scores()` functions
    - Remove best_scores reading from `load_state()` (the `ls_get(KEY_BEST_SCORES)` block and `best_scores` from the returned struct literal)
    - Remove `ls_set(KEY_BEST_SCORES, ...)` call from `save_state()`
    - Remove tests: `best_scores_round_trip_default`, `best_scores_round_trip_with_values`, `deserialize_best_scores_invalid_json_falls_back_to_default`
    - Remove `use crate::state::BestScores;` from the test module if it becomes unused
    - Update `load_state_returns_defaults_in_native_target`: remove the `assert_eq!(state.best_scores.key_sig, None);` assertion line
    - _Bug_Condition: isBugCondition(X) where X contains KEY_BEST_SCORES/best_scores in PersistedState/serialize_best_scores/deserialize_best_scores_
    - _Expected_Behavior: PersistedState has no best_scores field; load_state and save_state do not touch cof_best_scores_
    - _Preservation: theme, muted, favorites, metronome_active serialization/deserialization functions and tests remain unchanged_
    - _Requirements: 2.1, 2.3_

  - [ ] 3.5 Remove quiz props from src/components/nav_bar.rs
    - Remove `on_enter_quiz: Callback<()>` from `NavBarProps`
    - Remove `let on_enter_quiz = props.on_enter_quiz.reform(|_: MouseEvent| ());` from component body
    - Remove the `<button class="nav-bar__btn nav-bar__btn--quiz" onclick={on_enter_quiz}>{ "Quiz Mode" }</button>` element from rendered HTML
    - _Bug_Condition: isBugCondition(X) where X contains on_enter_quiz prop in NavBarProps or "Quiz Mode" button in NavBar HTML_
    - _Expected_Behavior: NavBarProps has no on_enter_quiz field; nav bar renders no Quiz Mode button_
    - _Preservation: bpm, on_set_bpm, metronome_active, on_toggle_metronome, on_enter_practice, midi_status, on_toggle_theme, on_toggle_mute, selected_key, theme props and their rendered elements remain unchanged_
    - _Requirements: 2.1, 2.4_

  - [ ] 3.6 Remove quiz wiring from src/components/app.rs
    - Remove `use crate::components::quiz_panel::QuizPanel;` import
    - Remove `SessionResult` from `use crate::state::{...}` import
    - Remove `s.best_scores = persisted.best_scores;` from state init block
    - Remove `state.best_scores.clone()` from the `use_effect_with` persistence dependency tuple
    - Remove `on_enter_quiz` callback definition
    - Remove `on_quiz_exit` callback definition
    - Remove `on_session_end` callback definition
    - Remove `on_enter_quiz={on_enter_quiz}` prop from `<NavBar>` usage
    - Remove `else if state.quiz_active { <QuizPanel ... /> }` render branch
    - _Bug_Condition: isBugCondition(X) where X contains QuizPanel import/SessionResult import/quiz callbacks/quiz render branch in app.rs_
    - _Expected_Behavior: app.rs has no quiz imports, no quiz callbacks, no quiz render branch, and NavBar usage has no on_enter_quiz prop_
    - _Preservation: all other callbacks, state init fields, persistence deps, and render branches remain unchanged_
    - _Requirements: 2.1, 2.5_

  - [ ] 3.7 Verify bug condition exploration test now passes (fix checking)
    - **Property 1: Expected Behavior** - Quiz Symbols Absent After Fix
    - **IMPORTANT**: Re-run the SAME grep checks from task 1 — do NOT write new checks
    - Run all grep commands from task 1 on the FIXED code
    - **EXPECTED OUTCOME**: All grep commands return zero matches (confirms `isBugCondition` is `false`)
    - Confirm `cargo check` compiles with zero errors
    - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [ ] 3.8 Verify preservation tests still pass
    - **Property 2: Preservation** - Non-Quiz Behavior Unchanged
    - **IMPORTANT**: Re-run the SAME tests from task 2 — do NOT write new tests
    - Run `cargo test` on the fixed codebase
    - **EXPECTED OUTCOME**: All preservation tests PASS (confirms no regressions)
    - Confirm all tests listed in task 2 still pass after the fix
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 4. Checkpoint - Ensure all tests pass
  - Run `cargo test` and confirm exit code 0 with zero test failures
  - Confirm zero compilation errors or warnings related to the removed quiz symbols
  - Ask the user if any questions arise
