# Piano Chord Highlight Sync Bugfix Design

## Overview

When a chord progression is played via `audio.play_progression()`, the piano highlight freezes
on the first chord and never updates. The root cause is that `play_progression()` schedules all
chord audio up-front using the WebAudio API's timeline and then returns immediately — it is
fire-and-forget with no mechanism to notify the Yew state machine as each chord begins playing.
Because `highlighted_chord` in `AppState` is only updated by reducer actions, and no actions are
dispatched during playback, the piano stays frozen.

The fix introduces a new `AppAction::AdvanceProgressionChord` action and dispatches it from
`on_progression_click` in `app.rs` using `gloo_timers::callback::Timeout` — one timeout per
chord, spaced at 1-second intervals matching the audio schedule — so that `highlighted_chord`
advances in lockstep with the audio.

## Glossary

- **Bug_Condition (C)**: The condition that triggers the bug — an active progression exists but
  `highlighted_chord` does not match the chord at `active_progression.current_index`
- **Property (P)**: The desired behavior — `highlighted_chord` always equals
  `chord_at(active_progression.id, active_progression.current_index)`
- **Preservation**: Existing behaviors (next/prev arrows, single chord clicks, key selection,
  no-progression state) that must remain unchanged by the fix
- **`play_progression`**: The method in `src/audio/mod.rs` that schedules all chords of a
  progression onto the WebAudio timeline and returns immediately
- **`on_progression_click`**: The callback in `src/components/app.rs` that calls
  `audio.play_progression()` and dispatches `AppAction::SelectProgression`
- **`AdvanceProgressionChord`**: The new `AppAction` variant to be added — advances
  `active_progression.current_index` and updates `highlighted_chord` in the reducer
- **`active_progression`**: The `Option<ActiveProgression>` field in `AppState` tracking which
  progression is selected and which chord index is current

## Bug Details

### Bug Condition

The bug manifests when a progression is selected and audio begins playing. `play_progression()`
fires all notes onto the WebAudio timeline and returns. No subsequent state actions are dispatched,
so `active_progression.current_index` stays at 0 and `highlighted_chord` stays frozen on the
first chord for the entire duration of playback.

**Formal Specification:**
```
FUNCTION isBugCondition(state)
  INPUT: state of type AppState
  OUTPUT: boolean

  IF state.active_progression IS NONE THEN RETURN false
  
  LET ap = state.active_progression.unwrap()
  LET expected = chord_at(ap.id, ap.current_index)
  
  RETURN expected IS SOME
    AND state.highlighted_chord != expected
END FUNCTION
```

### Examples

- User selects "I–V–vi–IV" in C major. Audio plays C, G, Am, F in sequence. Piano stays on C
  major highlight for all 4 seconds instead of advancing to G, Am, F.
- User selects a 3-chord blues progression. After the first chord plays, the piano should show
  the IV chord highlight — instead it remains on the I chord.
- Edge case: progression with 1 chord — bug condition never triggers (index stays 0, highlight
  is correct). Fix must not break this.

## Expected Behavior

### Preservation Requirements

**Unchanged Behaviors:**
- Clicking the next (▶) or previous (◀) arrow buttons must continue to advance
  `active_progression.current_index` and update `highlighted_chord` correctly
- Clicking a single diatonic chord in the key info panel must continue to set `highlighted_chord`
  and clear `active_progression`
- When no progression is active, the piano must continue to show scale highlights (or none)
  based on the selected key alone
- Selecting a different key from the circle must continue to clear `active_progression` and
  `highlighted_chord`

**Scope:**
All inputs that do NOT involve progression playback timing (i.e., where `isBugCondition` is
false) must be completely unaffected by this fix. This includes:
- Mouse clicks on next/prev buttons
- Single chord clicks from the key info panel
- Key selection from the circle of fifths
- States where no progression is active

## Hypothesized Root Cause

1. **Fire-and-forget audio scheduling**: `AudioEngine::play_progression()` uses the WebAudio
   API's built-in timeline (`ctx.current_time() + i * 1.0`) to schedule all chords at once,
   then returns. There is no callback or event fired when each chord starts playing.

