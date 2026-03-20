# Remove Quiz Mode Bugfix Design

## Overview

Quiz-related code is entangled across five source files and one standalone component file. The fix is a pure deletion: no new logic is introduced. Every quiz symbol is removed from `src/components/quiz_panel.rs` (deleted), `src/components/mod.rs`, `src/state/mod.rs`, `src/storage/mod.rs`, `src/components/nav_bar.rs`, and `src/components/app.rs`. All other features — circle, key info, progressions, piano, audio, MIDI, metronome, practice, play-along, theme, and storage — are left completely intact.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug — the compiled codebase contains any quiz symbol (see formal specification below)
- **Property (P)**: The desired state after the fix — zero quiz symbols remain anywhere in the source tree and `cargo test` passes
- **Preservation**: All non-quiz behavior (MIDI, metronome, storage, nav bar controls, circle rendering) that must remain identical before and after the fix
- **isBugCondition**: Pseudocode predicate that returns `true` when the codebase still contains quiz artifacts
- **quiz symbol**: Any of `QuizPanel`, `QuestionType`, `Question`, `BestScores`, `SessionResult`, `quiz_active`, `EnterQuiz`, `ExitQuiz`, `RecordQuizResult`, `KEY_BEST_SCORES`, `serialize_best_scores`, `deserialize_best_scores`, `on_enter_quiz`, `pub mod quiz_panel`
- **AppState**: Top-level Yew reducer state in `src/state/mod.rs`
- **PersistedState**: localStorage-backed struct in `src/storage/mod.rs`
- **NavBarProps**: Yew properties struct for the nav bar component

## Bug Details

### Bug Condition

The bug manifests whenever the codebase is compiled: quiz symbols are present as first-class citizens entangled with core app code, adding dead weight to the state shape, polluting the reducer, persisting unused data, and exposing a UI entry point for a feature that should be absent.

**Formal Specification:**
```
FUNCTION isBugCondition(X)
  INPUT: X — the compiled source tree
  OUTPUT: boolean

  RETURN X contains ANY of:
    file  src/components/quiz_panel.rs
    token "pub mod quiz_panel"          in src/components/mod.rs
    type  QuestionType                  in src/state/mod.rs
    type  Question                      in src/state/mod.rs
    type  BestScores                    in src/state/mod.rs
    type  SessionResult                 in src/state/mod.rs
    field quiz_active                   in AppState
    field best_scores                   in AppState
    variant EnterQuiz                   in AppAction
    variant ExitQuiz                    in AppAction
    variant RecordQuizResult            in AppAction
    const KEY_BEST_SCORES               in src/storage/mod.rs
    field best_scores                   in PersistedState
    fn serialize_best_scores            in src/storage/mod.rs
    fn deserialize_best_scores          in src/storage/mod.rs
    prop on_enter_quiz                  in NavBarProps
    button "Quiz Mode"                  in NavBar HTML
    import QuizPanel                    in src/components/app.rs
    import SessionResult                in src/components/app.rs
    callback on_enter_quiz              in app.rs
    callback on_quiz_exit               in app.rs
    callback on_session_end             in app.rs
    field s.best_scores                 in app.rs state init
    dep state.best_scores.clone()       in use_effect_with persistence
    prop on_enter_quiz={on_enter_quiz}  in <NavBar> usage
    branch else if state.quiz_active    in app.rs render
END FUNCTION
```

### Examples

