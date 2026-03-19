# Design Document: MIDI Keyboard Integration

## Overview

MIDI Keyboard Integration extends the Circle of Fifths app with real-time hardware MIDI keyboard support. The feature bridges the browser's Web MIDI API (accessed via raw `js-sys`/`wasm-bindgen` interop, since `web-sys` does not yet expose it) into the existing Yew/WASM architecture. MIDI events flow from JS callbacks into Yew's unidirectional state system by dispatching new `AppAction` variants into the existing reducer. No backend is required; all processing runs in the browser.

The feature adds:
- MIDI device enumeration and hot-plug detection
- Real-time note highlighting on the existing `PianoPanel` with velocity-proportional intensity
- Chord recognition from held notes
- Key/scale detection from a rolling window of recent notes
- Practice Mode: play target chords, get per-note feedback
- Play-Along Mode: follow a progression at a configurable BPM

---

## Architecture

The existing app uses a unidirectional data flow: `AppState` + `AppAction` reducer via `use_reducer`, with side effects in `use_effect` hooks. This feature extends that pattern without breaking it.

```
┌──────────────────────────────────────────────────────────────────┐
│                          Browser                                 │
│  navigator.requestMIDIAccess()  ──►  MIDIAccess JS object        │
│         │                                                        │
│  onmidimessage callback (JS)                                     │
│         │                                                        │
│  wasm_bindgen Closure<dyn Fn(JsValue)>                           │
│         │                                                        │
│  MidiEngine (Rust struct, held in use_memo)                      │
│    - parses raw bytes → MidiEvent                                │
│    - dispatches AppAction via Yew Callback                       │
│         │                                                        │
│  AppState reducer  ◄──────────────────────────────────────────── │
│    - HeldNotes, RollingWindow, MidiStatus, Mode                  │
│         │                                                        │
│  Yew component tree re-renders                                   │
│    - PianoPanel: velocity-tinted highlights                      │
│    - MidiStatusBar: device name, chord name, key suggestions     │
│    - PracticePanel / PlayAlongPanel                              │
└──────────────────────────────────────────────────────────────────┘
```

### JS Interop Layer

The Web MIDI API is not in `web-sys`. All access goes through `js_sys` raw interop:

```rust
// Pseudocode — actual calls use js_sys::Reflect, Promise, wasm_bindgen::closure::Closure
let navigator: JsValue = web_sys::window().unwrap().navigator().into();
let promise: js_sys::Promise = js_sys::Reflect::get(&navigator, &"requestMIDIAccess".into())
    .unwrap()
    .dyn_into::<js_sys::Function>()
    .unwrap()
    .call0(&navigator)
    .unwrap()
    .dyn_into::<js_sys::Promise>()
    .unwrap();
```

The resolved `MIDIAccess` object is stored as a `JsValue` in a `use_ref`. Input port enumeration and `onmidimessage` handler registration are done via `js_sys::Reflect`. Each port's `onmidimessage` is set to a `wasm_bindgen::closure::Closure<dyn Fn(JsValue)>` that parses the raw MIDI bytes and dispatches into Yew.

### MIDI Event Bridge

```
JS onmidimessage  →  Closure<dyn Fn(JsValue)>  →  parse_midi_message()  →  AppAction dispatch
```

`parse_midi_message` extracts the `data` `Uint8Array` from the `MIDIMessageEvent` JsValue, reads the status byte and data bytes, and returns a `MidiEvent` enum. The closure holds a `Callback<AppAction>` cloned from the Yew dispatch handle.

Hot-plug is handled by setting `onstatechange` on the `MIDIAccess` object to a similar closure that dispatches `AppAction::MidiStateChange`.

---

## Components and Interfaces

### AudioEngine (extended)

`AudioEngine` gains a `schedule_metronome_click` method used by the metronome beat scheduler in `App`:

```rust
impl AudioEngine {
    /// Schedule a single short high-pitched click at `start` seconds (AudioContext time).
    /// Uses a triangle oscillator at 1200 Hz with a 30 ms duration and fast decay,
    /// distinct from the sine-wave notes used for chord/scale playback.
    pub fn schedule_metronome_click(&self, start: f64) { /* ... */ }
}
```

The metronome beat loop runs in `App` via a `use_interval` (gloo_timers) set to `60_000 / bpm` ms. On each tick it calls `audio_engine.schedule_metronome_click(ctx.current_time() + small_lookahead)`. The interval is recreated whenever `bpm` or `metronome_active` changes. When `metronome_active` is false or the engine is muted, no clicks are scheduled.

---

### MidiEngine

