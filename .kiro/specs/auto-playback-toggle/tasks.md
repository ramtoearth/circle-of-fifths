# Implementation Plan: Auto-Playback Toggle

## Overview

Add a persistent boolean toggle that gates whether clicking a circle segment or selecting a
chord progression starts a Playback_Session. When disabled, clicks produce an immediate
Static_Highlight with no audio. When enabled (default), all existing behaviour is preserved.
The toggle state is persisted to localStorage under `cof_auto_playback`.

## Tasks

- [ ] 1. Extend AppState and reducer with auto_playback_enabled
  - Add `pub auto_playback_enabled: bool` field to `AppState` struct in `src/state/mod.rs`
  - Set `auto_playback_enabled: true` in the `Default` impl for `AppState`
  - Add `ToggleAutoPlayback` variant to `AppAction` enum
  - Add reducer arm for `ToggleAutoPlayback` that flips `auto_playback_enabled` and leaves all other fields unchanged
  - _Requirements: 1.3, 7.1, 7.2, 7.3_

  - [ ]* 1.1 Write property tests for ToggleAutoPlayback reducer
    - **Property 1: Toggle is a boolean flip** — for any initial `auto_playback_enabled`, dispatching `ToggleAutoPlayback` produces `!initial`
    - **Property 2: Toggle round-trip restores original value** — two dispatches return to original
    - **Property 8: Auto-playback toggle does not affect mute state** — `ToggleAutoPlayback` leaves `muted` unchanged; `ToggleMute` leaves `auto_playback_enabled` unchanged
    - **Validates: Requirements 1.3, 7.1, 7.2, 7.3**

  - [ ]* 1.2 Write unit tests for ToggleAutoPlayback reducer
    - `ToggleAutoPlayback` on `true` state → `auto_playback_enabled` is `false`
    - `ToggleAutoPlayback` on `false` state → `auto_playback_enabled` is `true`
    - `ToggleAutoPlayback` does not mutate any other field (bpm, muted, selected_key, etc.)
    - `ToggleMute` does not change `auto_playback_enabled`
    - _Requirements: 1.3, 7.2, 7.3_

- [x] 2. Add auto_playback serialization helpers and persistence
  - [x]* 2.1 Write property test for serialization round-trip
  - [x]* 2.2 Write unit tests for storage helpers

- [ ] 3. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 4. Wire auto_playback_enabled into the App component
  - In `src/components/app.rs`, restore `auto_playback_enabled` from `PersistedState` in the `use_reducer` initializer
  - Add `state.auto_playback_enabled` to the dependency tuple of the `save_state` `use_effect_with` hook
  - Add `on_toggle_auto_playback` callback: if `state.is_playing`, call the cancel-session block (`animation_handles.borrow_mut().clear()`, `audio.stop()`, `playing_note.set(None)`, `dispatch SetPlaying(false)`), then dispatch `ToggleAutoPlayback`
  - In `on_segment_click`, after the cancel-existing-session block and before the new-session block, add guard: `if !state.auto_playback_enabled { state.dispatch(AppAction::SelectKey(key)); return; }`
  - In `on_progression_click`, after the cancel-existing-session block and before the new-session block, add guard: `if !state.auto_playback_enabled { state.dispatch(AppAction::SelectProgression(id)); return; }`
  - Pass `auto_playback_enabled={state.auto_playback_enabled}` and `on_toggle_auto_playback={on_toggle_auto_playback}` props to `NavBar`
  - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 4.1, 4.2, 5.1, 5.2, 5.3_

- [ ] 5. Add toggle button to NavBar component
  - In `src/components/nav_bar.rs`, add `pub auto_playback_enabled: bool` and `pub on_toggle_auto_playback: Callback<()>` to `NavBarProps`
  - Derive the button label: `"Auto-Play: On"` when `true`, `"Auto-Play: Off"` when `false`
  - Derive the `aria-label`: `"Disable auto-playback"` when `true`, `"Enable auto-playback"` when `false`
  - Render a `<button>` with class `nav-bar__btn nav-bar__btn--auto-playback`, `aria-pressed={props.auto_playback_enabled.to_string()}`, and the derived label/aria-label, wired to `on_toggle_auto_playback`
  - _Requirements: 1.1, 1.2, 1.4_

- [ ] 6. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for faster MVP
- Each task references specific requirements for traceability
- Property tests use `proptest` (already a dev-dependency); tag each test with `// Feature: auto-playback-toggle, Property N: ...`
- The guard in `on_segment_click` / `on_progression_click` sits in the App component (not the reducer) because `Timeout` handles and audio calls live outside the reducer
- `cancel_active_session` logic is reused verbatim from the cancellable-playback feature — no new cancellation code needed
