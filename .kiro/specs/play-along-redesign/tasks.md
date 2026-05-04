# Implementation Plan: Play-Along Redesign

## Overview

Replace the BPM-timer-based play-along mode with a wait-based, beginner-friendly model. The
interval beat timer in `PlayAlongPanel` is removed and replaced with a chord-detection effect that
watches `held_notes` and advances only when the correct chord has been held for 300 ms. A
`Hand_Position_Overlay` (finger indicator circles) is added inside the existing `PianoPanel` keys.
The progression loops indefinitely. The per-chord result list and all scoring infrastructure are
deleted.

Pure-Rust logic is tested with `cargo test`. Browser-dependent behavior is manually verified.

## Tasks

- [ ] 1. Simplify `PlayAlongState` and remove scoring types in `src/state/mod.rs` and `src/midi/mod.rs`
  - In `src/state/mod.rs`: replace `PlayAlongState` fields with `progression_id`, `current_chord_index`, `chords_played: u32`, `showing_loop_cue: bool` — remove `score`, `started_at_ms`, `pre_play_along_metronome_active`
  - In `src/midi/mod.rs`: delete `PlayAlongScore` struct and `ChordResult` struct (verify no other code references them; remove references in `src/midi/tests.rs` too)
  - Update `AppAction`: remove `PlayAlongTick` and `RecordPlayAlongChordResult(ChordResult)` variants; add `PlayAlongChordCorrect` and `PlayAlongLoopCueDone`
  - _Requirements: 1.1, 3.1, 4.4_
  - _Depends on: None_

- [ ] 2. Rewrite reducer arms for play-along actions in `src/state/mod.rs`
  - `EnterPlayAlong(id)`: guard on `midi_status == Connected` and `chord_count > 0`; initialize new `PlayAlongState` with `current_chord_index: 0`, `chords_played: 0`, `showing_loop_cue: false`; do NOT force `metronome_active`
  - `PlayAlongChordCorrect`: compute `next_index = (current + 1) % chord_count`; set `showing_loop_cue = true` when `next_index == 0` (wrapped); update `highlighted_chord` via `chord_highlight_at`; increment `chords_played`
  - `PlayAlongLoopCueDone`: set `showing_loop_cue = false` in `play_along_state`
  - `ExitPlayAlong`: set `app_mode = Normal`, `play_along_state = None` — do NOT restore `metronome_active` (no longer saved)
  - _Requirements: 1.1, 3.1, 3.2, 5.3_
  - _Depends on: 1_

- [ ]* 2.1 Write unit and property tests for play-along reducer actions
  - Unit: `PlayAlongChordCorrect` from index 0 in 4-chord progression → index 1, `showing_loop_cue = false`
  - Unit: `PlayAlongChordCorrect` from index 3 (last) in 4-chord progression → index 0, `showing_loop_cue = true`
  - Unit: `PlayAlongLoopCueDone` → `showing_loop_cue = false`
  - Unit: `ExitPlayAlong` → `app_mode == Normal`, `play_along_state == None`
  - Unit: `EnterPlayAlong` when `midi_status != Connected` → state unchanged
  - **Property 3**: for any progression length N and any current_index == N-1, `PlayAlongChordCorrect` always produces `current_chord_index == 0` and `showing_loop_cue == true`
  - **Property 4**: for any succession of `PlayAlongChordCorrect` dispatches, `app_mode` remains `PlayAlong`
  - **Property 6**: `ExitPlayAlong` from any play-along state always produces `app_mode == Normal` and `play_along_state == None`
  - _Validates: Requirements 3.1, 3.2, 5.3_
  - _Depends on: 2_

- [ ] 3. Implement `chord_fully_held` in `src/components/play_along_panel.rs`
  - Add `pub fn chord_fully_held(target: &[PitchClass], held: &[HeldNote]) -> bool` — collect distinct PitchClasses from `held`, return `target.iter().all(|pc| held_pcs.contains(pc))`
  - Empty target returns `true` (vacuous)
  - _Requirements: 1.1, 1.5_
  - _Depends on: None_

- [ ]* 3.1 Write unit and property tests for `chord_fully_held`
  - Unit: all 3 target PCs present in any octave → true
  - Unit: one target PC missing → false
  - Unit: empty target → true
  - Unit: empty held → false (non-empty target)
  - **Property 1**: for any target PitchClasses P1..Pn and held notes that each contain at least one matching PC per target in any octave, `chord_fully_held` returns true
  - **Property 5**: `chord_fully_held(&[], _)` always true for any held notes
  - _Validates: Requirements 1.1, 1.5_
  - _Depends on: 3_

- [ ] 4. Rewrite `PlayAlongPanel` component in `src/components/play_along_panel.rs`
  - Remove `Interval` import and all timer-based beat logic
  - New props: `progression`, `current_chord_index`, `chords_played`, `showing_loop_cue`, `held_notes`, `on_stop`, `on_chord_correct: Callback<()>`, `on_loop_cue_done: Callback<()>` — remove `bpm`, `score`, `on_tick`, `on_record_result`
  - Add `use_mut_ref::<Option<gloo_timers::callback::Timeout>>` for the 300 ms debounce
  - Add `use_effect_with((held_notes, current_chord_index), ...)`: cancel pending timeout; if `chord_fully_held` → start 300 ms `Timeout` dispatching `on_chord_correct`; cleanup drops timeout
  - Add `use_effect_with(showing_loop_cue, ...)`: when true → start 1.5 s `Timeout` dispatching `on_loop_cue_done`
  - Render: chord name + Roman numeral header, "Chord N of M" position indicator, loop cue banner (when `showing_loop_cue`), Stop button — NO BPM control, NO result list
  - Pass `practice_target` (target PitchClasses) as existing prop to be forwarded to `PianoPanel` via `App`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 3.3, 3.4, 4.1, 4.2, 4.3_
  - _Depends on: 2, 3_

