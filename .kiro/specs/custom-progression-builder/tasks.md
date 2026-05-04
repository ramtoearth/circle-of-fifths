# Implementation Plan: Custom Progression Builder

## Overview

Add a mode where the user builds a chord progression from diatonic chord tiles and launches the
existing wait-based play-along with it. The main structural change is switching `PlayAlongState`
from storing a `ProgressionId` (predefined lookup) to storing an inline `Progression` value
(works for both predefined and custom). The builder UI is a new component wired into the existing
`App` / `ProgressionPanel` layout.

Pure-Rust reducer logic is tested with `cargo test`. Browser behaviour is manually verified.

## Tasks

- [ ] 1. Refactor `PlayAlongState` to store `Progression` inline (replaces `progression_id`)
  - In `src/state/mod.rs`: change `PlayAlongState.progression_id: ProgressionId` to
    `progression: Progression`; add `from_builder: bool` field
  - Update all reducer arms that call `data::find_progression(pa.progression_id)` to use
    `pa.progression` directly (`EnterPlayAlong`, `PlayAlongChordCorrect`)
  - Update `EnterPlayAlong(id)`: call `data::find_progression(id)` once in the arm, store the
    full `Progression` in `PlayAlongState`; guard returns if not found
  - Update `ExitPlayAlong`: if `play_along_state.from_builder == true`, set
    `app_mode = AppMode::CustomProgressionBuilder`; otherwise `AppMode::Normal`
  - Update `src/components/play_along_panel.rs` props and render: replace `progression_id` with
    `progression: Progression` if it stored the ID; verify it already receives the full
    `Progression` via props (no change needed if `App` already passes it)
  - _Requirements: 7.5, 7.6_
  - _Depends on: None_

- [ ]* 1.1 Write unit tests for updated `ExitPlayAlong` routing
  - `ExitPlayAlong` with `from_builder == true` → `app_mode == CustomProgressionBuilder`
  - `ExitPlayAlong` with `from_builder == false` → `app_mode == Normal`
  - `ExitPlayAlong` always clears `play_along_state`
  - _Validates: Requirements 7.5, 7.6_
  - _Depends on: 1_

- [ ] 2. Add `AppMode::CustomProgressionBuilder` variant and `builder_progression` state field
  - In `src/state/mod.rs`: add `CustomProgressionBuilder` to `AppMode` enum
  - Add `builder_progression: Vec<ScaleDegree>` to `AppState`; default is `vec![]`
  - Add new `AppAction` variants: `EnterBuilder`, `ExitBuilder`, `BuilderToggle(ScaleDegree)`,
    `BuilderShiftAppend(ScaleDegree)`, `BuilderReset`, `EnterPlayAlongCustom`
  - Implement reducer arms (see design.md for exact logic):
    - `EnterBuilder`: set `app_mode = CustomProgressionBuilder`, clear `builder_progression`
    - `ExitBuilder`: set `app_mode = Normal`, clear `builder_progression`
    - `BuilderToggle(d)`: if `builder_progression` contains `d`, remove last occurrence; else
      append if `len < 16`
    - `BuilderShiftAppend(d)`: append if `len < 16`
    - `BuilderReset`: clear `builder_progression`
    - `EnterPlayAlongCustom`: guard on `midi_status == Connected`, `selected_key.is_some()`,
      `!builder_progression.is_empty()`; construct ephemeral `Progression { id: u32::MAX, key,
      chords: builder_progression.clone(), tags: vec![Custom], borrowed_chord: None }`;
      initialize `PlayAlongState { progression, current_chord_index: 0, chords_played: 0,
      showing_loop_cue: false, from_builder: true }`; set `app_mode = PlayAlong`
  - _Requirements: 1.3, 2.1, 3.1, 4.1, 5.2, 7.4_
  - _Depends on: 1_

- [ ]* 2.1 Write unit and property tests for builder reducer actions
  - Unit: `BuilderToggle` on empty list → appends the degree
  - Unit: `BuilderToggle` on list containing the degree → removes last occurrence only
  - Unit: `BuilderToggle` on list containing degree twice → removes last, first remains
  - Unit: `BuilderToggle` on list NOT containing the degree → appends
  - Unit: `BuilderShiftAppend` always appends regardless of existing occurrences
  - Unit: `BuilderReset` → empty list
  - Unit: `EnterPlayAlongCustom` with empty `builder_progression` → state unchanged
  - Unit: `EnterPlayAlongCustom` with `midi_status != Connected` → state unchanged
  - Unit: `EnterPlayAlongCustom` success → `app_mode == PlayAlong`, `play_along_state.from_builder == true`
  - **Property 1** (toggle idempotence): `BuilderToggle(D)` twice on empty list → empty
  - **Property 2** (shift always grows): for any list of len < 16, `BuilderShiftAppend` increases length by 1
  - **Property 3** (16-chord cap): for list of len == 16, neither action increases length
  - **Property 4** (reset clears all): for any list, `BuilderReset` produces empty list
  - _Validates: Requirements 2.1, 2.4, 3.1, 4.1, 4.3, 5.2_
  - _Depends on: 2_

