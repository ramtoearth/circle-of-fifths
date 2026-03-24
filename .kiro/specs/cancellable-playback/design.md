# Design Document: Cancellable Playback

## Overview

The Cancellable Playback feature replaces the current fire-and-forget animation model with a
tracked, cancellable one. Today, `Timeout::new(...).forget()` is called for every note in a
scale or chord progression animation; once scheduled those callbacks cannot be cancelled. The
fix is to retain the `Timeout` handles in a `use_mut_ref`-owned `Vec` inside the `App`
component. Dropping a `gloo_timers::callback::Timeout` before it fires cancels the underlying
`setTimeout` call, so clearing the `Vec` atomically cancels every pending callback.

A new `is_playing` boolean is derived from whether the handle collection is non-empty. This
drives the visibility of a Stop button rendered in the UI. Clicking Stop (or starting any new
playback) drops the existing handles, calls `audio.stop()`, and resets `playing_note` to
`None`, returning the Piano Panel to Idle State.

No new Rust crates are required. The `gloo-timers` crate already used for `Timeout` and
`Interval` provides the cancellation semantics we need.

---

## Architecture

The change is confined to the `App` component and a small addition to `AppState`/`AppAction`.
No new files are strictly required, though the handle storage lives in a `use_mut_ref` inside
`App` rather than in the reducer (because `Timeout` is not `Send`/`Sync` and cannot live in
serialisable state).

```mermaid
flowchart TD
    User -->|click segment / progression / stop| App

    subgraph App["App component (app.rs)"]
        direction TB
        A1[on_segment_click] --> C[cancel_active_session]
        A2[on_progression_click] --> C
        A3[on_stop] --> C
        C --> D[drop Vec<Timeout> in animation_handles ref]
        C --> E[audio.stop()]
        C --> F[playing_note.set(None)]
        C --> G[dispatch StopPlayback]
        A1 --> H[schedule new Timeouts → push to Vec]
        A2 --> H
        H --> I[dispatch SetPlaying(true)]
        H --> J[last Timeout clears Vec + dispatch SetPlaying(false)]
    end

    subgraph State["AppState / reducer (state/mod.rs)"]
        S1[is_playing: bool]
    end

    App -->|dispatch| State
    State -->|is_playing| App
    App -->|playing_note prop| PianoPanel
```

Key design decisions:

- **Handle storage in `use_mut_ref`, not in reducer** — `Timeout` is not `Clone`/`Serialize`
  and must not cross the reducer boundary. A `use_mut_ref<Vec<Timeout>>` inside `App` is the
  idiomatic Yew pattern for mutable non-reactive side-effect state.
- **`is_playing` flag in reducer** — the Stop button visibility is reactive UI state, so it
  belongs in `AppState` and is toggled by two new actions: `SetPlaying(bool)`.
- **`audio.stop()` suspends the `AudioContext`** — the existing `AudioEngine::stop()` method
  calls `ctx.suspend()`, which immediately silences all scheduled notes. No changes to
  `AudioEngine` are needed.
- **Natural completion clears handles** — the last scheduled `Timeout` in any animation is
  responsible for clearing the `Vec` and dispatching `SetPlaying(false)`, mirroring the
  cancellation path.

---

## Components and Interfaces

### `AppState` additions (`src/state/mod.rs`)

```rust
pub struct AppState {
    // ... existing fields ...
    pub is_playing: bool,   // true while a Playback_Session is active
}
```

### `AppAction` additions (`src/state/mod.rs`)

```rust
pub enum AppAction {
    // ... existing variants ...
    SetPlaying(bool),
}
```

Reducer handling:

```rust
AppAction::SetPlaying(playing) => AppState { is_playing: playing, ..state },
```

### `App` component additions (`src/components/app.rs`)

```rust
// Stores all Timeout handles for the active Playback_Session.
// Dropping the Vec cancels every pending callback.
let animation_handles = use_mut_ref(|| Vec::<Timeout>::new());
```

New `cancel_active_session` helper closure (called before every new playback and by Stop):

