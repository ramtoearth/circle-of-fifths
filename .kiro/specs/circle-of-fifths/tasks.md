# Implementation Plan: Circle of Fifths

## Overview

Implement a fully static Rust/WASM single-page application using Yew + Trunk. The build follows a bottom-up approach: pure data and music theory logic first, then the state reducer, then persistence, then UI components, then audio, then quiz, and finally wiring everything together.

Tasks are grouped so that independent work streams are visible. Where tasks have no dependencies on each other they can be executed in parallel.

---

## Dependency Key

- **None** — can start immediately (no prior task required)
- **Task N** — must wait for task N to be fully complete

---

## Tasks

- [x] 1. Project scaffold and Trunk configuration
  - Initialize a new Yew + Trunk project (`cargo new --lib`, add `Cargo.toml` deps: `yew`, `web-sys`, `wasm-bindgen`, `proptest`, `serde`, `serde_json`, `gloo-storage`)
  - Add `index.html` and `Trunk.toml`
  - Set up `src/` module tree: `music_theory/`, `state/`, `data/`, `audio/`, `storage/`, `components/`
  - _Requirements: all (foundational)_
  - _Depends on: None_

- [x] 2. Core data models
  - Define all structs and enums from the design: `PitchClass`, `Mode`, `Key`, `ScaleDegree`, `ChordQuality`, `DiatonicChord`, `Progression`, `BorrowedChord`, `ActiveProgression`, `ProgressionTag`, `ChordHighlight`, `KeyRole`, `QuestionType`, `Question`, `BestScores`, `AppState`, `AppAction`, `Theme`
  - Place in `src/music_theory/mod.rs` and `src/state/mod.rs` as appropriate
  - Derive `Clone`, `Copy`, `PartialEq`, `Eq`, `Hash`, `serde::Serialize/Deserialize` where needed
  - _Requirements: all (foundational types)_
  - _Depends on: 1_

- [x] 3. Music theory pure functions
  - Implement `diatonic_chords(key: Key) -> [DiatonicChord; 7]` using the W-W-H-W-W-W-H formula
  - Implement `key_signature(key: Key) -> KeySignature` (sharps/flats count + names)
  - Implement `scale_notes(key: Key) -> [PitchClass; 7]`
  - Implement `relative_minor(major: Key) -> Key` and `relative_major(minor: Key) -> Key`
  - Implement `adjacent_keys(key: Key) -> (Key, Key)` and `opposite_key(key: Key) -> Key`
  - Implement Roman numeral formatting helper
  - Place in `src/music_theory/mod.rs`
  - _Requirements: 1.4, 1.5, 2.1, 2.2, 2.3, 3.1, 3.2_
  - _Depends on: 2_

  - [x]* 3.1 Write property test for circle geometry (Property 3)
    - **Property 3: Circle geometry correctness**
    - **Validates: Requirements 1.4, 1.5**
    - _Depends on: 3_

  - [x]* 3.2 Write property test for diatonic chord correctness (Property 4)
    - **Property 4: Diatonic chord correctness**
    - **Validates: Requirements 2.2, 3.1, 3.2**
    - _Depends on: 3_

  - [x]* 3.3 Write property test for chord display format (Property 5)
    - **Property 5: Chord display format**
    - **Validates: Requirements 2.3**
    - _Depends on: 3_

  - [x]* 3.4 Write unit tests for music theory functions
    - Known key signatures (C=0, G=1♯, F=1♭), known diatonic chord names for C major
    - _Requirements: 2.1, 3.1_
    - _Depends on: 3_

- [x] 4. Static progression data
  - Define all progressions as `static` arrays in `src/data/mod.rs`
  - At least 4 progressions per key, covering ≥3 distinct `ProgressionTag` values
  - At least 1 progression per key with a non-None `borrowed_chord`
  - Include resolved chord name lookup helper
  - _Requirements: 4.1, 4.2, 4.6, 4.7_
  - _Depends on: 2, 3_

  - [x]* 4.1 Write property test for progression data invariants (Property 7)
    - **Property 7: Progression data invariants**
    - **Validates: Requirements 4.1, 4.6, 4.7**
    - _Depends on: 4_

  - [x]* 4.2 Write property test for progression display format (Property 8)
    - **Property 8: Progression display format**
    - **Validates: Requirements 4.2**
    - _Depends on: 4_