- **Before fix**: `cargo grep QuizPanel` returns hits in `app.rs`, `components/mod.rs`, and `quiz_panel.rs`. After fix: zero hits.
- **Before fix**: `AppState::default()` allocates `quiz_active: false` and `best_scores: BestScores::default()`. After fix: neither field exists.
- **Before fix**: `save_state()` calls `ls_set(KEY_BEST_SCORES, ...)`. After fix: that call is gone.
- **Before fix**: The nav bar renders a `<button class="nav-bar__btn--quiz">Quiz Mode</button>`. After fix: that element is absent.

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- `cargo test` continues to pass all remaining tests (MIDI reducer property tests, storage round-trip tests, music theory tests, circle-of-fifths reducer tests)
- `AppState::default()` still compiles and contains `metronome_active`, `bpm`, `muted`, `theme`, `favorites`, `selected_key`, `app_mode`, `practice_state`, `play_along_state`, and all MIDI fields
- `PersistedState` still contains `theme`, `muted`, `favorites`, and `metronome_active`; `load_state()` and `save_state()` still function correctly for those fields
- `NavBarProps` still contains `bpm`, `on_set_bpm`, `metronome_active`, `on_toggle_metronome`, `on_enter_practice`, `midi_status`, `on_toggle_theme`, `on_toggle_mute`, `selected_key`, `theme`
- The nav bar still renders the BPM slider, theme toggle, mute toggle, metronome toggle, and the conditional Practice button
- All `AppAction` variants unrelated to quiz (`SelectKey`, `ToggleMute`, `ToggleTheme`, `MidiNoteOn`, `EnterPractice`, `EnterPlayAlong`, `ToggleMetronome`, etc.) continue to produce identical state transitions

**Scope:**
All inputs that do NOT involve quiz symbols should be completely unaffected by this fix. This includes:
- MIDI note-on/note-off handling
- Metronome scheduling and persistence
- Practice and play-along mode transitions
- Theme and mute toggling
- Favorites persistence
- Circle segment selection and chord highlighting

## Hypothesized Root Cause

This is not a logic bug — it is a structural/cleanliness issue. The root cause is that quiz functionality was implemented directly inside the core app modules rather than as an isolated feature:

1. **State pollution**: `QuestionType`, `Question`, `BestScores`, `SessionResult` were added to `src/state/mod.rs` alongside core types, and `quiz_active`/`best_scores` were added to `AppState` and `AppState::default()`.

2. **Reducer pollution**: `EnterQuiz`, `ExitQuiz`, and `RecordQuizResult` arms were added to the central `app_reducer` function, which handles all state transitions.

3. **Storage pollution**: `KEY_BEST_SCORES`, `best_scores` field in `PersistedState`, and `serialize_best_scores`/`deserialize_best_scores` helpers were added to `src/storage/mod.rs`, causing quiz data to be written to localStorage on every save.

4. **Component surface pollution**: `pub mod quiz_panel` was added to `src/components/mod.rs`, and `src/components/quiz_panel.rs` was created as a full Yew component with its own test suite.

5. **App wiring pollution**: `src/components/app.rs` imports `QuizPanel` and `SessionResult`, creates three quiz callbacks, reads `persisted.best_scores` during state init, includes `best_scores` in the persistence `use_effect_with` dependency tuple, passes `on_enter_quiz` to `<NavBar>`, and renders a `<QuizPanel>` branch.

6. **Nav bar pollution**: `NavBarProps` includes `on_enter_quiz: Callback<()>` and the rendered HTML includes a "Quiz Mode" button.

## Correctness Properties

Property 1: Bug Condition - Quiz Symbols Absent After Fix

_For any_ version of the source tree where `isBugCondition` returns `true` (quiz symbols are present), after applying the fix, `isBugCondition` SHALL return `false`: no quiz symbol from the enumerated list appears anywhere in the source tree, and `cargo test` compiles and passes with zero errors and zero test failures.

**Validates: Requirements 2.1, 2.2, 2.3, 2.4, 2.5, 2.6**

Property 2: Preservation - Non-Quiz Behavior Unchanged

_For any_ behavior that does NOT involve quiz symbols (MIDI handling, metronome, storage of `theme`/`muted`/`favorites`/`metronome_active`, nav bar controls, circle rendering, practice/play-along modes), the fixed codebase SHALL produce exactly the same behavior as the original codebase, preserving all existing test outcomes and runtime behavior for non-quiz interactions.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4, 3.5, 3.6, 3.7**

