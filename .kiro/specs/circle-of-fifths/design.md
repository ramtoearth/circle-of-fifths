# Design Document: Circle of Fifths

## Overview

Circle of Fifths is a fully static, client-side web application built with Rust/WebAssembly using the Yew component framework, bundled by Trunk. There is no backend — all music theory data is computed or hardcoded in Rust, and user preferences (favorites, quiz scores, theme, mute state) are persisted to `localStorage` via `web-sys`.

The app is a single-page application (SPA) with a component tree managed by Yew. State is held in a top-level `App` component and passed down via props and callbacks, with `use_reducer` managing the global application state. Audio synthesis is handled by the Web Audio API accessed through `web-sys`.

### Tech Stack

- Language: Rust (compiled to WASM via `wasm-bindgen`)
- Framework: Yew 0.21+
- Bundler: Trunk
- Audio: Web Audio API via `web-sys`
- Persistence: `localStorage` via `web-sys`
- Diagrams: Inline SVG rendered by Yew components

---

## Architecture

The application follows a unidirectional data flow pattern (Elm-like), which maps naturally onto Yew's `use_reducer` hook.

```
┌─────────────────────────────────────────────────────┐
│                     App (root)                      │
│  AppState (use_reducer) + localStorage sync         │
│                                                     │
│  ┌──────────────┐  ┌──────────────────────────────┐ │
│  │  CircleView  │  │        MainPanel             │ │
│  │  (SVG)       │  │  ┌──────────────────────┐   │ │
│  └──────────────┘  │  │   KeyInfoPanel       │   │ │
│                    │  ├──────────────────────┤   │ │
│                    │  │   ProgressionPanel   │   │ │
│                    │  ├──────────────────────┤   │ │
│                    │  │   PianoPanel         │   │ │
│                    │  └──────────────────────┘   │ │
│                    └──────────────────────────────┘ │
│  ┌──────────────────────────────────────────────┐   │
│  │              QuizPanel (modal/page)          │   │
│  └──────────────────────────────────────────────┘   │
│  ┌──────────────────────────────────────────────┐   │
│  │              AudioEngine (no UI)             │   │
│  └──────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────┘
```

All user interactions dispatch `AppAction` variants to the reducer. The reducer produces a new `AppState`, which Yew re-renders reactively. Side effects (audio, localStorage) are handled in `use_effect` hooks triggered by state changes.

---

## Components and Interfaces

### AppState / AppAction

Central state managed by `use_reducer`. All components receive slices of this state via props.

### CircleView

Renders the circle of fifths as an inline SVG. Each of the 24 segments (12 major + 12 minor) is a `<path>` element. Emits `on_segment_click(key: Key)` callbacks.

Props:
- `selected_key: Option<Key>`
- `on_segment_click: Callback<Key>`

### KeyInfoPanel

Displays key name, key signature, scale notes, and the 7 diatonic chords. Hidden / shows placeholder when no key is selected.

Props:
- `selected_key: Option<Key>`
- `on_chord_click: Callback<DiatonicChord>`

### ProgressionPanel

Lists curated progressions for the selected key, with tag labels, Roman numeral + resolved chord names, favorite toggle, and next/prev controls when a progression is active.

Props:
- `selected_key: Option<Key>`
- `active_progression: Option<ActiveProgression>`
- `favorites: Vec<ProgressionId>`
- `on_progression_click: Callback<ProgressionId>`
- `on_next: Callback<()>`
- `on_prev: Callback<()>`
- `on_favorite_toggle: Callback<ProgressionId>`

### PianoPanel

Renders a scrollable horizontal piano keyboard (3+ octaves). Highlights scale notes and chord notes color-coded by role.

Props:
- `selected_key: Option<Key>`
- `highlighted_chord: Option<ChordHighlight>`
- `show_labels: bool`
- `octave_offset: i8`
- `on_toggle_labels: Callback<()>`
- `on_octave_shift: Callback<i8>`

### QuizPanel

Full-screen quiz mode. Manages its own local session state (current question, score) but reads/writes best scores via dispatched actions.

Props:
- `best_scores: BestScores`
- `on_session_end: Callback<SessionResult>`
- `on_exit: Callback<()>`

### AudioEngine

Not a visual component — a Rust struct wrapping a `web_sys::AudioContext`. Exposed as a Yew context so any component can trigger playback.

Methods:
- `play_scale(key: Key)`
- `play_chord(notes: &[Note])`
- `play_progression(progression: &Progression)`
- `stop()`
- `set_muted(muted: bool)`

### NavBar

Top bar with theme toggle, mute toggle, and quiz mode entry button.