- [x] 5. App state reducer
  - Implement `app_reducer(state: AppState, action: AppAction) -> AppState` in `src/state/mod.rs`
  - Handle all `AppAction` variants: `SelectKey`, `DeselectKey`, `SelectChord`, `SelectProgression`, `NextChord`, `PrevChord`, `ToggleFavorite`, `ToggleNoteLabels`, `ShiftOctave`, `ToggleTheme`, `ToggleMute`, `EnterQuiz`, `ExitQuiz`, `RecordQuizResult`
  - Clamp `octave_offset` to valid range (-2..=2)
  - No-op guard for `NextChord`/`PrevChord` when no active progression
  - _Requirements: 1.2, 1.6, 3.3, 4.3, 4.4, 4.5, 5.5, 5.6, 6.1, 7.7, 8.2_
  - _Depends on: 2, 3, 4_

  - [x]* 5.1 Write property test for segment selection (Property 1)
    - **Property 1: Segment selection state transition**
    - **Validates: Requirements 1.2**
    - _Depends on: 5_

  - [x]* 5.2 Write property test for segment deselection round-trip (Property 2)
    - **Property 2: Segment deselection round-trip**
    - **Validates: Requirements 1.6**
    - _Depends on: 5_

  - [x]* 5.3 Write property test for chord click updates piano highlight (Property 6)
    - **Property 6: Chord click updates piano highlight**
    - **Validates: Requirements 3.3, 5.3**
    - _Depends on: 5_

  - [x]* 5.4 Write property test for progression activation sets first chord (Property 9)
    - **Property 9: Progression activation sets first chord**
    - **Validates: Requirements 4.3**
    - _Depends on: 5_

  - [x]* 5.5 Write property test for progression navigation round-trip (Property 10)
    - **Property 10: Progression navigation round-trip**
    - **Validates: Requirements 4.4**
    - _Depends on: 5_

  - [x]* 5.6 Write property test for favorite toggle round-trip (Property 11)
    - **Property 11: Favorite toggle round-trip**
    - **Validates: Requirements 4.5**
    - _Depends on: 5_

  - [x]* 5.7 Write property test for note label toggle idempotence (Property 13)
    - **Property 13: Note label toggle idempotence**
    - **Validates: Requirements 5.5**
    - _Depends on: 5_

  - [x]* 5.8 Write property test for octave shift round-trip (Property 14)
    - **Property 14: Octave shift round-trip**
    - **Validates: Requirements 5.6**
    - _Depends on: 5_

  - [x]* 5.9 Write property test for mute toggle round-trip (Property 20)
    - **Property 20: Mute toggle round-trip**
    - **Validates: Requirements 7.7**
    - _Depends on: 5_

  - [x]* 5.10 Write property test for theme toggle round-trip (Property 21)
    - **Property 21: Theme toggle round-trip**
    - **Validates: Requirements 8.2**
    - _Depends on: 5_

  - [x]* 5.11 Write unit tests for reducer state transitions
    - Quiz mode entry/exit, octave clamp boundary, no-op navigation guard
    - _Requirements: 5.6, 6.1_
    - _Depends on: 5_

- [x] 6. localStorage persistence layer
  - Implement `load_state() -> PersistedState` and `save_state(state: &AppState)` in `src/storage/mod.rs`
  - Serialize/deserialize `theme`, `muted`, `favorites`, `best_scores` using the schema from the design
  - Fail silently if `localStorage` is unavailable; fall back to defaults on deserialization error
  - _Requirements: 4.5, 6.7, 7.8, 8.3_
  - _Depends on: 2_

  - [x]* 6.1 Write property test for localStorage round-trip (Property 18)
    - **Property 18: localStorage round-trip**
    - **Validates: Requirements 4.5, 6.7, 7.8, 8.3**
    - _Depends on: 6_

  - [x]* 6.2 Write unit tests for storage error handling
    - Deserialization failure falls back to defaults; unavailable storage does not panic
    - _Requirements: 4.5, 8.3_
    - _Depends on: 6_