- [ ] 3. Create `CustomProgressionBuilderPanel` component (`src/components/custom_progression_builder.rs`)
  - Define `CustomProgressionBuilderProps` with: `selected_key: Key`, `working_progression:
    Vec<ScaleDegree>`, `midi_status: MidiStatus`, `on_toggle: Callback<ScaleDegree>`,
    `on_shift_append: Callback<ScaleDegree>`, `on_reset: Callback<()>`,
    `on_start_play_along: Callback<()>`, `on_back: Callback<()>`
  - Derive diatonic chord tiles from `diatonic_chords(selected_key)` — 7 tiles
  - Each tile `onclick`: if `e.shift_key()` → `on_shift_append.emit(degree)`, else
    `on_toggle.emit(degree)`
  - Tile badge: count occurrences of `degree` in `working_progression`
  - Working progression display: map each slot to `"{roman_numeral} – {chord_name}"` using
    `diatonic_chords(selected_key)` for lookup by degree; show placeholder when empty
  - Reset button always visible; "Start Play Along" disabled when `working_progression.is_empty()`
    or `midi_status != MidiStatus::Connected`
  - Back button dispatches `on_back`
  - _Requirements: 1.4, 2.3, 3.3, 4.2, 5.1, 5.3, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1, 7.2, 7.3, 8.1_
  - _Depends on: 2_

- [ ] 4. Add CSS for the builder panel and chord tiles (`index.css`)
  - `.builder-panel` — flex column, gap, padding
  - `.builder-panel__header` — flex row, space-between
  - `.builder-panel__slots` — flex wrap, gap, min-height
  - `.builder-panel__slot` — styled chip/badge for each progression slot
  - `.builder-panel__placeholder` — muted italic text
  - `.chord-tiles` — CSS grid, 7 columns
  - `.chord-tile` — card-like button; hover state; `position: relative` for badge
  - `.chord-tile__badge` — absolute circle top-right showing occurrence count
  - _Requirements: 6.1, 6.2, 6.3_
  - _Depends on: 3_

- [ ] 5. Wire `CustomProgressionBuilderPanel` into `App` (`src/components/app.rs`)
  - Add `on_enter_builder`, `on_exit_builder`, `on_builder_toggle`, `on_builder_shift_append`,
    `on_builder_reset`, `on_start_play_along_custom` callbacks dispatching the new `AppAction` variants
  - Conditionally render `CustomProgressionBuilderPanel` when
    `state.app_mode == AppMode::CustomProgressionBuilder` in the side panel
  - Pass `working_progression = state.builder_progression.clone()` and `selected_key` (unwrapped)
  - _Requirements: 1.1, 1.5, 7.4, 8.2_
  - _Depends on: 3_

- [ ] 6. Add "Build Custom" entry point in `ProgressionPanel` (`src/components/progression_panel.rs`)
  - Add `on_enter_builder: Callback<()>` prop to `ProgressionPanelProps`
  - Render a "Build Custom" button (or link) visible when `selected_key.is_some()`; the button
    emits `on_enter_builder`
  - The button is disabled / not rendered when `selected_key` is `None`
  - Wire the new prop in `App` where `ProgressionPanel` is rendered
  - _Requirements: 1.1, 1.2, 1.5_
  - _Depends on: 5_

- [ ] 7. Checkpoint — run `cargo test` and ensure all tests pass
  - Run `cargo test` and fix any compilation errors from new types/variants
  - Verify all 168 existing tests still pass
  - Verify new unit and property tests pass
  - _Depends on: 2, 3, 5, 6_

- [ ] 8. Manual verification in browser
  - Build with `trunk serve`
  - Select a key, click "Build Custom" — verify builder panel opens with 7 chord tiles and
    empty working progression placeholder
  - Click tiles: verify chords append, badge counts update, slot list grows
  - Click the same tile again (plain): verify last occurrence removed from slots
  - Shift+click a tile already in the progression: verify it appends a second copy
  - Click Reset: verify slots clear, builder remains open
  - Click Back: verify return to normal progression panel, working progression discarded
  - Connect MIDI, add 1+ chords, click "Start Play Along": verify play-along launches with
    the custom chords, finger hints correct, chord advance works through all custom chords
  - Complete a loop: verify loop cue appears, play-along returns to first custom chord
  - Press Stop: verify return to builder with working progression intact
  - Add 16 chords: verify 17th click has no effect
  - Change key while builder is open: verify tile names update, slot names update
  - _Depends on: 4, 6, 7_

## Notes

- Tasks marked `*` are optional for a faster MVP
- Task 1 is the only task that modifies existing play-along infrastructure; all others are additive
- Tasks 3 and 4 can be done in parallel with task 2.1
- The sentinel ID `u32::MAX` in the ephemeral `Progression` is safe because `data::all_progressions()`
  only generates IDs 0–59; nothing looks up `u32::MAX` in the predefined pool
- `ScaleDegree` must implement `Copy` for the builder toggle action — confirm it does (it is a
  fieldless enum so `Copy` is derivable); add `#[derive(Copy)]` if missing