---

## Data Models

All data models are pure Rust structs/enums with no runtime allocation beyond `Vec` and `String`.

```rust
/// The 12 pitch classes
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum PitchClass { C, Db, D, Eb, E, F, Gb, G, Ab, A, Bb, B }

/// A key is a pitch class + mode
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Key {
    pub root: PitchClass,
    pub mode: Mode,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Mode { Major, Minor }

/// Scale degree 1-7
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ScaleDegree { I, II, III, IV, V, VI, VII }

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ChordQuality { Major, Minor, Diminished }

#[derive(Clone, PartialEq)]
pub struct DiatonicChord {
    pub degree: ScaleDegree,
    pub quality: ChordQuality,
    pub root: PitchClass,
    pub notes: [PitchClass; 3],
}

#[derive(Clone, PartialEq)]
pub struct Progression {
    pub id: ProgressionId,
    pub key: Key,
    pub chords: Vec<ScaleDegree>,
    pub tags: Vec<ProgressionTag>,
    pub borrowed_chord: Option<BorrowedChord>,
}

#[derive(Clone, PartialEq)]
pub struct BorrowedChord {
    pub degree: ScaleDegree,
    pub source_key: Key,
}

pub type ProgressionId = u32;

#[derive(Clone, PartialEq)]
pub struct ActiveProgression {
    pub id: ProgressionId,
    pub current_index: usize,
}

#[derive(Clone, PartialEq)]
pub enum ProgressionTag {
    Pop, Jazz, Blues, Classical, Melancholic, Uplifting, Custom(String),
}

/// Chord highlight for piano panel
#[derive(Clone, PartialEq)]
pub struct ChordHighlight {
    pub root: PitchClass,
    pub third: PitchClass,
    pub fifth: PitchClass,
}

#[derive(Clone, PartialEq)]
pub enum KeyRole { Root, Third, Fifth, ScaleNote, None }

/// Quiz types
#[derive(Clone, PartialEq)]
pub enum QuestionType {
    KeySignatureAccidentals,
    RelativeMinor,
    ScaleNotes,
}

#[derive(Clone, PartialEq)]
pub struct Question {
    pub q_type: QuestionType,
    pub key: Key,
}

#[derive(Clone, Default)]
pub struct BestScores {
    pub key_sig: Option<u32>,
    pub relative_minor: Option<u32>,
    pub scale_notes: Option<u32>,
}

/// Top-level app state
#[derive(Clone)]
pub struct AppState {
    pub selected_key: Option<Key>,
    pub active_progression: Option<ActiveProgression>,
    pub favorites: Vec<ProgressionId>,
    pub highlighted_chord: Option<ChordHighlight>,
    pub show_note_labels: bool,
    pub octave_offset: i8,
    pub theme: Theme,
    pub muted: bool,
    pub quiz_active: bool,
    pub best_scores: BestScores,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Theme { Dark, Light }

/// All state transitions
pub enum AppAction {
    SelectKey(Key),
    DeselectKey,
    SelectChord(DiatonicChord),
    SelectProgression(ProgressionId),
    NextChord,
    PrevChord,
    ToggleFavorite(ProgressionId),
    ToggleNoteLabels,
    ShiftOctave(i8),
    ToggleTheme,
    ToggleMute,
    EnterQuiz,
    ExitQuiz,
    RecordQuizResult(SessionResult),
}
```

### Static Data

All progressions and music theory data are defined as `const` / `static` arrays in a `data` module, computed at compile time where possible. The `music_theory` module exposes pure functions:

```rust
pub fn diatonic_chords(key: Key) -> [DiatonicChord; 7]
pub fn key_signature(key: Key) -> KeySignature  // sharps/flats count + names
pub fn scale_notes(key: Key) -> [PitchClass; 7]
pub fn relative_minor(major: Key) -> Key
pub fn relative_major(minor: Key) -> Key
pub fn adjacent_keys(key: Key) -> (Key, Key)   // neighbors on circle
pub fn opposite_key(key: Key) -> Key
```

### localStorage Schema

Keys stored in `localStorage`:

| Key | Value |
|-----|-------|
| `cof_theme` | `"dark"` \| `"light"` |
| `cof_muted` | `"true"` \| `"false"` |
| `cof_favorites` | JSON array of `ProgressionId` |
| `cof_best_scores` | JSON object `{key_sig, relative_minor, scale_notes}` |


---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Segment selection state transition

*For any* key in the circle of fifths, dispatching `AppAction::SelectKey(key)` to the reducer should produce a state where `selected_key == Some(key)`.