- [x] 7. Checkpoint — pure logic complete
  - Ensure all `cargo test` tests pass (music theory, reducer, data, storage)
  - Ask the user if questions arise before proceeding to UI components.
  - _Depends on: 3, 4, 5, 6_

- [ ] 8. CircleView SVG component
  - Implement `CircleView` in `src/components/circle_view.rs`
  - Render 24 `<path>` arc segments (12 major outer, 12 minor inner) as inline SVG
  - Highlight selected segment, adjacent segments, and opposite segment with distinct CSS classes
  - Display key signature accidental count on each segment
  - Emit `on_segment_click(key: Key)` callback; clicking selected segment dispatches `DeselectKey`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6_
  - _Depends on: 2, 3, 5_

- [ ] 9. KeyInfoPanel component
  - Implement `KeyInfoPanel` in `src/components/key_info_panel.rs`
  - Show key name, key signature (count + note names), seven scale notes
  - List all 7 diatonic chords with Roman numeral + full chord name
  - Show placeholder prompt when `selected_key` is `None`
  - Emit `on_chord_click(chord: DiatonicChord)` callback
  - _Requirements: 2.1, 2.2, 2.3, 2.4_
  - _Depends on: 2, 3_

- [x] 10. ProgressionPanel component
  - Implement `ProgressionPanel` in `src/components/progression_panel.rs`
  - List progressions for selected key with tag labels
  - Display Roman numeral sequence and resolved chord names for each progression
  - Show borrowed chord label when present
  - Favorite toggle button per progression
  - Next/prev controls when a progression is active
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 4.7_
  - _Depends on: 2, 4, 5_

- [ ] 11. PianoPanel component
  - Implement `PianoPanel` in `src/components/piano_panel.rs`
  - Render scrollable horizontal keyboard spanning ≥3 octaves (≥36 keys)
  - Highlight scale notes and chord notes color-coded by `KeyRole` (root/third/fifth)
  - Toggle note name labels on/off
  - Octave range selector (+1/-1 shift)
  - Auto-scroll when highlighted notes fall outside visible range
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7_
  - _Depends on: 2, 3, 5_

  - [ ]* 11.1 Write property test for piano scale highlight correctness (Property 12)
    - **Property 12: Piano scale highlight correctness**
    - **Validates: Requirements 5.2**
    - _Depends on: 11_

  - [ ]* 11.2 Write unit test for piano key count
    - Verify rendered keyboard contains ≥36 keys for 3-octave range
    - _Requirements: 5.1_
    - _Depends on: 11_

- [ ] 12. NavBar component
  - Implement `NavBar` in `src/components/nav_bar.rs`
  - Theme toggle button (dispatches `ToggleTheme`)
  - Mute toggle button (dispatches `ToggleMute`)
  - Quiz mode entry button (dispatches `EnterQuiz`)
  - _Requirements: 6.1, 7.7, 8.1, 8.2_
  - _Depends on: 2, 5_

- [ ] 13. QuizPanel component
  - Implement `QuizPanel` in `src/components/quiz_panel.rs`
  - Full-screen modal/page with local session state (current question index, score)
  - Generate shuffled question pool covering all three `QuestionType` variants
  - Display question, accept answer input, show correct/incorrect feedback with correct answer reveal
  - Running score display during session; summary screen on session end
  - Exit button dispatches `ExitQuiz`; session end dispatches `RecordQuizResult`
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  - _Depends on: 2, 3, 5_

  - [ ]* 13.1 Write property test for question pool completeness and shuffle (Property 15)
    - **Property 15: Question pool completeness and shuffle**
    - **Validates: Requirements 6.2, 6.3**
    - _Depends on: 13_

  - [ ]* 13.2 Write property test for answer evaluation correctness (Property 16)
    - **Property 16: Answer evaluation correctness**
    - **Validates: Requirements 6.4**
    - _Depends on: 13_

  - [ ]* 13.3 Write property test for score tracking invariant (Property 17)
    - **Property 17: Score tracking invariant**
    - **Validates: Requirements 6.5, 6.6**
    - _Depends on: 13_

  - [ ]* 13.4 Write unit tests for quiz state transitions
    - Entry/exit transitions, answer submission, session summary
    - _Requirements: 6.1, 6.4, 6.5, 6.6_
    - _Depends on: 13_

