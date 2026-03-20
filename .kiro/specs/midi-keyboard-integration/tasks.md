# Implementation Plan: MIDI Keyboard Integration

## Overview

Extend the existing Circle of Fifths Yew/WASM app with Web MIDI API support via `js-sys` interop. MIDI events flow through a new `MidiEngine` struct into the existing `AppState` reducer via new `AppAction` variants. New UI components (`MidiStatusBar`, `PracticePanel`, `PlayAlongPanel`) and extensions to existing ones (`PianoPanel`, `NavBar`, `ProgressionPanel`) surface the feature to the user.

All Rust-only logic (note math, chord recognition, key detection, reducer) is tested with `cargo test`. Browser-API-dependent tests run with `wasm-pack test --headless --chrome`.

## Tasks

- [x] 1. Add `src/midi/mod.rs` with core data types
  - Define `MidiEvent`, `HeldNote`, `MidiStatus`, `RecognizedChord`, `KeySuggestion`, `PracticeScore`, `ChordResult`, `PlayAlongScore` as specified in the design
  - Implement `HeldNote::from_midi(note, velocity)` and `HeldNote::velocity_opacity()`
  - Implement `PitchClass::from_index(u8)` if not already present in `src/music_theory/mod.rs`
  - _Requirements: 2.3, 2.4_
  - _Depends on: None_

- [x] 2. Extend `AppState` and `AppAction` with MIDI fields and variants
  - Add `midi_status`, `device_names`, `held_notes`, `rolling_window`, `recognized_chord`, `key_suggestions`, `app_mode`, `practice_state`, `play_along_state`, `metronome_active` fields to `AppState` in `src/state/mod.rs`
  - Add `AppMode`, `PracticeState`, `PlayAlongState` types
  - Add all new `AppAction` variants: `MidiStatusChanged`, `MidiDevicesChanged`, `MidiNoteOn`, `MidiNoteOff`, `ClearRollingWindow`, `EnterPractice`, `ExitPractice`, `PracticeAdvance`, `EnterPlayAlong`, `ExitPlayAlong`, `PlayAlongTick`, `RecordPlayAlongChordResult`, `ToggleMetronome`
  - Note: `PlayAlongSetBpm` is NOT needed — BPM changes go through the existing `SetBpm` action which updates `AppState.bpm`
  - Implement reducer arms for all new variants
  - Reducer arm for `MidiNoteOn`: add to `held_notes`, append `(pitch_class, timestamp)` to `rolling_window`
  - Reducer arm for `MidiNoteOff`: remove matching `midi_note` from `held_notes`; velocity=0 NoteOn also removes (Property 4)
  - Reducer arm for `MidiDevicesChanged` with empty list: clear `held_notes` (Property 12)
  - Reducer arm for `ClearRollingWindow`: empty `rolling_window` and `key_suggestions` (Property 11)
  - Reducer arm for `ExitPlayAlong`: set `app_mode = Normal`, `play_along_state = None`, restore `metronome_active` from `play_along_state.pre_play_along_metronome_active` (Property 16)
  - Reducer arm for `SetBpm`: clamp bpm to [40, 200] and update `AppState.bpm` (Property 15) — note `SetBpm` already exists but currently does NOT clamp; add clamping
  - Reducer arm for `ToggleMetronome`: flip `metronome_active` (Property 17)
  - _Requirements: 1.7, 1.8, 2.1, 2.2, 2.5, 4.6, 6.2, 6.7, 7.1, 7.4_
  - _Depends on: 1_