## Fix Implementation

### Changes Required

**File**: `src/components/quiz_panel.rs`
**Action**: Delete the entire file.

---

**File**: `src/components/mod.rs`
**Changes**:
1. Remove the line `pub mod quiz_panel;`

---

**File**: `src/state/mod.rs`
**Changes**:
1. **Remove types**: Delete `QuestionType` enum, `Question` struct, `BestScores` struct, `SessionResult` struct (the entire "Quiz / app-level types" section header and its contents, keeping `Theme` which is not quiz-specific)
2. **Remove from `AppState`**: Delete fields `quiz_active: bool` and `best_scores: BestScores`
3. **Remove from `AppState::default()`**: Delete `quiz_active: false` and `best_scores: BestScores::default()`
4. **Remove from `AppAction`**: Delete variants `EnterQuiz`, `ExitQuiz`, `RecordQuizResult(SessionResult)`
5. **Remove reducer arms**: Delete the `AppAction::EnterQuiz`, `AppAction::ExitQuiz`, and `AppAction::RecordQuizResult` match arms from `app_reducer`
6. **Remove tests**: Delete `enter_quiz_sets_quiz_active` and `exit_quiz_clears_quiz_active` test functions

---

**File**: `src/storage/mod.rs`
**Changes**:
1. **Remove import**: Remove `BestScores` from the `use crate::state::{...}` import
2. **Remove constant**: Delete `KEY_BEST_SCORES` constant
3. **Remove from `PersistedState`**: Delete `best_scores: BestScores` field
4. **Remove from `PersistedState::default()`**: Delete `best_scores: BestScores::default()`
5. **Remove functions**: Delete `serialize_best_scores()` and `deserialize_best_scores()` functions
6. **Remove from `load_state()`**: Delete the `best_scores` reading logic and remove `best_scores` from the returned `PersistedState` struct literal
7. **Remove from `save_state()`**: Delete the `ls_set(KEY_BEST_SCORES, ...)` call
8. **Remove tests**: Delete `best_scores_round_trip_default`, `best_scores_round_trip_with_values`, and `deserialize_best_scores_invalid_json_falls_back_to_default` test functions; also remove the `use crate::state::BestScores;` import from the test module if it becomes unused; update `load_state_returns_defaults_in_native_target` to remove the `best_scores` assertion

---

**File**: `src/components/nav_bar.rs`
**Changes**:
1. **Remove from `NavBarProps`**: Delete `on_enter_quiz: Callback<()>` field
2. **Remove from component body**: Delete `let on_enter_quiz = props.on_enter_quiz.reform(|_: MouseEvent| ());`
3. **Remove from rendered HTML**: Delete the `<button class="nav-bar__btn nav-bar__btn--quiz" onclick={on_enter_quiz}>{ "Quiz Mode" }</button>` element

---

**File**: `src/components/app.rs`
**Changes**:
1. **Remove import**: Delete `use crate::components::quiz_panel::QuizPanel;`
2. **Remove from state import**: Remove `SessionResult` from `use crate::state::{...}`
3. **Remove from state init**: Delete `s.best_scores = persisted.best_scores;`
4. **Remove from persistence deps**: Remove `state.best_scores.clone()` from the `use_effect_with` tuple
5. **Remove callbacks**: Delete `on_enter_quiz`, `on_quiz_exit`, and `on_session_end` callback definitions
6. **Remove from `<NavBar>` usage**: Delete `on_enter_quiz={on_enter_quiz}` prop
7. **Remove render branch**: Delete the `else if state.quiz_active { <QuizPanel ... /> }` branch

## Testing Strategy

### Validation Approach

The testing strategy follows a two-phase approach: first, surface counterexamples that demonstrate the bug on unfixed code (confirm quiz symbols are present), then verify the fix is complete (confirm zero quiz symbols remain and all tests pass).