2. **No state dispatch during playback**: `on_progression_click` in `app.rs` calls
   `audio.play_progression(p)` and then dispatches `AppAction::SelectProgression(id)` which
   sets `current_index = 0`. After that, nothing dispatches further actions — so the reducer
   never advances `current_index` or updates `highlighted_chord`.

3. **Missing `AdvanceProgressionChord` action**: The reducer has `NextChord` and `PrevChord`
   for manual navigation, but no action for programmatic advancement during playback. The fix
   needs a new action (or reuse of `NextChord`) that can be dispatched from a timer callback.

4. **No timer-based synchronization**: Unlike the scale playback (which uses `Timeout` per note
   to drive `playing_note` state), progression playback has no equivalent timing mechanism to
   drive `highlighted_chord` updates.

## Correctness Properties

Property 1: Bug Condition - Highlighted Chord Tracks Active Progression Index

_For any_ `AppState` where `active_progression` is `Some(ap)` and `chord_at(ap.id, ap.current_index)`
is `Some(expected)`, the fixed system SHALL ensure `highlighted_chord == Some(expected)` — i.e.,
the piano highlight always reflects the chord at the current progression index, including during
audio playback.

**Validates: Requirements 2.1, 2.2, 2.3**

Property 2: Preservation - Non-Playback Behaviors Unchanged

_For any_ input where `isBugCondition` does NOT hold (manual next/prev navigation, single chord
clicks, key selection, no active progression), the fixed code SHALL produce exactly the same
`highlighted_chord` and `active_progression` state as the original code, preserving all existing
piano highlight behavior.

**Validates: Requirements 3.1, 3.2, 3.3, 3.4**

## Fix Implementation

### Changes Required

**File**: `src/state/mod.rs`

**Change 1 — New action variant**:
Add `AdvanceProgressionChord(usize)` to `AppAction`. The `usize` is the target chord index,
allowing the timer callback to be idempotent (only advances if the progression is still active
and the index matches what was scheduled).

**Change 2 — Reducer arm**:
Handle `AppAction::AdvanceProgressionChord(target_index)` in `app_reducer`:
```
AdvanceProgressionChord(target_index) =>
  IF active_progression IS SOME AND active_progression.current_index < target_index THEN
    SET active_progression.current_index = target_index
    SET highlighted_chord = chord_at(active_progression.id, target_index)
  ELSE
    RETURN state unchanged   // guard: user may have navigated away
  END IF
```

---

**File**: `src/components/app.rs`

**Change 3 — Timer dispatch in `on_progression_click`**:
After calling `audio.play_progression(p)` and dispatching `SelectProgression(id)`, schedule
one `Timeout` per chord (indices 1..len) at `i * 1000 ms` intervals. Each timeout dispatches
`AppAction::AdvanceProgressionChord(i)`. Use `.forget()` so the timeouts are not dropped.

```
for i in 1..progression.chords.len() {
    let state = state.clone();
    let id_copy = id;
    Timeout::new((i as u32) * 1000, move || {
        // Guard: only advance if this progression is still active
        state.dispatch(AppAction::AdvanceProgressionChord(i));
    }).forget();
}
```

**Change 4 — Guard in reducer (idempotency)**:
The reducer arm for `AdvanceProgressionChord` must check that `active_progression` is still
`Some` and that its `id` matches the scheduled progression before mutating state. This prevents
stale timeouts from corrupting state if the user selects a different progression or key before
the timers fire.

## Testing Strategy

### Validation Approach

Two-phase approach: first write exploratory tests that demonstrate the bug on unfixed code
(expected to fail), then verify the fix and preservation after implementation.

### Exploratory Bug Condition Checking

**Goal**: Surface counterexamples demonstrating that `highlighted_chord` does not update during
progression playback on the unfixed code. Confirm the root cause is the missing timer dispatch.

**Test Plan**: In the reducer unit tests, simulate the sequence of actions that playback would
produce if the fix were in place, and verify the current (unfixed) reducer does not handle
`AdvanceProgressionChord`. Also verify that calling `SelectProgression` followed by no further
actions leaves `highlighted_chord` frozen at index 0.

**Test Cases**:
1. **Frozen highlight test**: Dispatch `SelectProgression(id)`, then assert `highlighted_chord`
   equals chord at index 0. Dispatch nothing further. Assert `highlighted_chord` is still index 0
   even though "time has passed" — demonstrating the freeze. (Passes on unfixed code, confirms
   the bug exists by absence of advancement.)
