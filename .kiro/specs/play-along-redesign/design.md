# Design Document: Play-Along Redesign

## Overview

The redesigned Play-Along Mode replaces the BPM-timer model with a wait-based approach. The core
change is removing `gloo_timers::Interval` from `PlayAlongPanel` and replacing it with a
Chord_Detection effect that watches `held_notes`. A new Hand_Position_Overlay renders finger
indicators inside the existing `PianoPanel` keys. The progression loops indefinitely instead of
ending. The per-chord result list and scoring infrastructure (`PlayAlongScore`, `ChordResult`,
`RecordPlayAlongChordResult`) are removed entirely.

---

## Architecture

The existing unidirectional data flow (`AppState` + `AppAction` via `use_reducer`) is preserved.
The changes are:

```
MIDI NoteOn/NoteOff
        │
        ▼
 AppAction::MidiNoteOn / MidiNoteOff
        │
        ▼
  AppState.held_notes updated
        │
        ▼
  PlayAlongPanel use_effect_with(held_notes, play_along_state)
        │
        ├── chord matches? start 300ms debounce timer (use_timeout)
        │       │
        │       └── still matches after 300ms? dispatch PlayAlongChordCorrect
        │
        └── chord doesn't match? cancel pending debounce timer
```

The metronome is NOT auto-enabled in the new mode (requirement 4.3: BPM is irrelevant).
`EnterPlayAlong` no longer saves/forces `metronome_active`.

---

## Data Model Changes