- [ ] 14. AudioEngine
  - Implement `AudioEngine` struct in `src/audio/mod.rs` wrapping `web_sys::AudioContext`
  - `play_scale(key)`: schedule notes at 300ms intervals
  - `play_chord(notes)`: play all notes simultaneously
  - `play_progression(progression)`: play each chord for 1 second in sequence
  - `stop()` and `set_muted(bool)`
  - Degrade gracefully if `AudioContext::new()` fails; set `audio_error` in `AppState`
  - Expose as Yew context
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 7.8_
  - _Depends on: 2, 3_

  - [ ]* 14.1 Write property test for audio note sequence correctness (Property 19)
    - **Property 19: Audio note sequence correctness**
    - **Validates: Requirements 7.1, 7.2, 7.3**
    - _Depends on: 14_

  - [ ]* 14.2 Write unit tests for audio engine degraded mode
    - Initialization failure sets `audio_error`, all non-audio features remain functional
    - _Requirements: 7.5_
    - _Depends on: 14_

- [ ] 15. Checkpoint — all components built
  - Ensure all `cargo test` and `wasm-pack test` tests pass
  - Ask the user if questions arise before proceeding to wiring.
  - _Depends on: 8, 9, 10, 11, 12, 13, 14_

- [ ] 16. Root App component and wiring
  - Implement `App` in `src/components/app.rs`
  - Initialize `use_reducer` with `AppState` (seeded from `load_state()`)
  - Wire `use_effect` hooks to sync persisted fields to `localStorage` on state change
  - Wire `use_effect` hook to sync `AudioEngine` mute state from `AppState`
  - Compose `NavBar`, `CircleView`, `KeyInfoPanel`, `ProgressionPanel`, `PianoPanel`, `QuizPanel`, and `AudioEngine` context provider
  - Apply theme CSS class to root element
  - Display audio error banner when `audio_error` is `Some`
  - _Requirements: 1.2, 4.5, 7.4, 7.5, 7.8, 8.1, 8.3, 8.4_
  - _Depends on: 5, 6, 8, 9, 10, 11, 12, 13, 14_

- [ ] 17. CSS and theme styling
  - Write `index.css` with dark and light theme variables
  - Style all components: Circle segments (selected/adjacent/opposite states), KeyInfoPanel, ProgressionPanel, PianoPanel (key roles color-coded), NavBar, QuizPanel
  - Minimal design, no ads or third-party promotional content
  - _Requirements: 1.3, 1.5, 5.3, 8.1, 8.4, 8.5, 8.6_
  - _Depends on: 8, 9, 10, 11, 12, 13_

- [ ] 18. Final checkpoint — full integration
  - Ensure all `cargo test` and `wasm-pack test --headless --firefox` tests pass
  - Verify `trunk build` produces a working static bundle
  - Ask the user if questions arise.
  - _Depends on: 16, 17_

---

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Tasks 3, 4, 5, and 6 depend only on task 2 and can be worked in parallel with each other after task 2 is done
- Tasks 8, 9, 10, 11, 12, 13, and 14 all depend on tasks 2/3/5 but are independent of each other and can be built in parallel
- Task 17 (CSS) can be worked in parallel with task 16 (wiring) once the component shells exist
- Property tests are co-located with their parent implementation tasks for early error detection
- All property tests use `proptest` and run with `cargo test` (pure Rust) or `wasm-pack test` (WASM-dependent)