- [x]* 2.1 Write property tests for MIDI reducer actions
  - **Property 1: NoteOn/NoteOff round-trip** — `held_notes` unchanged after NoteOn+NoteOff for same note
  - **Property 4: Velocity=0 treated as NoteOff** — NoteOn with velocity=0 removes note from `held_notes`
  - **Property 11: ClearRollingWindow resets state** — `rolling_window` and `key_suggestions` both empty after dispatch
  - **Property 12: Device disconnection clears held notes** — `MidiDevicesChanged([])` empties `held_notes`
  - **Property 15: BPM clamping** — `AppState.bpm` always in [40, 200] after `SetBpm` with any input (note: `SetBpm` already exists; this test validates the new clamping behavior)
  - **Property 16: ExitPlayAlong resets mode** — `app_mode == Normal` and `play_along_state == None`
  - **Property 17: Metronome toggle round-trip** — `metronome_active` unchanged after two `ToggleMetronome` dispatches
  - _Validates: Requirements 1.7, 2.1, 2.2, 2.5, 4.6, 6.2, 6.7, 7.1, 7.8_
  - _Depends on: 2_

- [x] 3. Implement chord recognition in `src/midi/mod.rs`
  - Define `CHORD_DICTIONARY` static array of `(name: &str, intervals: &[u8])` covering triads (major, minor, diminished, augmented) and seventh chords (maj7, min7, dom7, half-dim7, dim7)
  - Implement `recognize_chord(held: &[HeldNote], selected_key: Option<Key>) -> Option<RecognizedChord>` following the algorithm in the design (collect distinct PitchClasses, try all inversions, score by matching PitchClasses, tie-break by fewest extra notes)
  - Populate `roman_numeral` and `is_diatonic` fields when `selected_key` is `Some`
  - Return `None` when fewer than 3 distinct PitchClasses (Property 5)
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.6_
  - _Depends on: 1_

- [x]* 3.1 Write property tests for chord recognition
  - **Property 5: Chord recognition requires 3+ distinct PitchClasses** — `recognize_chord` returns `None` for < 3 distinct PCs
  - **Property 6: Known chords recognized in all inversions** — all dictionary chords recognized in every inversion
  - **Property 7: Chord-in-key annotation correctness** — `roman_numeral` and `is_diatonic` correct for all key/chord combos
  - _Validates: Requirements 3.1, 3.2, 3.3, 3.4_
  - _Depends on: 3_

- [x] 4. Implement key detection in `src/midi/mod.rs`
  - Implement `filter_rolling_window(entries: &[(PitchClass, f64)], now_ms: f64) -> Vec<(PitchClass, f64)>` — keep only entries where `now_ms - timestamp_ms <= 10_000.0` (Property 8)
  - Implement `detect_keys(window: &[(PitchClass, f64)], now_ms: f64) -> Vec<KeySuggestion>` — return empty Vec when < 4 distinct PitchClasses (Property 9), otherwise score all 24 keys and return top 3 sorted by score descending (Property 10)
  - _Requirements: 4.1, 4.2, 4.3, 4.5_
  - _Depends on: 1_

- [x]* 4.1 Write property tests for key detection
  - **Property 8: Rolling window excludes stale notes** — `filter_rolling_window` excludes entries older than 10s
  - **Property 9: Key detection threshold** — `detect_keys` returns empty Vec for < 4 distinct PitchClasses
  - **Property 10: Key detection ranking** — results sorted by score descending, scores computed correctly
  - _Validates: Requirements 4.1, 4.2, 4.3_
  - _Depends on: 4_

- [x]* 4.2 Write property tests for HeldNote math
  - **Property 2: MIDI note to PitchClass/Octave derivation** — `from_midi(n, v).pitch_class == PitchClass::from_index(n % 12)` and `.octave == (n/12) as i8 - 1`
  - **Property 3: Velocity opacity is monotonically increasing** — `velocity_opacity(v1) < velocity_opacity(v2)` for v1 < v2; boundary values 0.35 and 1.0
  - _Validates: Requirements 2.3, 2.4_
  - _Depends on: 1_

- [x] 4.3 Wire existing NavBar props in `app.rs` and fix BPM slider range
  - `NavBarProps` already has `selected_key`, `bpm`, and `on_set_bpm` fields, but `app.rs` does NOT yet pass them to `<NavBar>`
  - Pass `selected_key={state.selected_key}`, `bpm={state.bpm}`, and `on_set_bpm={on_set_bpm}` in the `<NavBar>` usage in `src/components/app.rs`
  - In `src/components/nav_bar.rs`, fix the BPM slider range from `min="60" max="240"` to `min="40" max="200"` (Requirement 7.8)
  - Also add clamping to the `SetBpm` reducer arm in `src/state/mod.rs` so values outside [40, 200] are clamped (currently no clamping exists)
  - _Requirements: 7.8_
  - _Depends on: None_

