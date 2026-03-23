# Implementation Plan

> **Parallelism notes**
> - Tasks 1 and 2 are **sequential** (exploration test first, preservation test second — both on unfixed code)
> - Tasks 3.1 and 3.2 are **parallel** (state change and app.rs change are independent)
> - Tasks 3.3 and 3.4 are **parallel** (unit tests and integration tests are independent)
> - Tasks 3.3 and 3.4 depend on 3.1 + 3.2 being complete
> - Task 4 depends on all of 3.x being complete

---

- [x] 1. Write bug condition exploration test
  - **Property 1: Bug Condition** - Highlighted Chord Freezes During Progression Playback
  - **CRITICAL**: This test MUST FAIL on unfixed code — failure confirms the bug exists
  - **DO NOT attempt to fix the test or the code when it fails**
  - **NOTE**: This test encodes the expected behavior — it will validate the fix when it passes after implementation
  - **GOAL**: Surface counterexamples demonstrating that `highlighted_chord` does not advance after `SelectProgression`
  - **Scoped PBT Approach**: Scope the property to progression ID 0 (a known multi-chord progression) with target indices 1..len
  - In `src/state/mod.rs` tests, add a property test that:
    - Dispatches `SelectProgression(0)` to get `state_0` (index=0, highlight=chord_0)
    - For each target index `i` in `1..progression.chords.len()`, dispatches `AdvanceProgressionChord(i)` (action does not exist yet — test will fail to compile, confirming the action is missing)
    - Asserts `state_1.highlighted_chord == chord_at(id, i)` and `state_1.active_progression.current_index == i`
  - Also add a deterministic "frozen highlight" test: dispatch `SelectProgression(0)`, assert `highlighted_chord` is chord at index 0, dispatch nothing further, assert `highlighted_chord` is still index 0 — documents the freeze
  - Run test on UNFIXED code
  - **EXPECTED OUTCOME**: Test FAILS (compile error or assertion failure — this proves the bug exists)
  - Document counterexamples found (e.g., "`AdvanceProgressionChord` variant does not exist; after `SelectProgression`, `highlighted_chord` stays frozen at index 0")
  - Mark task complete when test is written, run, and failure is documented
  - _Requirements: 1.1, 1.2, 1.3_
  - **COMPLETED**: Added `frozen_highlight_documents_the_bug` (deterministic, documents the freeze) and
    `prop_advance_progression_chord_updates_highlight` (PBT) in `src/state/mod.rs` under
    `tests::bug_condition_exploration`. Running `cargo test` on unfixed code produces:
    `error[E0599]: no variant or associated item named AdvanceProgressionChord found for enum state::AppAction`
    **Counterexamples**: (1) `AdvanceProgressionChord` variant does not exist in `AppAction`;
    (2) after `SelectProgression(0)`, `highlighted_chord` stays frozen at index 0 indefinitely.

