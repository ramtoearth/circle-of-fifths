# Implementation Plan: Metronome Time Signature

## Overview

Extend the metronome with time signature support (numerator/denominator), accent click on beat 1, denominator-aware beat intervals, and localStorage persistence. Changes span four layers: `state`, `audio`, `storage`, and `components`.

## Tasks

- [x] 1. Add `TimeSignature` struct and `SetTimeSignature` action to `state/mod.rs`
  - Define `TimeSignature { numerator: u32, denominator: u32 }` with `#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]`
  - Implement `TimeSignature::DEFAULT` (4/4), `validated(n, d) -> Option<Self>`, and `beat_interval_ms(bpm, d) -> u32`
  - Add `time_signature: TimeSignature` field to `AppState` (default `TimeSignature::DEFAULT`)
  - Add `AppAction::SetTimeSignature(u32, u32)` variant
  - Implement the reducer arm: validate via `TimeSignature::validated`; if invalid, return unchanged state
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.1_

  - [x]* 1.1 Write property test for valid numerator acceptance
    - **Property 1: Valid numerator acceptance**
    - **Validates: Requirements 1.2, 1.4**

  - [x]* 1.2 Write property test for valid denominator acceptance
    - **Property 2: Valid denominator acceptance**
    - **Validates: Requirements 1.3, 1.5**

  - [x]* 1.3 Write property test for beat interval formula correctness
    - **Property 8: Beat interval formula correctness**
    - **Validates: Requirements 5.1, 5.2, 5.3, 5.4**

  - [x]* 1.4 Write unit tests for `TimeSignature`
    - `DEFAULT` equals `{ 4, 4 }`
    - `validated` returns `None` for numerator 0 and 17
    - `validated` returns `None` for denominator 3, 5, 6, 7
    - `beat_interval_ms(120, 4)` == 500, `(120, 8)` == 250, `(120, 2)` == 1000
    - `SetTimeSignature(0, 4)` leaves state unchanged
    - `SetTimeSignature(4, 3)` leaves state unchanged
    - `SetTimeSignature(3, 8)` updates state to `{ 3, 8 }`
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 5.2, 5.3, 5.4_

- [x] 2. Add `schedule_metronome_click_accented` to `AudioEngine` and `AudioEngineHandle`
  - Add constants `ACCENT_FREQ: f32 = 1800.0` and `REGULAR_FREQ: f32 = 1200.0` in `audio/mod.rs`
  - Implement `AudioEngine::schedule_metronome_click_accented(&self, start: f64, is_accent: bool)` — same triangle oscillator / 30 ms / exponential decay as existing `schedule_metronome_click`, selecting frequency by `is_accent`
  - Add the corresponding `AudioEngineHandle::schedule_metronome_click_accented` delegate
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

  - [x]* 2.1 Write unit test for accent pitch higher than regular pitch
    - **Property 6: Accent pitch is higher than regular pitch**
    - Assert `ACCENT_FREQ > REGULAR_FREQ`
    - **Validates: Requirements 4.3**

  - [x]* 2.2 Write unit test for mute suppression
    - **Property 7: Mute suppresses all clicks**
    - Construct a degraded engine, set `muted = true`, call `schedule_metronome_click_accented`, assert no panic
    - **Validates: Requirements 4.5**

- [ ] 3. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [x] 4. Update `storage/mod.rs` to persist `time_signature`
  - Add `KEY_TIME_SIGNATURE: &str = "cof_time_signature"` constant
  - Add `time_signature: TimeSignature` field to `PersistedState` (default `TimeSignature::DEFAULT`)
  - Implement `serialize_time_signature(ts: TimeSignature) -> String` producing `"{n}/{d}"`
  - Implement `deserialize_time_signature(s: &str) -> TimeSignature` — parse `"n/d"`, validate via `TimeSignature::validated`, fall back to `DEFAULT` on any error or absence
  - Update `load_state` and `save_state` to read/write the new key
  - _Requirements: 1.6, 6.1, 6.2, 6.3, 6.4_

  - [x]* 4.1 Write property test for time signature serialization round-trip
    - **Property 3: Time signature serialization round-trip**
    - **Validates: Requirements 1.6, 6.1, 6.2**

  - [x]* 4.2 Write unit tests for storage helpers
    - Deserializing absent/empty string returns `DEFAULT`
    - Deserializing `"0/4"` returns `DEFAULT`
    - Deserializing `"4/3"` returns `DEFAULT`
    - `load_state` in non-WASM target returns `DEFAULT` for `time_signature`
    - _Requirements: 6.3, 6.4_

- [ ] 5. Update metronome interval effect in `app.rs`
  - Add `let beat_index = use_mut_ref(|| 0u32);` near the other `use_mut_ref` declarations
  - Extend the `use_effect_with` dependency tuple to include `state.time_signature`
  - At the start of the effect closure, reset `*beat_index.borrow_mut() = 0`
  - Compute `interval_ms` using `TimeSignature::beat_interval_ms(bpm, time_signature.denominator)`
  - In the `Interval` callback: read `beat_index`, call `audio.schedule_metronome_click_accented(start, *idx == 0)`, then increment and wrap modulo `numerator`
  - Update the `save_state` `use_effect_with` dependency tuple to include `state.time_signature`
  - Restore `time_signature` from `persisted` when initializing state in `use_reducer`
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 5.1, 5.5_

  - [ ]* 5.1 Write property test for beat index modular wrap
    - **Property 4: Beat index modular wrap**
    - **Validates: Requirements 3.1, 3.2, 3.5**

  - [ ]* 5.2 Write property test for accent selection correctness
    - **Property 5: Accent selection correctness**
    - **Validates: Requirements 4.1, 4.2**

- [x] 6. Add time signature controls to `NavBar`
  - Add `time_signature: TimeSignature` and `on_set_time_signature: Callback<(u32, u32)>` to `NavBarProps`
  - Add a numerator `<select>` with options 1–16 that emits `on_set_time_signature` with the new numerator and current denominator on change
  - Add a denominator `<select>` with options 1, 2, 4, 8, 16 that emits `on_set_time_signature` with the current numerator and new denominator on change
  - Add a read-only label displaying `"{numerator}/{denominator}"`
  - Wire `on_set_time_signature` callback in `app.rs` dispatching `AppAction::SetTimeSignature(n, d)`
  - Pass `time_signature` and `on_set_time_signature` props from `App` to `NavBar`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6_

  - [x]* 6.1 Write property test for time signature display format
    - **Property 9: Time signature display format**
    - Extract a pure `format_time_signature(ts: TimeSignature) -> String` helper and test it
    - **Validates: Requirements 2.6**

- [ ] 7. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Checkpoints ensure incremental validation
- Property tests validate universal correctness properties
- Unit tests validate specific examples and edge cases
