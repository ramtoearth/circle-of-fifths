# Design Document: Custom Progression Builder

## Overview

The Custom Progression Builder adds a new `AppMode` variant and a matching UI panel. The user
assembles a `Vec<ScaleDegree>` from the diatonic chords of the selected key. When they start
play-along, the reducer constructs an ephemeral `Progression` value (no database ID needed) and
passes it directly into the existing `PlayAlongState` machinery. No changes to the music theory
layer, audio engine, or MIDI pipeline are required — the builder plugs into the existing
unidirectional data flow.

---

## Architecture

```
User clicks Diatonic_Chord_Tile
        │
        ▼
AppAction::BuilderAppend(ScaleDegree) or BuilderPop(ScaleDegree) or BuilderShiftAppend(ScaleDegree)
        │
        ▼
  AppState.builder_progression: Vec<ScaleDegree> updated
        │
        ▼
  CustomProgressionBuilderPanel re-renders
        │
        └── "Start Play Along" clicked
                │
                ▼
        AppAction::EnterPlayAlongCustom
                │
                ▼
        PlayAlongState created with ephemeral Progression
                │
                ▼
        Existing PlayAlongPanel + PianoPanel (unchanged)
```

The existing `EnterPlayAlong(ProgressionId)` action is kept for predefined progressions. A new
`EnterPlayAlongCustom` action creates the `PlayAlongState` directly from `builder_progression`
without a database ID.

---

## Data Model Changes

### New `AppMode` variant (in `src/state/mod.rs`)

```rust
pub enum AppMode {
    Normal,
    PlayAlong,
    CustomProgressionBuilder,   // ← new
}
```

### New `AppState` field (in `src/state/mod.rs`)

```rust
pub struct AppState {
    // ... existing fields ...
    pub builder_progression: Vec<ScaleDegree>,   // ← new; ephemeral, not persisted
}
```

The field is `Vec<ScaleDegree>` — the same element type used by `Progression.chords`. It is not
persisted to `PersistedState`; every time the user opens the builder they start fresh
(Requirement 1.3, 8.4).

### New `AppAction` variants (in `src/state/mod.rs`)

```rust
AppAction::EnterBuilder,                          // switch to CustomProgressionBuilder mode
AppAction::ExitBuilder,                           // return to Normal mode, discard builder_progression
AppAction::BuilderToggle(ScaleDegree),            // plain click: pop last occurrence if present, else append
AppAction::BuilderShiftAppend(ScaleDegree),       // shift+click: always append
AppAction::BuilderReset,                          // clear builder_progression
AppAction::EnterPlayAlongCustom,                  // start play-along from builder_progression
```

### Ephemeral `Progression` for play-along

`EnterPlayAlongCustom` creates a `Progression` on the fly:

```rust
AppAction::EnterPlayAlongCustom => {
    if state.midi_status != MidiStatus::Connected { return state; }
    if state.builder_progression.is_empty() { return state; }
    let key = match state.selected_key { Some(k) => k, None => return state };

    // Use a sentinel ID that will never clash with the 60 predefined IDs (0–59)
    let custom_progression = Progression {
        id: u32::MAX,
        key,
        chords: state.builder_progression.clone(),
        tags: vec![ProgressionTag::Custom],
        borrowed_chord: None,
    };

    let highlighted_chord = chord_highlight_at(&custom_progression, 0);
    AppState {
        app_mode: AppMode::PlayAlong,
        play_along_state: Some(PlayAlongState {
            progression: custom_progression,   // see below
            current_chord_index: 0,
            chords_played: 0,
            showing_loop_cue: false,
        }),
        highlighted_chord,
        ..state
    }
}
```

### `PlayAlongState` stores `Progression` directly

Currently `PlayAlongState` holds a `progression_id: ProgressionId`, and the reducer/component
looks up the `Progression` via `data::find_progression(id)`. This lookup only works for predefined
progressions. To support custom ones without a database ID, change `PlayAlongState` to store the
`Progression` value inline:

```rust
pub struct PlayAlongState {
    pub progression: Progression,        // ← was progression_id: ProgressionId
    pub current_chord_index: usize,
    pub chords_played: u32,
    pub showing_loop_cue: bool,
}
```

All reducer arms and components that call `data::find_progression(pa.progression_id)` are updated
to use `pa.progression` directly. This is a search-and-replace refactor — logic is unchanged.

### Reducer arms

**`EnterBuilder`**:
```rust
AppAction::EnterBuilder => AppState {
    app_mode: AppMode::CustomProgressionBuilder,
    builder_progression: vec![],
    ..state
}
```

**`ExitBuilder`**:
```rust
AppAction::ExitBuilder => AppState {
    app_mode: AppMode::Normal,
    builder_progression: vec![],
    ..state
}
```