- [ ] 5. Add `FingerHint` type and `finger_hints_for_chord` function
  - In `src/components/piano_panel.rs`: define `pub struct FingerHint { pub pitch_class: PitchClass, pub finger: u8, pub held: bool }`
  - Add `pub fn finger_hints_for_chord(chord: &DiatonicChord, held: &[HeldNote]) -> Vec<FingerHint>` — zip `chord.notes[0..3]` with fingers `[1, 3, 5]`; set `held` by checking held PitchClasses (octave-agnostic)
  - Add `#[prop_or_default] pub finger_hints: Option<Vec<FingerHint>>` prop to `PianoPanelProps`
  - _Requirements: 2.1, 2.2_
  - _Depends on: None_

- [ ]* 5.1 Write unit tests for `finger_hints_for_chord`
  - Correct finger numbers (1, 3, 5) for root/third/fifth
  - `held` field true only for PitchClasses present in held notes, octave-agnostic
  - Works for both major and minor chords
  - _Validates: Requirements 2.2, 2.4_
  - _Depends on: 5_

- [ ] 6. Render `FingerHint` indicators inside `PianoPanel` keys
  - In `piano_panel` render loop: look up matching `FingerHint` by `pitch_class`
  - If found, render `<div class={cls}>{ hint.finger }</div>` inside the `piano-key` div, where `cls` is `"finger-hint"` or `"finger-hint finger-hint--held"` based on `hint.held`
  - Ensure `piano-key--white` has `position: relative` in `index.css` (black keys already use absolute positioning relative to parent)
  - _Requirements: 2.1, 2.3, 2.4, 2.5, 2.6_
  - _Depends on: 5_

- [ ] 7. Add CSS for finger hints and loop cue in `index.css`
  - Add `.finger-hint` styles: `position: absolute; top: -30px; left: 50%; transform: translateX(-50%); width: 24px; height: 24px; border-radius: 50%; display: flex; align-items: center; justify-content: center; font-size: 13px; font-weight: 700; pointer-events: none; z-index: 10` plus color vars
  - Add `.finger-hint--held` styles: held background color, `transform: translateX(-50%) scale(1.15)`
  - Add `.play-along__loop-cue` with the fade animation
  - Add `@keyframes loop-cue-fade`
  - Verify that `.piano-key--white` has `position: relative` (add if missing)
  - _Requirements: 2.3, 2.4, 3.3_
  - _Depends on: 6_

- [ ] 8. Wire new callbacks and finger hints through `App` in `src/components/app.rs`
  - Add `on_chord_correct` callback: `Callback::from(move |_| dispatch(AppAction::PlayAlongChordCorrect))`
  - Add `on_loop_cue_done` callback: `Callback::from(move |_| dispatch(AppAction::PlayAlongLoopCueDone))`
  - Remove `on_tick` and `on_record_result` callbacks and their `AppAction` dispatches
  - Compute `finger_hints`: when `app_mode == PlayAlong`, derive target `DiatonicChord` from `play_along_state.current_chord_index` and call `finger_hints_for_chord`; otherwise pass `None`
  - Pass `finger_hints` to `PianoPanel`
  - Add audio preview: `use_effect_with((play_along_state.map(|s| s.current_chord_index), app_mode), ...)` — when mode is PlayAlong, play the current target chord via `audio_engine.play_chord(notes)` unless muted
  - Wire `on_chord_correct` and `on_loop_cue_done` to `PlayAlongPanel`
  - Remove `bpm` prop from `PlayAlongPanel` (no longer used)
  - Add `use_effect_with(midi_status, ...)` that dispatches `ExitPlayAlong` when midi_status drops to non-Connected while in PlayAlong mode
  - _Requirements: 2.1, 5.5, 6.1, 6.2_
  - _Depends on: 4, 5, 6_

- [ ] 9. Checkpoint — run `cargo test` and ensure all tests pass
  - Run `cargo test` and fix any compilation errors from removed types/variants
  - Verify all existing tests still pass
  - Verify new unit and property tests pass
  - _Depends on: 2, 3, 5, 8_

- [ ] 10. Manual verification in browser
  - Build with `trunk serve` and connect a MIDI keyboard
  - Select a key and progression, click "Play Along"
  - Verify: Target_Chord and Roman numeral displayed; no BPM control visible; no result list appears
  - Verify: Finger_Indicators (1, 3, 5) appear above the correct piano keys
  - Verify: Pressing the correct chord causes a 300 ms pause then advances (finger indicators update)
  - Verify: Held indicators turn green/highlighted when note is pressed
  - Verify: Releasing a note before 300 ms cancels the advance
  - Verify: After the last chord, the loop cue flash appears and the progression returns to chord 1
  - Verify: Stop button exits cleanly; no finger hints after exit
  - Verify: Audio preview plays on each chord advance (when unmuted)
  - Verify: Disconnecting MIDI mid-session exits play-along mode
  - _Depends on: 8, 7_

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Property tests use `proptest` with tag format `// Feature: play-along-redesign, Property N: <text>`
- Tasks 3 and 5 have no dependencies and can begin immediately in parallel
- Tasks 2 and 3.1/5.1 can run in parallel once their respective parents are done
- Task 4 requires task 2 (new reducer) and task 3 (chord_fully_held function)
- Tasks 6, 7, 8 can begin once tasks 4 and 5 are done
- Task 9 gates on all implementation tasks; run before manual testing