A non-visual Rust struct held in `use_memo` (similar to `AudioEngineHandle`). Owns the JS interop lifecycle.

```rust
pub struct MidiEngine {
    midi_access: Option<JsValue>,          // the MIDIAccess JS object
    _closures: Vec<wasm_bindgen::closure::Closure<dyn Fn(JsValue)>>, // kept alive
}

impl MidiEngine {
    pub fn request_access(dispatch: Callback<AppAction>) -> Self;
    pub fn register_ports(&self, dispatch: Callback<AppAction>);
    pub fn connected_device_names(&self) -> Vec<String>;
}
```

`MidiEngine` is initialized in `App` via `use_effect_with((), ...)` after mount, mirroring how `AudioEngineHandle` is set up. The `Callback<AppAction>` passed in is the Yew dispatch handle.

### MidiStatusBar (new component)

Displays MIDI connection status, connected device name(s), recognized chord, Roman numeral, diatonic/borrowed indicator, and key detection suggestions.

Props:
```rust
pub struct MidiStatusBarProps {
    pub midi_status: MidiStatus,
    pub device_names: Vec<String>,
    pub recognized_chord: Option<RecognizedChord>,
    pub key_suggestions: Vec<KeySuggestion>,
    pub on_clear_window: Callback<()>,
}
```

### PianoPanel (extended)

Existing component gains new props for MIDI-driven highlights:

```rust
// New props added to existing PianoPanelProps:
pub held_notes: Vec<HeldNote>,          // MIDI notes currently depressed
pub practice_target: Option<Vec<PitchClass>>,  // notes to match in practice/play-along
```

Each key renders with a `midi-held` CSS class when present in `held_notes`, with an inline `opacity` or `filter: brightness(...)` derived from velocity. Practice/play-along keys get `midi-correct` (green) or `midi-incorrect` (red) classes.

### PracticePanel (new component)

Full-screen mode (similar to `QuizPanel`) for Practice Mode.

Props:
```rust
pub struct PracticePanelProps {
    pub target_chord: DiatonicChord,
    pub held_notes: Vec<HeldNote>,
    pub score: PracticeScore,
    pub on_exit: Callback<()>,
}
```

### PlayAlongPanel (new component)

Overlays the `ProgressionPanel` area when Play-Along Mode is active.

Props:
```rust
pub struct PlayAlongPanelProps {
    pub progression: Progression,
    pub current_chord_index: usize,
    pub bpm: u16,
    pub held_notes: Vec<HeldNote>,
    pub score: PlayAlongScore,
    pub on_stop: Callback<()>,
    pub on_bpm_change: Callback<u16>,
}
```

### NavBar (extended)

Gains a "Practice" button, visible only when `midi_status == MidiStatus::Connected`, and a "Metronome" toggle button adjacent to the BPM slider. The BPM slider range is updated to 40–200 to align with the play-along spec.

New props added to `NavBarProps`:
```rust
pub midi_status: MidiStatus,
pub metronome_active: bool,
pub on_enter_practice: Callback<()>,
pub on_toggle_metronome: Callback<()>,
```

### ProgressionPanel (extended)

Gains a "Play Along" button per progression, visible only when `midi_status == MidiStatus::Connected` and a progression is active.

---

## Data Models

### New types in `src/midi/mod.rs`

```rust
/// Raw parsed MIDI event
#[derive(Clone, Debug, PartialEq)]
pub enum MidiEvent {
    NoteOn  { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    Other,
}

/// A note currently held down
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HeldNote {
    pub midi_note: u8,       // 0–127
    pub velocity: u8,        // 1–127
    pub pitch_class: PitchClass,
    pub octave: i8,
}

impl HeldNote {
    pub fn from_midi(note: u8, velocity: u8) -> Self {
        HeldNote {
            midi_note: note,
            velocity,
            pitch_class: PitchClass::from_index(note % 12),
            octave: (note / 12) as i8 - 1,
        }
    }
    /// Maps velocity 1–127 to opacity 0.35–1.0 linearly
    pub fn velocity_opacity(self) -> f32 {
        0.35 + (self.velocity as f32 - 1.0) / 126.0 * 0.65
    }
}

/// MIDI subsystem connection state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiStatus {
    Unavailable,      // browser doesn't support Web MIDI
    PermissionDenied, // user denied access
    NoDevices,        // access granted but no inputs
    Connected,        // at least one input active
}

/// Result of chord recognition
#[derive(Clone, Debug, PartialEq)]
pub struct RecognizedChord {
    pub name: String,                    // e.g. "Am", "Cmaj7"
    pub pitch_classes: Vec<PitchClass>,
    pub roman_numeral: Option<String>,   // e.g. "vi", present when a key is selected
    pub is_diatonic: Option<bool>,       // None when no key selected
}

/// A key candidate from key detection
#[derive(Clone, Debug, PartialEq)]
pub struct KeySuggestion {
    pub key: Key,
    pub score: u8,   // count of rolling-window PitchClasses in this key's scale (0–7)
}

/// Practice mode per-chord score
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PracticeScore {
    pub correct_notes: u32,
    pub total_notes_played: u32,
}

/// Play-along per-chord result
#[derive(Clone, Debug, PartialEq)]
pub struct ChordResult {
    pub chord_index: usize,
    pub correct: bool,   // all target PitchClasses were present in held notes
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PlayAlongScore {
    pub chord_results: Vec<ChordResult>,
}
```