- [x] 5. Checkpoint — run `cargo test` and ensure all pure-Rust tests pass
  - Ensure all tests pass, ask the user if questions arise.
  - _Depends on: 2, 3, 4_

- [x] 6. Implement `MidiEngine` JS interop in `src/midi/mod.rs`
  - Implement `MidiEngine::request_access(dispatch: Callback<AppAction>) -> Self` using `js_sys::Reflect` and `wasm_bindgen::closure::Closure` as described in the design
  - Implement `parse_midi_message(data: &[u8]) -> MidiEvent` — parse status byte, handle NoteOn/NoteOff/velocity=0
  - Implement `MidiEngine::register_ports(&self, dispatch: Callback<AppAction>)` — iterate `MIDIAccess.inputs`, set `onmidimessage` closure on each port
  - Implement hot-plug via `onstatechange` on `MIDIAccess` — dispatch `MidiDevicesChanged` and re-register ports
  - Wrap all `js_sys::Reflect::get` and `.dyn_into()` calls in `Result`/`Option` chains; on failure dispatch `MidiStatusChanged(Unavailable)` rather than panic
  - Implement `MidiEngine::connected_device_names(&self) -> Vec<String>`
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7_
  - _Depends on: 2_

- [x] 7. Wire `MidiEngine` into `App` component
  - In `src/components/app.rs`, initialize `MidiEngine` in a `use_effect_with((), ...)` hook after mount, passing the Yew dispatch handle
  - Store the `MidiEngine` in a `use_mut_ref` to keep closures alive for the component lifetime
  - Add a `use_effect` that calls `recognize_chord` and `detect_keys` whenever `held_notes` or `rolling_window` changes, dispatching `AppAction` to update `recognized_chord` and `key_suggestions` in state
  - _Requirements: 1.1, 3.5, 4.5_
  - _Depends on: 6_

- [x] 8. Create `MidiStatusBar` component in `src/components/midi_status_bar.rs`
  - Implement props: `midi_status`, `device_names`, `recognized_chord`, `key_suggestions`, `on_clear_window`
  - Render MIDI connection status badge, device name(s), recognized chord name + Roman numeral + diatonic/borrowed indicator, top key suggestions with scores, and a "Clear" button that fires `on_clear_window`
  - Show unavailable/permission-denied/no-devices notices per Requirements 1.2, 1.3
  - Highlight top key suggestion on the Circle is handled via `key_suggestions[0]` passed down from `AppState` (no new state needed here)
  - Register component in `src/components/mod.rs`
  - _Requirements: 1.2, 1.3, 1.4, 1.8, 3.1, 3.3, 3.4, 3.6, 4.2, 4.3, 4.6_
  - _Depends on: 7_

- [x] 9. Extend `PianoPanel` with MIDI highlight props
  - Add `held_notes: Vec<HeldNote>` and `practice_target: Option<Vec<PitchClass>>` props to `PianoPanelProps` in `src/components/piano_panel.rs`
  - For each piano key, apply `midi-held` CSS class + inline `opacity` from `velocity_opacity()` when the key's `(pitch_class, octave)` matches a `HeldNote`
  - When `practice_target` is `Some`, apply `midi-correct` (green) to held notes whose PitchClass is in target, `midi-incorrect` (red) to held notes not in target (Property 13)
  - Implement auto-scroll: when `held_notes` is non-empty, scroll to keep the lowest held note in view
  - When `held_notes` is empty, revert to scale-only highlights
  - Add CSS rules for `.midi-held`, `.midi-correct`, `.midi-incorrect` in `index.css`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.6, 2.7, 5.3, 6.4_
  - _Depends on: 2_