**`BuilderToggle(degree)`** — plain click: pop last occurrence if present, else append:
```rust
AppAction::BuilderToggle(degree) => {
    let mut chords = state.builder_progression.clone();
    if let Some(pos) = chords.iter().rposition(|&d| d == degree) {
        chords.remove(pos);
    } else if chords.len() < 16 {
        chords.push(degree);
    }
    AppState { builder_progression: chords, ..state }
}
```

**`BuilderShiftAppend(degree)`** — shift+click: always append (up to 16):
```rust
AppAction::BuilderShiftAppend(degree) => {
    let mut chords = state.builder_progression.clone();
    if chords.len() < 16 {
        chords.push(degree);
    }
    AppState { builder_progression: chords, ..state }
}
```

**`BuilderReset`**:
```rust
AppAction::BuilderReset => AppState {
    builder_progression: vec![],
    ..state
}
```

---

## Components

### New: `CustomProgressionBuilderPanel` (`src/components/custom_progression_builder.rs`)

```rust
#[derive(Properties, PartialEq)]
pub struct CustomProgressionBuilderProps {
    pub selected_key: Key,
    pub working_progression: Vec<ScaleDegree>,
    pub midi_status: MidiStatus,
    pub on_toggle: Callback<ScaleDegree>,           // plain click
    pub on_shift_append: Callback<ScaleDegree>,     // shift+click
    pub on_reset: Callback<()>,
    pub on_start_play_along: Callback<()>,
    pub on_back: Callback<()>,
}
```

Render layout (top to bottom):

1. **Header** — "Build Your Progression" title + Back button (Requirement 8.1)
2. **Working progression display** — ordered list of chord slots:
   - Each slot: `"{roman_numeral} – {chord_name}"` (e.g., "I – C", "V – G")
   - Empty state: placeholder message "Click a chord below to start" (Requirement 6.3)
3. **Chord tile grid** — 7 tiles, one per diatonic chord:
   - Each tile shows Roman numeral + chord name (e.g., "I\nC", "V\nG")
   - `onclick` dispatches `on_toggle` with the `ScaleDegree` if `!shift_key()`, else `on_shift_append`
   - Tile badge: count of how many times this degree appears in `working_progression` (0 = no badge)
4. **Action row** — Reset button + "Start Play Along" button
   - "Start Play Along" disabled when `working_progression.is_empty()` or `midi_status != Connected`

Click handler inside the tile:
```rust
let onclick = {
    let degree = chord.degree;
    let on_toggle = props.on_toggle.clone();
    let on_shift_append = props.on_shift_append.clone();
    Callback::from(move |e: MouseEvent| {
        if e.shift_key() {
            on_shift_append.emit(degree);
        } else {
            on_toggle.emit(degree);
        }
    })
};
```

### `App` changes (`src/components/app.rs`)

1. Add `on_enter_builder` callback → `dispatch(AppAction::EnterBuilder)`
2. Add `on_exit_builder` callback → `dispatch(AppAction::ExitBuilder)`
3. Add `on_builder_toggle` callback → `dispatch(AppAction::BuilderToggle(d))`
4. Add `on_builder_shift_append` callback → `dispatch(AppAction::BuilderShiftAppend(d))`
5. Add `on_builder_reset` callback → `dispatch(AppAction::BuilderReset)`
6. Add `on_start_play_along_custom` callback → `dispatch(AppAction::EnterPlayAlongCustom)`
7. Conditionally render `CustomProgressionBuilderPanel` when `state.app_mode == AppMode::CustomProgressionBuilder`
8. Add "Build Custom" button to `ProgressionPanel` (or alongside it) when `selected_key.is_some()`; this button dispatches `on_enter_builder`
9. Wire `ExitPlayAlong` to return to `CustomProgressionBuilder` mode (not `Normal`) when play-along was entered via `EnterPlayAlongCustom`. To track this, add a boolean flag `play_along_from_builder: bool` to `PlayAlongState`:

```rust
pub struct PlayAlongState {
    pub progression: Progression,
    pub current_chord_index: usize,
    pub chords_played: u32,
    pub showing_loop_cue: bool,
    pub from_builder: bool,   // ← new; true if entered via EnterPlayAlongCustom
}
```

`ExitPlayAlong` reducer:
```rust
AppAction::ExitPlayAlong => {
    let from_builder = state.play_along_state
        .as_ref()
        .map(|s| s.from_builder)
        .unwrap_or(false);
    AppState {
        app_mode: if from_builder { AppMode::CustomProgressionBuilder } else { AppMode::Normal },
        play_along_state: None,
        ..state
    }
}
```

### `ProgressionPanel` changes (`src/components/progression_panel.rs`)

Add a new prop:
```rust
pub on_enter_builder: Callback<()>,
```

Render a "Build Custom" button below (or above) the progression list, visible only when
`selected_key.is_some()`.