### `PlayAlongState` (simplified — in `src/state/mod.rs`)

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct PlayAlongState {
    pub progression_id: ProgressionId,
    pub current_chord_index: usize,
    pub chords_played: u32,       // total correct plays since session start (for display only)
    pub showing_loop_cue: bool,   // true during the 1.5s loop indicator flash
}
```

Removed fields vs. old design:
- `score: PlayAlongScore` — eliminated (no scoring)
- `started_at_ms: f64` — eliminated (no timing)
- `pre_play_along_metronome_active: bool` — eliminated (metronome no longer auto-managed)

### Removed types

`PlayAlongScore` and `ChordResult` in `src/midi/mod.rs` are deleted. No new types are needed.

### `AppAction` changes

Removed variants:
- `PlayAlongTick` — timer-driven; replaced by chord detection
- `RecordPlayAlongChordResult(ChordResult)` — scoring; no longer needed

New variant:
```rust
PlayAlongChordCorrect,   // all target PitchClasses held for 300ms → advance chord, wrap at end
PlayAlongLoopCueDone,    // 1.5s loop cue timer expired → clear showing_loop_cue
```

`EnterPlayAlong(ProgressionId)` is kept unchanged (entry point is the same).
`ExitPlayAlong` is kept unchanged (exit behavior is the same).

### Reducer arms

**`EnterPlayAlong(id)`** — simplified:
```rust
AppAction::EnterPlayAlong(id) => {
    if state.midi_status != MidiStatus::Connected { return state; }
    AppState {
        app_mode: AppMode::PlayAlong,
        play_along_state: Some(PlayAlongState {
            progression_id: id,
            current_chord_index: 0,
            chords_played: 0,
            showing_loop_cue: false,
        }),
        // metronome_active is NOT changed (no longer forced on)
        ..state
    }
}
```

**`PlayAlongChordCorrect`**:
```rust
AppAction::PlayAlongChordCorrect => {
    let Some(ref pa) = state.play_along_state else { return state; };
    let Some(progression) = crate::data::find_progression(pa.progression_id) else { return state; };
    let chord_count = progression.chords.len();
    let next_index = (pa.current_chord_index + 1) % chord_count;
    let looped = next_index == 0 && pa.current_chord_index == chord_count - 1;
    let highlighted_chord = chord_highlight_at(&progression, next_index);
    AppState {
        play_along_state: Some(PlayAlongState {
            current_chord_index: next_index,
            chords_played: pa.chords_played + 1,
            showing_loop_cue: looped,
            ..pa.clone()
        }),
        highlighted_chord,
        ..state
    }
}
```

**`PlayAlongLoopCueDone`**:
```rust
AppAction::PlayAlongLoopCueDone => {
    let Some(ref pa) = state.play_along_state else { return state; };
    AppState {
        play_along_state: Some(PlayAlongState {
            showing_loop_cue: false,
            ..pa.clone()
        }),
        ..state
    }
}
```

**`ExitPlayAlong`** — simplified (no metronome restore):
```rust
AppAction::ExitPlayAlong => AppState {
    app_mode: AppMode::Normal,
    play_along_state: None,
    ..state
}
```

---

## Components

### `PlayAlongPanel` (rewritten — `src/components/play_along_panel.rs`)

The interval-based beat timer is removed entirely. The component now:

1. Derives the target chord from `progression` and `current_chord_index`.
2. Runs a `use_effect_with((held_notes, current_chord_index), ...)` that:
   - Computes whether all target PitchClasses are in `held_notes` (octave-agnostic).
   - If yes: starts a 300 ms `gloo_timers::callback::Timeout` that dispatches
     `PlayAlongChordCorrect` when it fires.
   - Drops the timeout handle when held notes change before it fires (cancellation).
3. Runs a separate `use_effect_with(showing_loop_cue, ...)` that, when `showing_loop_cue`
   becomes `true`, starts a 1.5 s timeout dispatching `PlayAlongLoopCueDone`.
4. Renders: chord name + Roman numeral, position indicator ("Chord N of M"), optional loop cue
   banner, and a Stop button. Does NOT render a BPM control or result list.
5. Passes `practice_target` (the target PitchClasses) to `PianoPanel` as before, so existing
   green/red key highlighting continues to work.

Props (trimmed):
```rust
pub struct PlayAlongPanelProps {
    pub progression: Progression,
    pub current_chord_index: usize,
    pub chords_played: u32,
    pub showing_loop_cue: bool,
    pub held_notes: Vec<HeldNote>,
    pub on_stop: Callback<()>,
    pub on_chord_correct: Callback<()>,      // dispatches PlayAlongChordCorrect
    pub on_loop_cue_done: Callback<()>,      // dispatches PlayAlongLoopCueDone
}
```

Removed props vs. old design: `bpm`, `score`, `on_tick`, `on_record_result`.

#### Chord detection logic (pure function, testable)

```rust
/// Returns true if every PitchClass in `target` appears in at least one HeldNote,
/// ignoring octave. Used by PlayAlongPanel to detect a correctly played chord.
pub fn chord_fully_held(target: &[PitchClass], held: &[HeldNote]) -> bool {
    let held_pcs: HashSet<PitchClass> = held.iter().map(|n| n.pitch_class).collect();
    target.iter().all(|pc| held_pcs.contains(pc))
}
```

The 300 ms debounce is implemented with a `use_mut_ref<Option<gloo_timers::callback::Timeout>>`
inside `PlayAlongPanel`:

```rust
let pending_timeout = use_mut_ref(|| Option::<gloo_timers::callback::Timeout>::None);