### AppState extensions

```rust
// Added fields to AppState:
pub midi_status: MidiStatus,
pub device_names: Vec<String>,
pub held_notes: Vec<HeldNote>,
pub rolling_window: Vec<(PitchClass, f64)>,  // (pitch_class, timestamp_ms)
pub recognized_chord: Option<RecognizedChord>,
pub key_suggestions: Vec<KeySuggestion>,
pub app_mode: AppMode,
pub practice_state: Option<PracticeState>,
pub play_along_state: Option<PlayAlongState>,
pub metronome_active: bool,                  // persisted in localStorage
```

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Practice,
    PlayAlong,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PracticeState {
    pub target_chord: DiatonicChord,
    pub score: PracticeScore,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayAlongState {
    pub progression_id: ProgressionId,
    pub current_chord_index: usize,
    pub bpm: u16,
    pub score: PlayAlongScore,
    pub started_at_ms: f64,
}
```

### New AppAction variants

```rust
// Added to AppAction enum:
MidiStatusChanged(MidiStatus),
MidiDevicesChanged(Vec<String>),
MidiNoteOn(HeldNote),
MidiNoteOff(u8),              // midi_note number
ClearRollingWindow,
EnterPractice,
ExitPractice,
PracticeAdvance,              // chord correctly played, advance to next
EnterPlayAlong(ProgressionId, u16 /*bpm*/),
ExitPlayAlong,
PlayAlongTick,                // beat timer fires, advance chord
PlayAlongSetBpm(u16),
RecordPlayAlongChordResult(ChordResult),
ToggleMetronome,              // flip metronome_active; persisted to localStorage
```

### Chord Recognition Algorithm

```
Input: held_notes: Vec<HeldNote>
1. Collect distinct PitchClasses from held_notes
2. If < 3 distinct PitchClasses → return None
3. For each entry in CHORD_DICTIONARY:
   a. For each inversion (rotate root to front):
      - Check if the inversion's PitchClasses are a subset of held PitchClasses
      - Score = number of matching PitchClasses
4. Return the highest-scoring match (ties broken by fewest extra notes)
5. If no match → return note names only
```

The `CHORD_DICTIONARY` is a `static` array of `(name: &str, intervals: &[u8])` covering triads (major, minor, diminished, augmented) and seventh chords (maj7, min7, dom7, half-dim7, dim7).

### Key Detection Algorithm

```
Input: rolling_window: Vec<(PitchClass, f64)>, now_ms: f64
1. Filter to entries where now_ms - timestamp_ms <= 10_000.0
2. Collect distinct PitchClasses from filtered entries
3. If < 4 distinct PitchClasses → return empty Vec
4. For each of the 24 keys (12 major + 12 minor):
   score = count of distinct PitchClasses that belong to scale_notes(key)
5. Sort by score descending, return top 3
```

### localStorage Schema additions

| Key | Value |
|-----|-------|
| `midi_practice_scores` | JSON object with per-chord accuracy history |
| `metronome_active` | `"true"` or `"false"` — persisted metronome toggle state |

No MIDI device preferences are persisted (device selection is automatic).

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: NoteOn/NoteOff round-trip

*For any* valid MIDI note number (0–127) and velocity (1–127), dispatching `MidiNoteOn` followed by `MidiNoteOff` for the same note should leave `held_notes` unchanged from its state before the `NoteOn`.

**Validates: Requirements 2.1, 2.2**

---

### Property 2: MIDI note to PitchClass/Octave derivation

*For any* MIDI note number N in 0–127, `HeldNote::from_midi(N, v).pitch_class` must equal `PitchClass::from_index(N % 12)` and `.octave` must equal `(N / 12) as i8 - 1`.

**Validates: Requirements 2.3**

---

### Property 3: Velocity opacity is monotonically increasing

*For any* two velocities V1 < V2 in 1–127, `velocity_opacity(V1) < velocity_opacity(V2)`. Additionally, `velocity_opacity(1) == 0.35` and `velocity_opacity(127) == 1.0`.

**Validates: Requirements 2.4**

---

### Property 4: Velocity = 0 treated as NoteOff

*For any* MIDI note number N that is present in `held_notes`, dispatching `MidiNoteOn { note: N, velocity: 0 }` should remove N from `held_notes`, identical to dispatching `MidiNoteOff { note: N }`.

**Validates: Requirements 2.5** (edge case)

---

### Property 5: Chord recognition requires 3+ distinct PitchClasses

*For any* set of held notes with fewer than 3 distinct `PitchClass` values, `recognize_chord` must return `None`.

**Validates: Requirements 3.2**

---

### Property 6: Known chords are recognized in all inversions

*For any* chord in the chord dictionary (triads and seventh chords), presenting its PitchClasses in any inversion (rotation of the note order) to `recognize_chord` must return the correct chord name.

**Validates: Requirements 3.1**

---

### Property 7: Chord-in-key annotation correctness

*For any* recognized chord and any selected `Key`, the `roman_numeral` field must be `Some(rn)` where `rn` matches the chord's scale degree in that key, and `is_diatonic` must be `true` if and only if the chord's root and quality appear in `diatonic_chords(key)`.

**Validates: Requirements 3.3, 3.4**

---

### Property 8: Rolling window excludes stale notes

*For any* sequence of `(PitchClass, timestamp_ms)` entries, `filter_rolling_window(entries, now_ms)` must return only entries where `now_ms - timestamp_ms <= 10_000.0`.

**Validates: Requirements 4.1**

---

### Property 9: Key detection threshold

*For any* set of fewer than 4 distinct `PitchClass` values in the rolling window, `detect_keys` must return an empty `Vec`.

**Validates: Requirements 4.3**

---

### Property 10: Key detection ranking

*For any* set of 4 or more distinct `PitchClass` values in the rolling window, `detect_keys` must return a `Vec` of at most 3 `KeySuggestion` values sorted by `score` descending, where each `score` equals the count of input `PitchClass` values that belong to `scale_notes(suggestion.key)`.

**Validates: Requirements 4.2**

---

### Property 11: Clear rolling window resets state

*For any* `AppState` with a non-empty `rolling_window`, dispatching `ClearRollingWindow` must produce a state where `rolling_window` is empty and `key_suggestions` is empty.

**Validates: Requirements 4.6**

---

### Property 12: Device disconnection clears held notes

*For any* `AppState` with a non-empty `held_notes`, dispatching `MidiDevicesChanged` with an empty device list must produce a state where `held_notes` is empty.

**Validates: Requirements 1.7**

---

### Property 13: Practice/play-along note color classification

*For any* set of `held_notes` and any `target_chord` (a `Vec<PitchClass>`), the classification function must assign: `Correct` to notes whose `PitchClass` is in `target_chord`, `Incorrect` to notes whose `PitchClass` is not in `target_chord`, and `Unplayed` to `target_chord` PitchClasses not present in `held_notes`. These three sets must be disjoint and their union must cover all held notes plus all target notes.

**Validates: Requirements 5.3, 6.4**

---

### Property 14: Accuracy score invariant

*For any* `PracticeScore` or `PlayAlongScore`, `correct_notes <= total_notes_played` must hold at all times, and `correct_notes / total_notes_played` (as f32, with total > 0) must be in [0.0, 1.0].

**Validates: Requirements 5.5, 6.5**

---

### Property 15: BPM clamping

*For any* BPM value passed to `PlayAlongSetBpm`, the resulting `play_along_state.bpm` must be clamped to the range [40, 200].

**Validates: Requirements 6.2**

---

### Property 16: ExitPlayAlong resets mode

*For any* `AppState` where `app_mode == AppMode::PlayAlong`, dispatching `ExitPlayAlong` must produce a state where `app_mode == AppMode::Normal` and `play_along_state == None`.

**Validates: Requirements 6.7**

---

### Property 17: Metronome toggle round-trip

*For any* `AppState`, dispatching `ToggleMetronome` twice must produce a state where `metronome_active` equals its original value (idempotent double-toggle).

**Validates: Requirement 7.1, 7.2, 7.3**

---

### Property 18: BPM range invariant

*For any* BPM value passed to `SetBpm`, the resulting `AppState.bpm` must be clamped to the range [40, 200]. This applies to both the NavBar slider and the play-along BPM control, which share the same `AppState.bpm` field.

**Validates: Requirements 6.2, 7.4, 7.8**

---

## Error Handling

### Web MIDI API Unavailable

If `navigator.requestMIDIAccess` does not exist on the navigator object (non-Chromium browser), the `MidiEngine` sets `MidiStatus::Unavailable` and dispatches `MidiStatusChanged(Unavailable)`. The app renders a notice banner. All non-MIDI features continue normally.

### Permission Denied

If the Promise returned by `requestMIDIAccess` rejects, the rejection handler dispatches `MidiStatusChanged(PermissionDenied)`. The app renders a notice with instructions for re-enabling permission in browser settings.

### No Devices

If `MIDIAccess.inputs` is empty after access is granted, the app dispatches `MidiStatusChanged(NoDevices)` and renders a "no MIDI devices found" message.

### Device Disconnection Mid-Session

When `onstatechange` fires with a port state of `"disconnected"`, the app dispatches `MidiDevicesChanged` with the updated device list and `MidiNoteOff` for any held notes from that device. This prevents phantom held notes.

### JS Interop Panics

All `js_sys::Reflect::get` calls and `.dyn_into()` casts are wrapped in `Result`/`Option` chains. Failures are logged to the browser console and result in `MidiStatus::Unavailable` rather than a WASM panic.

### Play-Along Timer Drift

The beat timer uses `gloo_timers` (or `web_sys::Window::set_interval`) with a fixed interval derived from BPM. Timer drift is not corrected; this is acceptable for a learning tool. If the component unmounts, the interval handle is dropped and the timer stops.

---

## Testing Strategy

### Dual Testing Approach

Both unit tests and property-based tests are required and complementary:

- Unit tests: specific examples, integration points, edge cases, error conditions
- Property tests: universal correctness across all inputs via randomized generation

### Property-Based Testing

The property-based testing library is **`proptest`** (same as the existing `circle-of-fifths` spec). Each correctness property from the Correctness Properties section must be implemented as a single `proptest!` test with a minimum of 100 iterations.

Tag format for each test:
```
// Feature: midi-keyboard-integration, Property N: <property_text>
```

Custom `proptest` strategies:
- `any_midi_note()` — generates a `u8` in 0..=127
- `any_velocity()` — generates a `u8` in 1..=127
- `any_pitch_class_set(min, max)` — generates a `HashSet<PitchClass>` of size min..=max
- `any_held_notes(n)` — generates a `Vec<HeldNote>` of length n
- `any_key()` — generates any `Key` (major or minor, any root)
- `any_rolling_window()` — generates a `Vec<(PitchClass, f64)>` with random timestamps
- `any_bpm()` — generates a `u16` in any range (including out-of-bounds for clamping tests)

Example:
```rust
// Feature: midi-keyboard-integration, Property 2: MIDI note to PitchClass/Octave derivation
proptest! {
    #[test]
    fn test_midi_note_derivation(note in 0u8..=127u8, vel in 1u8..=127u8) {
        let held = HeldNote::from_midi(note, vel);
        prop_assert_eq!(held.pitch_class, PitchClass::from_index(note % 12));
        prop_assert_eq!(held.octave, (note / 12) as i8 - 1);
    }
}
```

### Unit Tests

Unit tests cover:
- `requestMIDIAccess` called without sysex (mock JS environment)
- `MidiStatus::Unavailable` when API absent — non-MIDI state fields unaffected
- `MidiStatus::PermissionDenied` — non-MIDI state fields unaffected
- Specific chord recognition examples: C major triad → "C", A minor triad → "Am", G dominant 7 → "G7"
- Chord recognition with extra notes (4-note input matching a triad)
- Unrecognized note set returns note names without chord label
- Key detection: C major scale notes → top suggestion is C major
- Rolling window: notes at t=0 excluded when now=11000ms
- Practice mode entry blocked when `midi_status != Connected`
- Play-along mode entry blocked when `midi_status != Connected`
- `ExitPlayAlong` from Normal mode is a no-op

### Test Organization

```
src/
  midi/
    mod.rs          # MidiEngine, HeldNote, MidiEvent, chord/key algorithms
    tests.rs        # unit + property tests for all midi module functions
  state/
    mod.rs          # AppState reducer (extended with MIDI actions)
    tests.rs        # existing tests + new MIDI action tests
```

Pure Rust tests (no WASM APIs needed) run with `cargo test`. Tests requiring browser APIs run with `wasm-pack test --headless --chrome`.