**Validates: Requirements 1.2**

---

### Property 2: Segment deselection round-trip

*For any* key, if the current state has `selected_key == Some(key)`, dispatching `SelectKey(key)` again (clicking the already-selected segment) should produce a state where `selected_key == None`.

**Validates: Requirements 1.6**

---

### Property 3: Circle geometry correctness

*For any* major key K, `key_signature(K)` must return the correct accidental count (0–7 sharps or flats), `adjacent_keys(K)` must return the two keys one step clockwise and counterclockwise on the circle, and `opposite_key(K)` must return the key 6 semitones away. These relationships must hold for all 12 major keys.

**Validates: Requirements 1.4, 1.5**

---

### Property 4: Diatonic chord correctness

*For any* major key K, `diatonic_chords(K)` must return exactly 7 chords whose roots, qualities, and note content follow the major scale formula (W-W-H-W-W-W-H). The qualities must be: I=Major, ii=Minor, iii=Minor, IV=Major, V=Major, vi=Minor, vii°=Diminished. The Roman numeral labels must use uppercase for major, lowercase for minor, and lowercase+° for diminished.

**Validates: Requirements 2.2, 3.1, 3.2**

---

### Property 5: Chord display format

*For any* diatonic chord, the formatted display string must contain both the Roman numeral label (e.g. "vi") and the full chord name (e.g. "Am").

**Validates: Requirements 2.3**

---

### Property 6: Chord click updates piano highlight

*For any* diatonic chord C in any key, dispatching `AppAction::SelectChord(C)` should produce a state where `highlighted_chord` contains exactly the root, third, and fifth of C, with correct `KeyRole` assignments.

**Validates: Requirements 3.3, 5.3**

---

### Property 7: Progression data invariants

*For any* major key K, the static progression data must contain at least 4 progressions for K, those progressions must collectively cover at least 3 distinct `ProgressionTag` values, and at least one progression must have a non-None `borrowed_chord` field.

**Validates: Requirements 4.1, 4.6, 4.7**

---

### Property 8: Progression display format

*For any* progression P in any key K, the formatted display string must contain both the Roman numeral sequence (e.g. "I - V - vi - IV") and the resolved chord name sequence (e.g. "C - G - Am - F").

**Validates: Requirements 4.2**

---

### Property 9: Progression activation sets first chord

*For any* progression P, dispatching `AppAction::SelectProgression(P.id)` should produce a state where `active_progression.current_index == 0` and `highlighted_chord` matches the notes of the first chord in P.

**Validates: Requirements 4.3**

---

### Property 10: Progression navigation round-trip

*For any* active progression P of length N, dispatching `AppAction::NextChord` N times followed by `AppAction::PrevChord` N times should return `current_index` to its original value (with wrapping).

**Validates: Requirements 4.4**

---

### Property 11: Favorite toggle round-trip

*For any* progression ID, toggling favorite twice (add then remove) should leave the favorites list unchanged from its initial state.

**Validates: Requirements 4.5**

---

### Property 12: Piano scale highlight correctness

*For any* key K, the set of notes highlighted on the piano panel (with role `ScaleNote`) must exactly equal `scale_notes(K)` — no more, no fewer.

**Validates: Requirements 5.2**

---

### Property 13: Note label toggle idempotence

*For any* state, dispatching `AppAction::ToggleNoteLabels` twice should return `show_note_labels` to its original value.

**Validates: Requirements 5.5**

---

### Property 14: Octave shift round-trip

*For any* octave offset O, shifting +1 then -1 should return `octave_offset` to O.

**Validates: Requirements 5.6**

---

### Property 15: Question pool completeness and shuffle

*For any* quiz session, the shuffled question list must be a permutation of the full question pool, and the pool must contain at least one question of each of the three required types (`KeySignatureAccidentals`, `RelativeMinor`, `ScaleNotes`).

**Validates: Requirements 6.2, 6.3**

---

### Property 16: Answer evaluation correctness

*For any* question Q and any submitted answer A, the evaluation function must return `Correct` if and only if A matches the canonical correct answer for Q.

**Validates: Requirements 6.4**

---

### Property 17: Score tracking invariant

*For any* quiz session, `correct_count <= total_count` must hold at all times, and `correct_count` must increment by exactly 1 for each correct answer submitted.

**Validates: Requirements 6.5, 6.6**

---

### Property 18: localStorage round-trip

*For any* persistable state value (theme, mute state, favorites list, best scores), serializing the value to its localStorage string representation and then deserializing it must produce a value equal to the original.

**Validates: Requirements 4.5, 6.7, 7.8, 8.3**