---

## CSS Changes (`index.css`)

### Builder panel

```css
.builder-panel {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding: 12px;
}

.builder-panel__header {
    display: flex;
    align-items: center;
    justify-content: space-between;
}

.builder-panel__slots {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    min-height: 40px;
}

.builder-panel__slot {
    background: var(--chord-tile-bg, rgba(120, 180, 255, 0.2));
    border: 1px solid var(--chord-tile-border, rgba(120, 180, 255, 0.5));
    border-radius: 4px;
    padding: 4px 8px;
    font-size: 0.85em;
}

.builder-panel__placeholder {
    color: var(--text-muted, #888);
    font-style: italic;
    font-size: 0.9em;
}
```

### Chord tiles

```css
.chord-tiles {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 6px;
}

.chord-tile {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 8px 4px;
    border-radius: 6px;
    border: 2px solid var(--chord-tile-border, rgba(120, 180, 255, 0.4));
    background: var(--chord-tile-bg, rgba(120, 180, 255, 0.1));
    cursor: pointer;
    font-weight: 600;
    position: relative;
    transition: background 0.1s ease;
}

.chord-tile:hover {
    background: var(--chord-tile-hover-bg, rgba(120, 180, 255, 0.25));
}

.chord-tile__badge {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 18px;
    height: 18px;
    border-radius: 50%;
    background: var(--accent-color, #7cb8ff);
    color: #000;
    font-size: 11px;
    font-weight: 700;
    display: flex;
    align-items: center;
    justify-content: center;
}
```

---

## Correctness Properties

### Property 1: Toggle idempotence — append then pop

*For any* `ScaleDegree` D and empty `builder_progression`, dispatching `BuilderToggle(D)` twice
(first Append, then Pop) must produce an empty `builder_progression`.

**Validates: Requirements 2.1, 3.1**

---

### Property 2: ShiftAppend always grows the list

*For any* `builder_progression` of length < 16, dispatching `BuilderShiftAppend(D)` must produce
a list of length + 1 with D at the end, regardless of existing occurrences.

**Validates: Requirement 4.1**

---

### Property 3: 16-chord cap

*For any* `builder_progression` of length 16, dispatching `BuilderToggle(D)` for a degree not
already present OR `BuilderShiftAppend(D)` must not change the list length.

**Validates: Requirements 2.4, 4.3**

---

### Property 4: Reset clears all

*For any* `builder_progression`, dispatching `BuilderReset` must produce an empty list.

**Validates: Requirement 5.2**

---

### Property 5: ExitPlayAlong returns to builder when from_builder

*For any* `PlayAlongState` with `from_builder == true`, dispatching `ExitPlayAlong` must produce
`app_mode == CustomProgressionBuilder` and `play_along_state == None`.

**Validates: Requirement 7.5, 7.6**

---

## Error Handling

### No key selected

`EnterBuilder` does nothing if `selected_key` is `None` (the button is hidden in this case).
The `CustomProgressionBuilderPanel` takes `selected_key: Key` (non-optional) — it is only rendered
when a key is present.

### Empty progression on play-along

`EnterPlayAlongCustom` guards on `builder_progression.is_empty()` and returns state unchanged.
The "Start Play Along" button is disabled in the UI when the list is empty (Requirement 7.3), so
this is defense-in-depth.

### Key changes while builder is open

If the user clicks a different key on the circle while the builder is open, `selected_key` updates
normally. The diatonic chord tiles re-render for the new key. The `builder_progression` stores
`ScaleDegree` values (key-agnostic), so existing slots remain valid — the chord names displayed
in the slots update automatically because they are derived from `diatonic_chords(selected_key)`.

---

## Testing Strategy

### Unit tests (cargo test)

All reducer logic lives in `src/state/mod.rs` and is testable without WASM.

- `BuilderToggle` on empty list → appends
- `BuilderToggle` on list containing degree → removes last occurrence
- `BuilderToggle` on list containing degree twice → removes only the last one
- `BuilderShiftAppend` always appends regardless of existing occurrences
- `BuilderReset` clears any list
- `EnterPlayAlongCustom` with empty `builder_progression` → state unchanged
- `EnterPlayAlongCustom` with no `selected_key` → state unchanged
- `ExitPlayAlong` with `from_builder == true` → `app_mode == CustomProgressionBuilder`
- `ExitPlayAlong` with `from_builder == false` → `app_mode == Normal`

### Property-based tests (proptest)

Tag format: `// Feature: custom-progression-builder, Property N: <text>`

- **Property 1**: Toggle idempotence
- **Property 2**: ShiftAppend always grows
- **Property 3**: 16-chord cap respected for both actions
- **Property 4**: Reset always produces empty list
- **Property 5**: ExitPlayAlong mode routing