use_effect_with((props.held_notes.clone(), props.current_chord_index), {
    let on_chord_correct = props.on_chord_correct.clone();
    let pending_timeout = pending_timeout.clone();
    let target_pcs = target_pcs.clone(); // derived from current_chord_index

    move |(held_notes, _idx)| {
        // Cancel any pending advance first
        *pending_timeout.borrow_mut() = None;

        if chord_fully_held(&target_pcs, held_notes) {
            let cb = on_chord_correct.clone();
            let timeout = gloo_timers::callback::Timeout::new(300, move || {
                cb.emit(());
            });
            *pending_timeout.borrow_mut() = Some(timeout);
        }

        // Cleanup: drop the timeout (cancels it) when deps change or component unmounts
        let pending_timeout = pending_timeout.clone();
        move || { *pending_timeout.borrow_mut() = None; }
    }
});
```

### `PianoPanel` (extended — `src/components/piano_panel.rs`)

Add a new prop `finger_hints: Option<Vec<FingerHint>>` passed from `App` when in play-along mode.

```rust
/// A finger placement guide for a single piano key.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FingerHint {
    pub pitch_class: PitchClass,
    pub finger: u8,   // 1 = thumb, 3 = middle, 5 = pinky
    pub held: bool,   // true when the note is currently in Held_Notes
}
```

The `FingerHint` list is computed in `App` (not in `PianoPanel`) from the current target chord
and held notes:

```rust
fn finger_hints_for_chord(chord: &DiatonicChord, held: &[HeldNote]) -> Vec<FingerHint> {
    let held_pcs: HashSet<PitchClass> = held.iter().map(|n| n.pitch_class).collect();
    // Root-position fingering: notes[0]=root→1, notes[1]=third→3, notes[2]=fifth→5
    let fingers = [1u8, 3u8, 5u8];
    chord.notes.iter().zip(fingers.iter()).map(|(&pc, &finger)| {
        FingerHint { pitch_class: pc, finger, held: held_pcs.contains(&pc) }
    }).collect()
}
```

Inside `PianoPanel`, when rendering each key element, check if its `PitchClass` has a matching
`FingerHint`. If so, render a child `<div class="finger-hint [finger-hint--held]">N</div>`:

```rust
let finger_hint = props.finger_hints.as_deref().and_then(|hints| {
    hints.iter().find(|h| h.pitch_class == pitch)
});