2. **Missing action test**: Attempt to dispatch `AdvanceProgressionChord(1)` on unfixed code —
   this will fail to compile or be a no-op, confirming the action does not exist yet.
3. **Timer absence test**: Verify `on_progression_click` in `app.rs` does not schedule any
   `Timeout` calls for chord advancement (code inspection / grep).

**Expected Counterexamples**:
- After `SelectProgression`, `highlighted_chord` remains at index 0 indefinitely
- No `AdvanceProgressionChord` action exists in the unfixed codebase

### Fix Checking

**Goal**: Verify that for all inputs where the bug condition holds, the fixed reducer produces
the correct `highlighted_chord`.

**Pseudocode:**
```
FOR ALL (progression_id, target_index) WHERE isBugCondition holds DO
  state_0 ← app_reducer(default, SelectProgression(progression_id))
  state_1 ← app_reducer(state_0, AdvanceProgressionChord(target_index))
  ASSERT state_1.highlighted_chord = chord_at(progression_id, target_index)
  ASSERT state_1.active_progression.current_index = target_index
END FOR
```

### Preservation Checking

**Goal**: Verify that for all inputs where the bug condition does NOT hold, the fixed code
produces the same result as the original.

**Pseudocode:**
```
FOR ALL state WHERE NOT isBugCondition(state) DO
  ASSERT app_reducer_original(state, action).highlighted_chord
       = app_reducer_fixed(state, action).highlighted_chord
END FOR
```

**Testing Approach**: Property-based testing with `proptest` is recommended for preservation
checking because:
- It generates many random progression IDs, indices, and action sequences automatically
- It catches edge cases (single-chord progressions, out-of-range indices, stale timer guards)
- It provides strong guarantees that `NextChord`, `PrevChord`, `SelectChord`, and `SelectKey`
  are unaffected

**Test Cases**:
1. **Next/prev preservation**: Property test — for any valid progression state, dispatching
   `NextChord` or `PrevChord` produces the same `highlighted_chord` before and after the fix
2. **SelectChord preservation**: Dispatching `SelectChord` still sets `highlighted_chord` and
   clears `active_progression` identically
3. **SelectKey preservation**: Dispatching `SelectKey` still clears `active_progression` and
   `highlighted_chord` identically
4. **Stale timer guard**: Dispatch `SelectProgression(A)`, then `SelectProgression(B)`, then
   `AdvanceProgressionChord(1)` — assert state reflects progression B at index 0, not A at 1

### Unit Tests

- `advance_progression_chord_updates_highlight`: dispatch `SelectProgression` then
  `AdvanceProgressionChord(i)` for each valid index; assert `highlighted_chord` matches
- `advance_progression_chord_noop_when_no_active`: dispatch `AdvanceProgressionChord(1)` with
  no active progression; assert state unchanged
- `advance_progression_chord_noop_after_key_change`: select progression, select new key, then
  dispatch `AdvanceProgressionChord(1)`; assert `highlighted_chord` is `None`
- `advance_progression_chord_noop_for_stale_index`: dispatch `AdvanceProgressionChord` with an
  index already passed; assert state unchanged (idempotency guard)

### Property-Based Tests

- Generate random `ProgressionId` values and verify that for each valid index `i`,
  `AdvanceProgressionChord(i)` sets `highlighted_chord` to `chord_at(id, i)`
- Generate random action sequences not involving `AdvanceProgressionChord` and verify
  `highlighted_chord` behavior is identical between original and fixed reducer
- Generate random progressions and verify that `NextChord` / `PrevChord` round-trips still
  return to the original index and highlight

### Integration Tests

- Full playback simulation: dispatch `SelectProgression`, then simulate timer callbacks by
  dispatching `AdvanceProgressionChord(1)`, `AdvanceProgressionChord(2)`, etc. in sequence;
  assert `highlighted_chord` matches each chord in order
- Interruption test: start playback (dispatch `SelectProgression`), then manually click next
  before a timer fires; assert the manual navigation takes precedence and stale timer dispatch
  is a no-op
- Key change during playback: dispatch `SelectProgression`, then `SelectKey(new_key)`, then
  `AdvanceProgressionChord(1)`; assert `highlighted_chord` is `None` (key change cleared state)