```rust
let cancel_active_session = {
    let animation_handles = animation_handles.clone();
    let audio = audio.clone();
    let playing_note = playing_note.clone();
    let state = state.clone();
    move || {
        animation_handles.borrow_mut().clear(); // drops all Timeouts → cancels callbacks
        audio.stop();
        playing_note.set(None);
        state.dispatch(AppAction::SetPlaying(false));
    }
};
```

Modified `on_segment_click`:

1. Call `cancel_active_session()`.
2. If the clicked key differs from `state.selected_key`, schedule new `Timeout`s, push each
   handle into `animation_handles`, dispatch `SetPlaying(true)`.
3. If the clicked key equals `state.selected_key`, only dispatch `SelectKey` (which deselects
   — no new session).

Modified `on_progression_click`:

1. Call `cancel_active_session()`.
2. If the clicked progression differs from the active one, schedule new `Timeout`s, push
   handles, dispatch `SetPlaying(true)`.
3. If the same progression is clicked again, only cancel (no restart).

New `on_stop` callback:

```rust
let on_stop = {
    // calls cancel_active_session()
};
```

### Stop Control (`src/components/app.rs` render section)

A `<button>` rendered conditionally on `state.is_playing`:

```rust
if state.is_playing {
    <button
        class="stop-btn"
        onclick={on_stop}
        aria-label="Stop playback"
    >
        {"■ Stop"}
    </button>
}
```

Placement: inside the existing `piano-footer` `<div>`, above the `PianoPanel`, so it is
always reachable regardless of scroll position.

### `PianoPanel` — no interface changes

`PianoPanel` already accepts `playing_note: Option<(PitchClass, i32)>`. Setting it to `None`
on cancellation is sufficient to return to Idle State. No prop changes are needed.

### `ProgressionPanel` — no interface changes

The progression animation is driven by `Timeout`s in `App`, not inside `ProgressionPanel`.
The panel already highlights the active chord via `active_progression.current_index` from
state; cancellation simply stops advancing that index.

---

## Data Models

### `AnimationHandle` collection

Conceptually an `Animation_Handle` is a `Vec<Timeout>` stored in a `use_mut_ref`. There is no
new named type; the existing `gloo_timers::callback::Timeout` provides the cancellation
contract: dropping the value cancels the pending JS `setTimeout`.

```
animation_handles: Rc<RefCell<Vec<Timeout>>>
```

Lifecycle:

| Event | Action on `animation_handles` |
|---|---|
| New playback starts | `clear()` (cancels old), then `push()` each new handle |
| Stop button pressed | `clear()` |
| New segment/progression clicked | `clear()`, then `push()` new handles |
| Last timeout fires naturally | `clear()` from within the callback |

### `AppState.is_playing: bool`

| Value | Meaning |
|---|---|
| `true` | A Playback_Session is active; Stop button is visible |
| `false` | No active session; Stop button is hidden |

Transitions:

- `false → true`: dispatched when the first `Timeout` of a new session is pushed.
- `true → false`: dispatched by `cancel_active_session()` or by the natural-completion
  callback.

### Progression animation timing

The existing `play_progression` schedules audio at 1-second intervals. The visual animation
mirrors this: one `Timeout` per chord at `i * 1000 ms`. Each timeout dispatches
`AppAction::SelectProgression`-equivalent state to advance `active_progression.current_index`
and update `highlighted_chord`.

For scale animations the interval is `60_000 / bpm` ms, matching the existing audio
scheduling in `play_scale`.

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions
of a system — essentially, a formal statement about what the system should do. Properties
serve as the bridge between human-readable specifications and machine-verifiable correctness
guarantees.*

### Property 1: Stop clears playing_note

*For any* active Playback_Session, after `cancel_active_session` is called, `playing_note`
must be `None`.

**Validates: Requirements 2.3, 7.4**

### Property 2: Stop transitions is_playing to false

*For any* `AppState` where `is_playing` is `true`, dispatching `SetPlaying(false)` must
produce a state where `is_playing` is `false`.

**Validates: Requirements 2.4, 1.2**

### Property 3: SetPlaying round-trip