- [x] 2. Write preservation property tests (BEFORE implementing fix)
  - **Property 2: Preservation** - Non-Playback Behaviors Unchanged
  - **IMPORTANT**: Follow observation-first methodology — run UNFIXED code with non-buggy inputs first
  - **Observe on unfixed code**:
    - `SelectProgression(0)` → `NextChord` → `highlighted_chord` equals chord at index 1
    - `SelectProgression(0)` → `PrevChord` → `highlighted_chord` equals chord at last index
    - `SelectChord(c)` → `highlighted_chord` equals `c`, `active_progression` is `None`
    - `SelectKey(k)` → `active_progression` is `None`, `highlighted_chord` is `None`
  - In `src/state/mod.rs` tests, add property-based tests (using `proptest`) that:
    - For any valid progression ID and state, `NextChord` and `PrevChord` produce the same `highlighted_chord` before and after the fix (isBugCondition is false for these actions)
    - For any `DiatonicChord`, `SelectChord` sets `highlighted_chord` and clears `active_progression`
    - For any `Key`, `SelectKey` clears `active_progression` and `highlighted_chord`
    - Stale timer guard: `SelectProgression(A)` → `SelectProgression(B)` → `AdvanceProgressionChord(1)` asserts state reflects progression B at index 0 (not A at 1)
  - Run tests on UNFIXED code
  - **EXPECTED OUTCOME**: Tests PASS (confirms baseline behavior to preserve)
  - Mark task complete when tests are written, run, and passing on unfixed code
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 3. Fix for highlighted chord not tracking active progression during playback

  > Tasks 3.1 and 3.2 are **independent and can run in parallel**.
  > Tasks 3.3 and 3.4 are **independent and can run in parallel**, but both depend on 3.1 and 3.2.

  - [ ] 3.1 Add `AdvanceProgressionChord(usize)` action and reducer arm in `src/state/mod.rs`
    - Add `AdvanceProgressionChord(usize)` variant to the `AppAction` enum
    - Add reducer arm in `app_reducer` for `AdvanceProgressionChord(target_index)`:
      - Guard: if `active_progression` is `None`, return state unchanged
      - Guard: if `active_progression.current_index >= target_index`, return state unchanged (idempotency — rejects stale dispatches)
      - Otherwise: set `active_progression.current_index = target_index` and set `highlighted_chord = chord_highlight_at(&progression, target_index)`
    - _Bug_Condition: `isBugCondition(state)` where `state.active_progression` is `Some(ap)` and `state.highlighted_chord != chord_at(ap.id, ap.current_index)`_
    - _Expected_Behavior: after `AdvanceProgressionChord(i)`, `state.highlighted_chord == chord_highlight_at(progression, i)` and `state.active_progression.current_index == i`_
    - _Preservation: guard clause ensures `NextChord`, `PrevChord`, `SelectChord`, `SelectKey` arms are completely unaffected_
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ] 3.2 Schedule `Timeout` callbacks in `on_progression_click` in `src/components/app.rs`
    - In `on_progression_click`, after calling `audio.play_progression(p)` and before dispatching `SelectProgression(id)`, add a loop over `1..progression.chords.len()`
    - For each index `i`, create a `Timeout::new((i as u32) * 1000, move || { state.dispatch(AppAction::AdvanceProgressionChord(i)); }).forget()`
    - Clone `state` before the loop so each closure captures its own clone
    - The reducer's guard clause handles the case where the user navigates away before a timeout fires (stale dispatch is a no-op)
    - Single-chord progressions: `1..1` is an empty range — no timeouts scheduled, no change in behavior
    - _Bug_Condition: `on_progression_click` previously had no timer dispatch, leaving `highlighted_chord` frozen_
    - _Expected_Behavior: one `Timeout` per chord index 1..len, each dispatching `AdvanceProgressionChord(i)` at `i * 1000 ms`_
    - _Preservation: `SelectProgression` dispatch and `audio.play_progression` call are unchanged_
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ] 3.3 Write unit tests for the new reducer arm (depends on 3.1 + 3.2)
    - `advance_progression_chord_updates_highlight`: dispatch `SelectProgression(id)` then `AdvanceProgressionChord(i)` for each valid index `i` in `1..len`; assert `highlighted_chord` matches `chord_highlight_at(progression, i)` and `current_index == i`
    - `advance_progression_chord_noop_when_no_active`: dispatch `AdvanceProgressionChord(1)` on default state; assert state is unchanged
    - `advance_progression_chord_noop_after_key_change`: dispatch `SelectProgression(id)`, then `SelectKey(k)`, then `AdvanceProgressionChord(1)`; assert `highlighted_chord` is `None` and `active_progression` is `None`
    - `advance_progression_chord_noop_for_stale_index`: dispatch `SelectProgression(id)`, then `AdvanceProgressionChord(2)`, then `AdvanceProgressionChord(1)`; assert `current_index` is still 2 (stale dispatch rejected)
    - `advance_progression_chord_noop_after_progression_switch`: dispatch `SelectProgression(A)`, then `SelectProgression(B)`, then `AdvanceProgressionChord(1)`; assert `active_progression.id == B` and `current_index == 0` (stale timer from A is rejected because `current_index` guard fires — note: this relies on the id-match guard described in design Change 4; add id check to reducer if not already present)
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4_

  - [ ] 3.4 Write integration tests for full playback simulation (depends on 3.1 + 3.2)
    - Full playback simulation: dispatch `SelectProgression(id)`, then sequentially dispatch `AdvanceProgressionChord(1)`, `AdvanceProgressionChord(2)`, … up to `len-1`; assert `highlighted_chord` matches each chord in order
    - Interruption test: dispatch `SelectProgression(id)`, dispatch `NextChord` (manual navigation), then dispatch `AdvanceProgressionChord(1)`; assert the stale timer dispatch is a no-op (manual navigation already advanced the index)
    - Key change during playback: dispatch `SelectProgression(id)`, then `SelectKey(k)`, then `AdvanceProgressionChord(1)`; assert `highlighted_chord` is `None`
    - Single-chord progression: dispatch `SelectProgression` for a 1-chord progression; assert no `AdvanceProgressionChord` is needed and `highlighted_chord` is correct at index 0 throughout
    - _Requirements: 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4_

  - [ ] 3.5 Verify bug condition exploration test now passes (depends on 3.1 + 3.2)
    - **Property 1: Expected Behavior** - Highlighted Chord Tracks Active Progression Index
    - **IMPORTANT**: Re-run the SAME test from task 1 — do NOT write a new test
    - The test from task 1 encodes the expected behavior; when it passes, the fix is confirmed
    - Run bug condition exploration test from step 1
    - **EXPECTED OUTCOME**: Test PASSES (confirms bug is fixed)
    - _Requirements: 2.1, 2.2, 2.3_

  - [ ] 3.6 Verify preservation tests still pass (depends on 3.1 + 3.2)
    - **Property 2: Preservation** - Non-Playback Behaviors Unchanged
    - **IMPORTANT**: Re-run the SAME tests from task 2 — do NOT write new tests
    - Run preservation property tests from step 2
    - **EXPECTED OUTCOME**: Tests PASS (confirms no regressions in `NextChord`, `PrevChord`, `SelectChord`, `SelectKey`)
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 4. Checkpoint — Ensure all tests pass
  - Run `cargo test` and confirm all tests pass
  - Confirm the exploration test (task 1) now passes after the fix
  - Confirm the preservation tests (task 2) still pass
  - Confirm all unit tests (task 3.3) pass
  - Confirm all integration tests (task 3.4) pass
  - Ask the user if any questions arise