- [ ]* 9.1 Write property test for note color classification
  - **Property 13: Practice/play-along note color classification** — correct/incorrect/unplayed sets are disjoint and cover all held + target notes
  - _Validates: Requirements 5.3, 6.4_
  - _Depends on: 9_

- [x] 10. Extend `NavBar` with Practice mode entry and Metronome toggle
  - Note: `selected_key`, `bpm`, and `on_set_bpm` props already exist in `NavBarProps` and are wired in task 4.3
  - Add `midi_status: MidiStatus`, `on_enter_practice: Callback<()>`, `metronome_active: bool`, and `on_toggle_metronome: Callback<()>` props to `NavBar` in `src/components/nav_bar.rs`
  - Render "Practice" button only when `midi_status == MidiStatus::Connected`; if not connected, show inline message per Requirement 5.7
  - Render "Metronome" toggle button adjacent to the BPM slider; label reflects current state ("Metronome: On" / "Metronome: Off")
  - BPM slider range is already fixed to 40–200 in task 4.3
  - Button dispatches `AppAction::EnterPractice` and `AppAction::ToggleMetronome` via their respective callbacks
  - _Requirements: 5.1, 5.7, 7.1, 7.8_
  - _Depends on: 2, 4.3_

- [x] 11. Extend `ProgressionPanel` with Play-Along entry
  - Add `midi_status: MidiStatus` and `on_enter_play_along: Callback<ProgressionId>` props to `ProgressionPanel` in `src/components/progression_panel.rs`
  - Render "Play Along" button per progression only when `midi_status == MidiStatus::Connected` and a progression is active
  - If not connected, show inline message per Requirement 6.8
  - _Requirements: 6.1, 6.8_
  - _Depends on: 2_

- [x] 12. Create `PracticePanel` component in `src/components/practice_panel.rs`
  - Implement props: `target_chord: DiatonicChord`, `held_notes: Vec<HeldNote>`, `score: PracticeScore`, `on_exit: Callback<()>`
  - Display target chord name and notes; show per-note color feedback via `practice_target` passed to `PianoPanel`
  - Detect when all target PitchClasses are present in `held_notes` and dispatch `AppAction::PracticeAdvance`
  - Display accuracy score as `correct_notes / total_notes_played` (guard divide-by-zero)
  - Show progression summary when target progression is completed
  - Register in `src/components/mod.rs`; render from `App` when `app_mode == AppMode::Practice`
  - _Requirements: 5.2, 5.3, 5.4, 5.5, 5.6_
  - _Depends on: 9, 10_

- [x]* 12.1 Write property test for accuracy score invariant
  - **Property 14: Accuracy score invariant** — `correct_notes <= total_notes_played` always holds; ratio in [0.0, 1.0]
  - _Validates: Requirements 5.5, 6.5_
  - _Depends on: 12_

- [x] 13. Create `PlayAlongPanel` component in `src/components/play_along_panel.rs`
  - Implement props: `progression`, `current_chord_index`, `bpm`, `held_notes`, `score`, `on_stop`
  - Note: there is NO `on_bpm_change` prop — BPM is global via `AppState.bpm` and is controlled exclusively from the NavBar slider. The panel reads `bpm` from props but does not own a BPM input.
  - Set up a beat timer using `gloo_timers` (or `web_sys::Window::set_interval`) at interval `60_000 / bpm` ms; on each tick dispatch `AppAction::PlayAlongTick`
  - Drop/clear the timer handle on component unmount
  - Display current expected chord, highlight its notes as `practice_target` in `PianoPanel`
  - Dispatch `RecordPlayAlongChordResult` each tick based on whether all target PitchClasses were in `held_notes`
  - Show results summary when progression completes; provide "Stop" button firing `on_stop`
  - Register in `src/components/mod.rs`; render from `App` when `app_mode == AppMode::PlayAlong`
  - _Requirements: 6.2, 6.3, 6.4, 6.5, 6.6, 6.7_
  - _Depends on: 9, 11_