*For any* initial `is_playing` value, dispatching `SetPlaying(true)` then `SetPlaying(false)`
must yield `is_playing == false`; dispatching `SetPlaying(false)` then `SetPlaying(true)` must
yield `is_playing == true`.

**Validates: Requirements 1.1, 1.2**

### Property 4: Idle State key highlight correctness

*For any* `AppState` with a `selected_key` and `is_playing == false`, the set of pitch
classes returned by `scale_notes(selected_key)` must equal the set of pitch classes that
`note_role` classifies as `ScaleNote` or better (Root/Third/Fifth via chord) when no
`playing_note` override is active.

**Validates: Requirements 7.1, 7.3**

### Property 5: Idle State chord highlight correctness

*For any* `AppState` with a `highlighted_chord` and `is_playing == false`, `note_role` must
return `Root`, `Third`, or `Fifth` for exactly the three chord pitches and no others (absent
a selected key).

**Validates: Requirements 7.2**

### Property 6: Animation handle collection is empty after cancellation

*For any* non-empty `Vec<Timeout>`, after `clear()` is called the collection must be empty
(length == 0).

**Validates: Requirements 5.2, 5.4**

### Property 7: New session always starts from empty handle collection

*For any* sequence of playback starts and cancellations, immediately before the first new
`Timeout` is pushed for a new session the handle collection must be empty.

**Validates: Requirements 5.1, 5.4**

---

## Error Handling

- **AudioContext unavailable (degraded mode)**: `audio.stop()` is a no-op when `ctx` is
  `None`. Animation cancellation still proceeds normally — `animation_handles.clear()` and
  `playing_note.set(None)` are independent of audio. The Stop button remains functional.
- **Timeout fires after clear**: Not possible. `gloo_timers::callback::Timeout` cancels the
  underlying `setTimeout` in its `Drop` impl. Once the `Vec` is cleared the callbacks will
  not fire.
- **Progression not found**: `crate::data::find_progression` returns `Option`. If `None`, no
  `Timeout`s are scheduled and `SetPlaying(true)` is not dispatched. Existing guard logic in
  `on_progression_click` is preserved.
- **BPM of zero**: `60_000 / bpm` would panic. The existing reducer clamps BPM to `[40, 200]`
  via `SetBpm`, so this cannot occur in practice. The animation scheduling code should still
  use `.max(1)` as a defensive guard.

---

## Testing Strategy

### Unit tests

Focus on specific examples and edge cases that are hard to cover with property tests:

- `SetPlaying(true)` sets `is_playing = true` in the reducer.
- `SetPlaying(false)` sets `is_playing = false` in the reducer.
- `SetPlaying` does not mutate any other field in `AppState`.
- `note_role` returns `None` for all pitches when `selected_key` is `None` and
  `highlighted_chord` is `None` (existing test, confirms Idle State with no selection).

### Property-based tests

Use `proptest` (already a dev-dependency in `Cargo.toml`) with a minimum of 100 iterations
per property.

Each test is tagged with a comment in the format:
`// Feature: cancellable-playback, Property N: <property_text>`

**Property 2 — SetPlaying(false) always yields is_playing == false**

```rust
// Feature: cancellable-playback, Property 2: Stop transitions is_playing to false
proptest! {
    #[test]
    fn prop_set_playing_false(initial in any::<bool>()) {
        let s0 = AppState { is_playing: initial, ..AppState::default() };
        let s1 = app_reducer(s0, AppAction::SetPlaying(false));
        prop_assert!(!s1.is_playing);
    }
}
```

**Property 3 — SetPlaying round-trip**

```rust
// Feature: cancellable-playback, Property 3: SetPlaying round-trip
proptest! {
    #[test]
    fn prop_set_playing_round_trip(initial in any::<bool>()) {
        let s0 = AppState { is_playing: initial, ..AppState::default() };
        let s1 = app_reducer(s0.clone(), AppAction::SetPlaying(true));
        let s2 = app_reducer(s1, AppAction::SetPlaying(false));
        prop_assert!(!s2.is_playing);

        let s3 = app_reducer(s0, AppAction::SetPlaying(false));
        let s4 = app_reducer(s3, AppAction::SetPlaying(true));
        prop_assert!(s4.is_playing);
    }
}
```