html! {
    <div class={classes} style={style} key={i as u32}>
        if let Some(hint) = finger_hint {
            <div class={if hint.held { "finger-hint finger-hint--held" } else { "finger-hint" }}>
                { hint.finger.to_string() }
            </div>
        }
        if show_labels {
            <span class="piano-key__label">{ label }</span>
        }
    </div>
}
```

### `App` (updated — `src/components/app.rs`)

Changes needed:
1. Remove `on_tick` and `on_record_result` callbacks wired to `PlayAlongPanel`.
2. Add `on_chord_correct` callback dispatching `AppAction::PlayAlongChordCorrect`.
3. Add `on_loop_cue_done` callback dispatching `AppAction::PlayAlongLoopCueDone`.
4. Compute `finger_hints` from `play_along_state` + `held_notes` and pass to `PianoPanel`.
5. In `EnterPlayAlong` wiring: do NOT force `metronome_active = true`.
6. Play the target chord audio preview on chord advance: add a `use_effect_with(current_chord_index, ...)` that, when in play-along mode, calls `audio_engine.play_chord(target_notes)` unless muted.

---

## CSS Changes (`index.css`)

### Finger hint base style

```css
.finger-hint {
    position: absolute;
    top: -30px;
    left: 50%;
    transform: translateX(-50%);
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background-color: var(--finger-hint-bg, rgba(120, 180, 255, 0.85));
    color: var(--finger-hint-text, #111);
    font-size: 13px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 10;
    border: 2px solid rgba(255, 255, 255, 0.6);
    box-shadow: 0 2px 6px rgba(0, 0, 0, 0.25);
    transition: background-color 0.15s ease, transform 0.1s ease;
}

.finger-hint--held {
    background-color: var(--finger-hint-held-bg, rgba(80, 220, 120, 0.95));
    transform: translateX(-50%) scale(1.15);
}
```

`piano-key` already needs `position: relative` for black-key absolute positioning; confirm it is
set (it is, via `.piano-key--black { position: absolute; ... }`). White keys need
`position: relative` added if not present.

### Loop cue banner

```css
.play-along__loop-cue {
    background: var(--accent-color, #7cb8ff);
    color: #000;
    text-align: center;
    font-weight: 700;
    border-radius: 6px;
    padding: 6px 16px;
    font-size: 1.1em;
    animation: loop-cue-fade 1.5s ease forwards;
}

@keyframes loop-cue-fade {
    0%   { opacity: 1; }
    70%  { opacity: 1; }
    100% { opacity: 0; }
}
```

---

## Correctness Properties

### Property 1: Octave-agnostic chord detection

*For any* Target_Chord with PitchClasses {P1, P2, P3} and any set of HeldNotes that contains each
Pi in any octave, `chord_fully_held` must return `true`. If any Pi is absent, it must return `false`.

**Validates: Requirement 1.5**

---

### Property 2: Debounce cancellation on note release

*For any* state where Chord_Detection has succeeded and the debounce timer is pending, releasing
any target note (removing its PitchClass from HeldNotes) before 300 ms elapses must cancel the
timer and NOT dispatch `PlayAlongChordCorrect`.

**Validates: Requirement 1.4**

---

### Property 3: Progression wrap-around

*For any* progression of length N > 0, dispatching `PlayAlongChordCorrect` from
`current_chord_index == N - 1` must produce `current_chord_index == 0` and `showing_loop_cue == true`.

**Validates: Requirement 3.1**

---

### Property 4: Wrap-around never terminates

*For any* succession of `PlayAlongChordCorrect` dispatches starting from index 0 in a progression
of length N, `app_mode` must remain `AppMode::PlayAlong` after every dispatch (no automatic exit).

**Validates: Requirement 3.2**

---

### Property 5: `chord_fully_held` with empty target

*For* an empty target slice, `chord_fully_held(&[], held)` must return `true` for any `held`
(vacuous truth). This prevents an empty chord from stalling the mode.

**Validates: Requirement 1.1 (edge case)**

---

### Property 6: ExitPlayAlong clears state

*For any* AppState where `app_mode == AppMode::PlayAlong`, dispatching `ExitPlayAlong` must
produce a state where `app_mode == AppMode::Normal` and `play_along_state == None`.

**Validates: Requirement 5.3**

---

## Error Handling

### No active progression on entry

`EnterPlayAlong(id)` looks up the progression via `crate::data::find_progression(id)`. If not
found (should not happen in practice), the reducer returns state unchanged. The "Play Along" button
is only rendered when a progression is active, so this is a defense-in-depth guard.

### MIDI device disconnect during session

`MidiDevicesChanged` with an empty list continues to clear `held_notes`. A `use_effect_with` in
`App` watching `midi_status` dispatches `ExitPlayAlong` when `midi_status` drops below
`MidiStatus::Connected` while `app_mode == AppMode::PlayAlong`.

### Empty progression (zero chords)

`chord_fully_held` on an empty target returns `true` (vacuous), which would cause infinite rapid
advancing. Guard in the reducer: if `chord_count == 0`, `EnterPlayAlong` returns state unchanged.

---

## Testing Strategy

### Unit tests (cargo test)

All new pure-Rust functions live in `src/components/play_along_panel.rs` and
`src/state/mod.rs` and are tested without WASM.

- `chord_fully_held` — multiple pitch classes held in various octaves
- `chord_fully_held` — missing note returns false
- `chord_fully_held` — empty target returns true
- `finger_hints_for_chord` — correct finger numbers assigned (1, 3, 5)
- `finger_hints_for_chord` — `held` field true only for actually held pitch classes
- `PlayAlongChordCorrect` reducer — index increments correctly for mid-progression chords
- `PlayAlongChordCorrect` reducer — wraps to 0 and sets `showing_loop_cue = true` at end
- `PlayAlongLoopCueDone` reducer — clears `showing_loop_cue`
- `ExitPlayAlong` reducer — mode resets to Normal, `play_along_state` becomes None

### Property-based tests (proptest, cargo test)

Tag format: `// Feature: play-along-redesign, Property N: <text>`

- **Property 1**: `chord_fully_held` — octave-agnostic detection across all MIDI note ranges
- **Property 3**: wrap-around — `PlayAlongChordCorrect` from last index always lands on 0
- **Property 4**: mode never exits automatically after any number of `PlayAlongChordCorrect`
  dispatches
- **Property 5**: `chord_fully_held(&[], _)` always true
- **Property 6**: `ExitPlayAlong` always resets mode and clears state
