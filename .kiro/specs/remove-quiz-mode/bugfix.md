# Bugfix Requirements Document

## Introduction

Quiz Mode is currently implemented across multiple layers of the Circle of Fifths app — state types, reducer arms, storage, a dedicated component, and the nav bar. This entanglement makes the codebase harder to maintain and blocks clean re-introduction of quiz functionality as a standalone feature later. The "bug" is that quiz-related code is woven into the core app in a way that violates separation of concerns: it adds dead weight to the state shape, pollutes the reducer, persists data that is no longer needed, and exposes a UI entry point for a feature that should be temporarily absent. The fix removes every quiz artifact while leaving all other features (circle, key info, progressions, piano, audio, MIDI, metronome, practice, play-along, theme, storage) completely intact.

## Bug Analysis

### Current Behavior (Defect)

1.1 WHEN the codebase is compiled, THEN the system includes `QuizPanel`, `QuestionType`, `Question`, `BestScores`, `SessionResult`, `quiz_active`, `EnterQuiz`, `ExitQuiz`, and `RecordQuizResult` as first-class symbols entangled with core app code

1.2 WHEN `AppState` is constructed, THEN the system allocates `quiz_active: bool` and `best_scores: BestScores` fields that serve no purpose without quiz mode

1.3 WHEN the app persists state to localStorage, THEN the system writes a `cof_best_scores` key that stores quiz data no other feature reads

1.4 WHEN the nav bar renders, THEN the system displays a "Quiz Mode" button that navigates to a removed feature

1.5 WHEN `app.rs` initializes state, THEN the system reads `persisted.best_scores` and wires `on_enter_quiz`, `on_quiz_exit`, and `on_session_end` callbacks that reference quiz-only types

1.6 WHEN `src/components/mod.rs` is compiled, THEN the system exposes `pub mod quiz_panel` as part of the public component surface

### Expected Behavior (Correct)

2.1 WHEN the codebase is compiled, THEN the system SHALL contain no references to `QuizPanel`, `QuestionType`, `Question`, `BestScores`, `SessionResult`, `quiz_active`, `EnterQuiz`, `ExitQuiz`, or `RecordQuizResult`

2.2 WHEN `AppState` is constructed, THEN the system SHALL NOT include `quiz_active` or `best_scores` fields

2.3 WHEN the app persists state to localStorage, THEN the system SHALL NOT write or read a `cof_best_scores` key

2.4 WHEN the nav bar renders, THEN the system SHALL NOT display a "Quiz Mode" button

2.5 WHEN `app.rs` initializes state, THEN the system SHALL NOT reference quiz callbacks or quiz-related persisted fields

2.6 WHEN `src/components/mod.rs` is compiled, THEN the system SHALL NOT declare `pub mod quiz_panel`

### Unchanged Behavior (Regression Prevention)

3.1 WHEN `cargo test` is run after the removal, THEN the system SHALL CONTINUE TO pass all remaining tests with no failures

3.2 WHEN the app is compiled and served, THEN the system SHALL CONTINUE TO render the circle, key info panel, progression panel, piano panel, MIDI status bar, and nav bar correctly

3.3 WHEN a user interacts with MIDI input, THEN the system SHALL CONTINUE TO handle note-on/note-off events, chord recognition, key suggestions, and practice/play-along modes without change

3.4 WHEN the metronome is toggled, THEN the system SHALL CONTINUE TO schedule clicks and persist `metronome_active` to localStorage

3.5 WHEN the app persists state, THEN the system SHALL CONTINUE TO save and restore `theme`, `muted`, `favorites`, and `metronome_active` via localStorage

3.6 WHEN the nav bar renders, THEN the system SHALL CONTINUE TO display the BPM slider, theme toggle, mute toggle, metronome toggle, and the conditional Practice button for MIDI-connected devices

3.7 WHEN `AppAction` variants unrelated to quiz are dispatched, THEN the system SHALL CONTINUE TO produce identical state transitions as before the removal

---

## Bug Condition

```pascal
FUNCTION isBugCondition(X)
  INPUT: X — the compiled codebase
  OUTPUT: boolean

  RETURN X contains any of:
    QuizPanel component file,
    QuestionType / Question / BestScores / SessionResult types,
    quiz_active field in AppState,
    EnterQuiz / ExitQuiz / RecordQuizResult variants in AppAction,
    KEY_BEST_SCORES constant or best_scores field in PersistedState,
    serialize_best_scores / deserialize_best_scores functions,
    on_enter_quiz prop in NavBarProps,
    "Quiz Mode" button in NavBar HTML,
    pub mod quiz_panel in components/mod.rs,
    quiz-related callbacks or render branch in app.rs
END FUNCTION
```

```pascal
// Property: Fix Checking
FOR ALL X WHERE isBugCondition(X) DO
  result ← compile_and_test(X after fix)
  ASSERT result.quiz_symbols_present = false
  ASSERT result.cargo_test_passes = true
END FOR

// Property: Preservation Checking
FOR ALL X WHERE NOT isBugCondition(X) DO
  ASSERT F(X) = F'(X)   // all non-quiz behavior is identical before and after
END FOR
```