---

### Property 19: Audio note sequence correctness

*For any* key K, the note sequence passed to the audio engine for scale playback must equal `scale_notes(K)` in ascending pitch order. For any chord C, the notes passed for chord playback must equal the root, third, and fifth of C. For any progression P, the chord sequence passed must equal the chords of P in order.

**Validates: Requirements 7.1, 7.2, 7.3**

---

### Property 20: Mute toggle round-trip

*For any* mute state, dispatching `AppAction::ToggleMute` twice should return `muted` to its original value.

**Validates: Requirements 7.7**

---

### Property 21: Theme toggle round-trip

*For any* theme, dispatching `AppAction::ToggleTheme` twice should return `theme` to its original value.

**Validates: Requirements 8.2**

---

## Error Handling

### Audio Engine Initialization Failure

If `AudioContext::new()` returns an error (e.g. browser policy blocks autoplay, or Web Audio API is unavailable), the `AudioEngine` initializes in a degraded state. The app sets an `audio_error: Option<String>` field in `AppState` and renders an error banner. All non-audio features continue to function normally.

### localStorage Unavailability

If `localStorage` is unavailable (private browsing mode, storage quota exceeded, or security policy), all persistence operations fail silently. The app uses in-memory defaults and does not crash. A console warning is emitted.

### Invalid localStorage Data

If deserialization of a stored value fails (e.g. corrupted JSON), the app discards the stored value and uses the default. This prevents a bad stored value from breaking the app on reload.

### Out-of-Range Octave Shift

The `octave_offset` is clamped to a valid range (e.g. -2 to +2) so the piano panel always shows a meaningful range of notes.

### Empty Progression Navigation

If `NextChord` or `PrevChord` is dispatched with no active progression, the action is a no-op.

---

## Testing Strategy

### Dual Testing Approach

Both unit tests and property-based tests are required. They are complementary:

- Unit tests cover specific examples, integration points, and error conditions.
- Property-based tests verify universal correctness across all inputs.

### Property-Based Testing

The property-based testing library for Rust is **`proptest`** (crate: `proptest`). Each correctness property from the Correctness Properties section must be implemented as a single `proptest!` test, configured to run a minimum of 100 iterations.

Each property test must be tagged with a comment in the following format:

```
// Feature: circle-of-fifths, Property N: <property_text>
```

Example:

```rust
// Feature: circle-of-fifths, Property 4: Diatonic chord correctness
proptest! {
    #[test]
    fn test_diatonic_chord_correctness(key in any_major_key()) {
        let chords = diatonic_chords(key);
        prop_assert_eq!(chords.len(), 7);
        prop_assert_eq!(chords[0].quality, ChordQuality::Major);   // I
        prop_assert_eq!(chords[1].quality, ChordQuality::Minor);   // ii
        // ... etc
    }
}
```

Custom `proptest` strategies will be written for:
- `any_major_key()` — generates a random `Key` with `Mode::Major`
- `any_key()` — generates any `Key` (major or minor)
- `any_diatonic_chord(key)` — generates a random diatonic chord for a given key
- `any_progression(key)` — picks a random progression from the static data for a key
- `any_app_state()` — generates a random valid `AppState`

### Unit Tests

Unit tests (standard `#[test]`) cover:

- Specific known key signatures (e.g. C major = 0 accidentals, G major = 1 sharp, F major = 1 flat)
- Known diatonic chord names (e.g. C major → C, Dm, Em, F, G, Am, Bdim)
- Progression display formatting for a specific known progression
- Quiz answer evaluation for specific correct and incorrect answers
- Audio engine degraded-mode behavior when initialization fails
- localStorage deserialization failure fallback to defaults
- Piano panel rendering produces the correct number of keys (at least 36 for 3 octaves)
- Quiz mode entry/exit state transitions

### Test Organization

```
src/
  music_theory/
    mod.rs
    tests.rs          # unit + property tests for pure music theory functions
  state/
    mod.rs
    tests.rs          # unit + property tests for AppState reducer
  data/
    mod.rs
    tests.rs          # unit + property tests for static progression data invariants
  audio/
    mod.rs
    tests.rs          # unit tests for audio engine (mocked AudioContext)
  storage/
    mod.rs
    tests.rs          # unit + property tests for localStorage serialization round-trips
  components/
    ...               # Yew components (UI tests via wasm-bindgen-test if needed)
```

Property tests that require WASM APIs (e.g. localStorage) are run with `wasm-pack test --headless --firefox`.
Pure Rust property tests (music theory, state reducer) run with `cargo test`.