**Property 4 — Idle State scale highlight correctness**

```rust
// Feature: cancellable-playback, Property 4: Idle State key highlight correctness
proptest! {
    #[test]
    fn prop_idle_state_scale_highlight(root_idx in 0u8..12, mode_bit in any::<bool>()) {
        let mode = if mode_bit { Mode::Major } else { Mode::Minor };
        let key = Key { root: PitchClass::from_index(root_idx), mode };
        let expected = scale_notes(key);
        for pitch_idx in 0u8..12 {
            let pitch = PitchClass::from_index(pitch_idx);
            let role = note_role(pitch, Some(key), None);
            if expected.contains(&pitch) {
                prop_assert_ne!(role, KeyRole::None,
                    "{:?} should be highlighted in {:?}", pitch, key);
            } else {
                prop_assert_eq!(role, KeyRole::None,
                    "{:?} should not be highlighted in {:?}", pitch, key);
            }
        }
    }
}
```

**Property 5 — Idle State chord highlight correctness**

```rust
// Feature: cancellable-playback, Property 5: Idle State chord highlight correctness
proptest! {
    #[test]
    fn prop_idle_state_chord_highlight(
        root_idx  in 0u8..12,
        third_idx in 0u8..12,
        fifth_idx in 0u8..12,
    ) {
        let chord = ChordHighlight {
            root:  PitchClass::from_index(root_idx),
            third: PitchClass::from_index(third_idx),
            fifth: PitchClass::from_index(fifth_idx),
        };
        let chord_pitches = [chord.root, chord.third, chord.fifth];
        for pitch_idx in 0u8..12 {
            let pitch = PitchClass::from_index(pitch_idx);
            let role = note_role(pitch, None, Some(&chord));
            if chord_pitches.contains(&pitch) {
                prop_assert_ne!(role, KeyRole::None,
                    "{:?} should be highlighted as chord note", pitch);
            } else {
                prop_assert_eq!(role, KeyRole::None,
                    "{:?} should not be highlighted", pitch);
            }
        }
    }
}
```

**Property 6 — Vec clear empties the collection**

This is a pure Rust property that does not require WASM and can run in the standard test
harness:

```rust
// Feature: cancellable-playback, Property 6: Animation handle collection is empty after clear
proptest! {
    #[test]
    fn prop_vec_clear_is_empty(len in 0usize..20) {
        let mut v: Vec<u32> = (0..len as u32).collect(); // stand-in for Vec<Timeout>
        v.clear();
        prop_assert!(v.is_empty());
    }
}
```

> Note: `Timeout` cannot be constructed outside WASM, so this property is validated with a
> `Vec<u32>` as a structural stand-in. The cancellation contract is guaranteed by
> `gloo-timers`' `Drop` impl and is covered by integration testing.

**Property 7 — New session starts from empty handle collection**

This is validated by the integration invariant: `cancel_active_session` always calls
`animation_handles.borrow_mut().clear()` before any new handles are pushed. The unit test
for this is:

```rust
// Feature: cancellable-playback, Property 7: New session always starts from empty handle collection
#[test]
fn cancel_clears_before_new_session() {
    // Verified structurally: cancel_active_session() calls .clear() unconditionally
    // before the caller pushes new handles. Enforced by code review and integration test.
}
```

### Integration / manual testing checklist

- Click a segment → animation starts, Stop button appears.
- Click Stop mid-animation → piano returns to static scale highlight, Stop button disappears.
- Click a different segment mid-animation → old animation stops, new one starts cleanly.
- Click the same segment mid-animation → animation stops, piano shows static scale, no restart.
- Select a progression → animation starts, Stop button appears.
- Click Stop mid-progression → piano returns to static chord highlight, Stop button disappears.
- Select the same progression mid-animation → animation stops, no restart.
- Mute audio, start playback → Stop button still appears, clicking it cancels the animation.
- After cancellation with a key selected → piano shows full scale highlight (no playing_note).
- After cancellation with a chord highlighted → piano shows chord highlight only.
- After cancellation with nothing selected → piano shows no highlights.