### Exploratory Bug Condition Checking

**Goal**: Confirm that quiz symbols are present in the unfixed codebase before applying the fix. This establishes the baseline that `isBugCondition` returns `true`.

**Test Plan**: Run `cargo test` on the unfixed code and observe that it compiles with all quiz symbols present. Use `grep` or `cargo check` to confirm the presence of each symbol in the enumerated list.

**Test Cases**:
1. **Symbol presence check**: Confirm `QuizPanel` is importable from `crate::components::quiz_panel` (will fail after fix)
2. **State field check**: Confirm `AppState::default().quiz_active` compiles (will fail after fix)
3. **Storage field check**: Confirm `PersistedState::default().best_scores` compiles (will fail after fix)
4. **Nav bar prop check**: Confirm `NavBarProps` has `on_enter_quiz` field (will fail after fix)

**Expected Counterexamples**:
- All four checks pass on unfixed code, confirming `isBugCondition` is `true`
- After fix, all four checks fail to compile, confirming `isBugCondition` is `false`

### Fix Checking

**Goal**: Verify that after applying the fix, no quiz symbol remains in the source tree and `cargo test` passes.

**Pseudocode:**
```
FOR ALL symbol IN quiz_symbol_list DO
  result := grep(symbol, src/)
  ASSERT result.matches = 0
END FOR
ASSERT cargo_test() = PASS
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold (non-quiz behavior), the fixed codebase produces the same results as the original.

**Pseudocode:**
```
FOR ALL test IN existing_non_quiz_tests DO
  ASSERT test passes on fixed codebase
END FOR
```

**Testing Approach**: The existing test suite is the preservation oracle. Because this fix is a pure deletion with no new logic, passing all pre-existing non-quiz tests is sufficient to confirm preservation. Property-based tests in `src/state/mod.rs` (MIDI reducer properties) and `src/storage/mod.rs` (round-trip tests) provide strong coverage.

**Test Cases**:
1. **MIDI reducer preservation**: All `prop_note_on_off_round_trip`, `prop_velocity_zero_is_note_off`, `prop_clear_rolling_window`, `prop_empty_devices_clears_held_notes`, `prop_bpm_clamping`, `prop_exit_play_along_restores_metronome` tests continue to pass
2. **Storage preservation**: `theme_round_trip_dark`, `theme_round_trip_light`, `muted_round_trip_*`, `favorites_round_trip_*`, `metronome_active_round_trip_*`, `load_state_returns_defaults_in_native_target` tests continue to pass
3. **State reducer preservation**: `select_key_sets_selected_key`, `select_key_twice_deselects`, `favorite_toggle_round_trip`, `mute_toggle_round_trip`, `theme_toggle_round_trip`, `octave_clamp_*` tests continue to pass
4. **NavBar props preservation**: `NavBarProps` still accepts `bpm`, `on_set_bpm`, `metronome_active`, `on_toggle_metronome`, `on_enter_practice`, `midi_status`

### Unit Tests

- Verify `AppState::default()` compiles without `quiz_active` or `best_scores` fields
- Verify `PersistedState::default()` compiles without `best_scores` field
- Verify `AppAction` enum has no `EnterQuiz`, `ExitQuiz`, or `RecordQuizResult` variants
- Verify `NavBarProps` has no `on_enter_quiz` field

### Property-Based Tests

- Existing `proptest!` blocks in `src/state/mod.rs` cover MIDI reducer behavior across random inputs — all must continue to pass after the fix
- Existing storage round-trip tests cover `theme`, `muted`, `favorites`, `metronome_active` serialization — all must continue to pass

### Integration Tests

- `cargo test` (native target) must exit 0 with zero test failures after the fix
- `cargo check` (or `cargo build`) must succeed with zero compilation errors after the fix
- No quiz symbol from the enumerated list in the Bug Condition section should appear in any `grep` search of `src/`
