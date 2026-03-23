# Implementation Plan: Cancellable Playback

## Overview

Replace the fire-and-forget `Timeout::forget()` animation model with a tracked, cancellable
one. All `Timeout` handles for the active playback session are stored in a `use_mut_ref<Vec<Timeout>>`
inside `App`. Dropping the `Vec` cancels every pending callback. A new `is_playing: bool` field
in `AppState` drives Stop button visibility.

## Tasks

- [x] 1. Extend AppState and reducer with `is_playing` flag
  - Add `pub is_playing: bool` field to `AppState` in `src/state/mod.rs`, defaulting to `false`
  - Add `SetPlaying(bool)` variant to `AppAction`
  - Handle `AppAction::SetPlaying(playing)` in `app_reducer`: return `AppState { is_playing: playing, ..state }`
  - _Requirements: 1.1, 1.2, 2.4_

  - [x]* 1.1 Write unit tests for SetPlaying reducer
    - Test `SetPlaying(true)` sets `is_playing = true`
    - Test `SetPlaying(false)` sets `is_playing = false`
    - Test that `SetPlaying` does not mutate any other field
    - _Requirements: 1.1, 1.2_

  - [x]* 1.2 Write property test for Property 2: Stop transitions is_playing to false
    - **Property 2: Stop transitions is_playing to false**
    - **Validates: Requirements 2.4, 1.2**

  - [x]* 1.3 Write property test for Property 3: SetPlaying round-trip
    - **Property 3: SetPlaying round-trip**
    - **Validates: Requirements 1.1, 1.2**

- [x] 2. Add `animation_handles` ref and `cancel_active_session` closure to App
  - In `src/components/app.rs`, add `let animation_handles = use_mut_ref(|| Vec::<Timeout>::new());`
  - Define a `cancel_active_session` closure that:
    - Calls `animation_handles.borrow_mut().clear()` (drops all `Timeout`s, cancelling callbacks)
    - Calls `audio.stop()`
    - Calls `playing_note.set(None)`
    - Dispatches `AppAction::SetPlaying(false)`
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 5.1, 5.2_

- [x] 3. Modify `on_segment_click` to cancel before starting new playback
  - Call `cancel_active_session()` at the top of `on_segment_click`
  - After scheduling each `Timeout` for the scale animation, push the handle into
    `animation_handles` instead of calling `.forget()`
  - After the last note timeout is scheduled, push a final cleanup `Timeout` that calls
    `animation_handles.borrow_mut().clear()` and dispatches `SetPlaying(false)`
  - Dispatch `AppAction::SetPlaying(true)` after the first handle is pushed
  - When the clicked key equals `state.selected_key`, only call `cancel_active_session()` and
    dispatch `SelectKey` (deselect) — do not start a new session
  - _Requirements: 3.1, 3.2, 3.3, 5.3, 5.4_

- [ ] 4. Modify `on_progression_click` to cancel before starting new playback
  - Call `cancel_active_session()` at the top of `on_progression_click`
  - Schedule one `Timeout` per chord at `i * 1000 ms`; each timeout dispatches the equivalent
    of advancing `active_progression.current_index` (dispatch `AppAction::SelectProgression`
    for index 0, then use a new `AppAction::SetProgressionIndex(usize)` or reuse existing
    `NextChord` dispatches via timeouts for subsequent chords)
  - Push each handle into `animation_handles`
  - Push a final cleanup `Timeout` that clears `animation_handles` and dispatches `SetPlaying(false)`
  - Dispatch `AppAction::SetPlaying(true)` after the first handle is pushed
  - When the same progression is already active, only call `cancel_active_session()` — do not restart
  - _Requirements: 4.1, 4.2, 4.3, 5.3, 5.4_

- [ ] 5. Checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

- [ ] 6. Add Stop button to the render output
  - In the render section of `src/components/app.rs`, add an `on_stop` callback that calls
    `cancel_active_session()`
  - Inside the `piano-footer` `<div>`, above `<PianoPanel>`, render the Stop button
    conditionally on `state.is_playing`:
    ```rust
    if state.is_playing {
        <button class="stop-btn" onclick={on_stop} aria-label="Stop playback">{"■ Stop"}</button>
    }
    ```
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 6.1, 6.2_

- [x] 7. Add CSS for the Stop button
  - In `index.css`, add styles for `.stop-btn` so it is visually distinct and reachable
  - Ensure keyboard focus styles are present (`:focus-visible` outline)
  - _Requirements: 1.1, 1.3_

- [ ] 8. Write property tests for Idle State correctness
  - [ ]* 8.1 Write property test for Property 4: Idle State key highlight correctness
    - **Property 4: Idle State key highlight correctness**
    - **Validates: Requirements 7.1, 7.3**

  - [ ]* 8.2 Write property test for Property 5: Idle State chord highlight correctness
    - **Property 5: Idle State chord highlight correctness**
    - **Validates: Requirements 7.2**

  - [ ]* 8.3 Write property test for Property 6: Animation handle collection is empty after clear
    - **Property 6: Animation handle collection is empty after clear**
    - Use `Vec<u32>` as a structural stand-in for `Vec<Timeout>` (Timeout cannot be constructed outside WASM)
    - **Validates: Requirements 5.2, 5.4**

- [ ] 9. Final checkpoint — Ensure all tests pass
  - Ensure all tests pass, ask the user if questions arise.

## Notes

- Tasks marked with `*` are optional and can be skipped for a faster MVP
- `Timeout` handles are cancelled by `Drop`; clearing the `Vec` is the sole cancellation mechanism
- `audio.stop()` calls `ctx.suspend()` and is a no-op in degraded mode — animation cancellation is independent
- BPM is already clamped to `[40, 200]` by the reducer; use `.max(1)` defensively in interval calculations
- Property tests use `proptest` which is already a dev-dependency in `Cargo.toml`