- [x] 13.5 Implement metronome in `AudioEngine` and wire into `App`
  - Add `schedule_metronome_click(&self, start: f64)` to `AudioEngine` in `src/audio/mod.rs`; use a triangle oscillator at ~1200 Hz with a 30 ms duration and fast decay envelope (distinct from sine-wave note playback)
  - Add `schedule_metronome_click` forwarding method to `AudioEngineHandle`
  - In `src/components/app.rs`, add a `use_interval` (gloo_timers) that fires at `60_000 / bpm` ms when `metronome_active` is true; each tick calls `audio_engine.schedule_metronome_click(ctx.current_time() + lookahead)`
  - Recreate the interval whenever `bpm` or `metronome_active` changes (use `use_effect_with((bpm, metronome_active), ...)`)
  - When `metronome_active` is false or the engine is muted, skip scheduling
  - When `EnterPlayAlong` is dispatched, save the current `metronome_active` value in `PlayAlongState.pre_play_along_metronome_active` and force `metronome_active = true`
  - When `ExitPlayAlong` is dispatched, restore `metronome_active` from the saved value in `PlayAlongState`
  - Note: `piano-key--playing` CSS class already exists in `index.css` — do NOT re-add it
  - _Requirements: 7.2, 7.3, 7.5, 7.6, 7.9, 6.9_
  - _Depends on: 2, 10_

- [x] 13.6 Add `metronome_active` persistence to `src/storage/mod.rs`
  - Add `metronome_active: bool` field to `PersistedState` (default `false`)
  - Add `serialize_metronome_active(bool) -> String` and `deserialize_metronome_active(&str) -> bool` helpers
  - Wire into `load_state`: read `cof_metronome_active` key from localStorage
  - Wire into `save_state`: write `metronome_active` to localStorage on every save
  - In `app.rs`, load `metronome_active` from `PersistedState` on init (alongside existing theme/muted/favorites/best_scores hydration)
  - In `app.rs`, include `state.metronome_active` in the `use_effect_with` dependency tuple that triggers `save_state`
  - _Requirements: 7.7_
  - _Depends on: 2_

- [x] 14. Wire new props through `App` component
  - Pass `held_notes` and `practice_target` (derived from `practice_state` or `play_along_state`) to `PianoPanel`
  - Pass `midi_status`, `device_names`, `recognized_chord`, `key_suggestions` to `MidiStatusBar`; wire `on_clear_window` to dispatch `ClearRollingWindow`
  - Pass `midi_status`, `on_enter_practice`, `metronome_active`, and `on_toggle_metronome` to `NavBar` (note: `selected_key`, `bpm`, `on_set_bpm` are already wired from task 4.3)
  - Pass `midi_status` and `on_enter_play_along` to `ProgressionPanel`
  - Conditionally render `PracticePanel` or `PlayAlongPanel` based on `app_mode`
  - _Requirements: 1.4, 1.8, 2.1, 5.1, 6.1, 7.1_
  - _Depends on: 8, 9, 10, 11, 12, 13, 13.5, 13.6_

- [x] 15. Final checkpoint — ensure all tests pass
  - Run `cargo test` for pure-Rust tests
  - Run `wasm-pack test --headless --chrome` for browser-API tests
  - Ensure all tests pass, ask the user if questions arise.
  - _Depends on: 14_

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- Property tests use `proptest` with the tag format `// Feature: midi-keyboard-integration, Property N: <text>`
- Pure Rust tests (tasks 2.1, 3.1, 4.1, 4.2, 9.1, 12.1) run with `cargo test`
- Task 4.3 (wire NavBar props + fix BPM slider range) has no dependencies and can be done immediately
- Tasks 1, 3, 4 can begin in parallel immediately
- Tasks 2.1, 3.1, 4.1, 4.2 can run in parallel once their respective parents are done
- Tasks 9, 10, 11 can run in parallel once task 2 is done
- Task 13.5 and 13.6 can run in parallel with tasks 12 and 13 once task 2 and 10 are done
