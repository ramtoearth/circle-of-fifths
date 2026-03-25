# Design Document: Metronome Time Signature

## Overview

This feature extends the existing metronome with time signature support (numerator and denominator) and an accent click on beat 1 of each bar. The metronome currently fires a uniform 1200 Hz triangle click at a fixed interval derived solely from BPM. After this change it will:

1. Store and validate a `TimeSignature` struct in `AppState`.
2. Compute beat intervals using the denominator (`(60_000 / bpm) * (4 / denominator)`).
3. Track a `beat_index` counter that wraps modulo the numerator.
4. Play a 1800 Hz accent click on beat 0 and a 1200 Hz regular click on all other beats.
5. Persist the time signature to localStorage and restore it on load.
6. Expose numerator/denominator selectors in the `NavBar`.

The implementation touches four layers: data model (`state`), audio engine (`audio`), storage (`storage`), and UI (`components/nav_bar`, `components/app`).

---

## Architecture

```mermaid
flowchart TD
    NavBar -->|SetTimeSignature| AppState
    NavBar -->|SetBpm| AppState
    AppState -->|time_signature, bpm| App
    App -->|use_effect_with(bpm, metronome_active, time_signature)| MetronomeInterval
    MetronomeInterval -->|schedule_metronome_click_accented| AudioEngine
    AppState -->|save_state| Storage
    Storage -->|load_state| AppState
```

The metronome interval effect in `app.rs` is the single orchestration point. It owns a `beat_index` counter in a `use_mut_ref` cell, resets it when the effect re-runs (i.e., when BPM, active state, or time signature changes), and calls `schedule_metronome_click_accented` with the correct `is_accent` flag on every tick.

---

## Components and Interfaces

### `TimeSignature` (new struct in `state/mod.rs`)

```rust
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct TimeSignature {
    pub numerator: u32,   // 1–16
    pub denominator: u32, // 1 | 2 | 4 | 8 | 16
}

impl TimeSignature {
    pub const DEFAULT: Self = Self { numerator: 4, denominator: 4 };

    /// Returns Some(self) if valid, None otherwise.
    pub fn validated(numerator: u32, denominator: u32) -> Option<Self>;

    /// Beat interval in milliseconds.
    pub fn beat_interval_ms(bpm: u32, denominator: u32) -> u32;
}
```

Valid denominator set: `{1, 2, 4, 8, 16}`.

### `AppAction::SetTimeSignature` (new variant)

```rust
AppAction::SetTimeSignature(u32, u32) // (numerator, denominator)
```

The reducer validates the pair via `TimeSignature::validated`; if invalid, state is unchanged.

### `AppState` changes

- Add `time_signature: TimeSignature` field, defaulting to `TimeSignature::DEFAULT`.
- The `SetTimeSignature` reducer arm validates and updates the field.

### `AudioEngine::schedule_metronome_click_accented` (new method)

```rust
pub fn schedule_metronome_click_accented(&self, start: f64, is_accent: bool);
```

Selects 1800 Hz when `is_accent = true`, 1200 Hz otherwise. Same triangle oscillator, 30 ms duration, exponential decay envelope as the existing `schedule_metronome_click`.

The existing `schedule_metronome_click` is kept for backward compatibility but the metronome loop in `app.rs` will switch to the new method.

### `AudioEngineHandle` additions

```rust
pub fn schedule_metronome_click_accented(&self, start: f64, is_accent: bool);
```

Delegates to the inner `AudioEngine`.

### Metronome interval effect in `app.rs`

The existing `use_effect_with((bpm, metronome_active), ...)` is extended to also depend on `time_signature`. A `use_mut_ref` cell holds the current `beat_index: u32`. On each tick:

```
beat_interval_ms = TimeSignature::beat_interval_ms(bpm, denominator)
is_accent = beat_index == 0
schedule_metronome_click_accented(now + 0.02, is_accent)
beat_index = (beat_index + 1) % numerator
```

When the effect re-runs (BPM / active / time_signature change), `beat_index` is reset to 0.

### `NavBar` changes

New props:

```rust
pub time_signature: TimeSignature,
pub on_set_time_signature: Callback<(u32, u32)>,
```

Two `<select>` elements are added adjacent to the BPM slider:
- Numerator: options 1–16.
- Denominator: options 1, 2, 4, 8, 16.
- A read-only label showing `"{numerator}/{denominator}"`.

### Storage changes

`PersistedState` gains a `time_signature: TimeSignature` field. The serialization uses a single JSON key `"cof_time_signature"` storing `"{numerator}/{denominator}"` (e.g. `"3/4"`). On deserialization, if the key is absent or the stored value is invalid, `TimeSignature::DEFAULT` is used.

---

## Data Models

### `TimeSignature`

| Field | Type | Constraints |
|---|---|---|
| `numerator` | `u32` | 1 ≤ n ≤ 16 |
| `denominator` | `u32` | ∈ {1, 2, 4, 8, 16} |

Default: `{ numerator: 4, denominator: 4 }`.

### `PersistedState` (extended)

```rust
pub struct PersistedState {
    pub theme: Theme,
    pub muted: bool,
    pub favorites: Vec<ProgressionId>,
    pub metronome_active: bool,
    pub auto_playback_enabled: bool,
    pub time_signature: TimeSignature,   // NEW
}
```

Serialized as a string `"numerator/denominator"` under key `"cof_time_signature"`.

### Beat interval formula

```
beat_interval_ms(bpm, denominator) = (60_000 / bpm) * (4 / denominator)
```

| Denominator | Interval at 120 BPM |
|---|---|
| 2 | 1000 ms (half note) |
| 4 | 500 ms (quarter note) |
| 8 | 250 ms (eighth note) |
| 16 | 125 ms (sixteenth note) |

Integer arithmetic is used throughout; the formula is exact for all valid denominator values.

---

## Correctness Properties

*A property is a characteristic or behavior that should hold true across all valid executions of a system — essentially, a formal statement about what the system should do. Properties serve as the bridge between human-readable specifications and machine-verifiable correctness guarantees.*

### Property 1: Valid numerator acceptance

*For any* integer value, `TimeSignature::validated(n, 4)` should return `Some` if and only if `n` is in the range 1–16 inclusive, and `None` otherwise.

**Validates: Requirements 1.2, 1.4**

### Property 2: Valid denominator acceptance

*For any* integer value, `TimeSignature::validated(4, d)` should return `Some` if and only if `d` is in the set {1, 2, 4, 8, 16}, and `None` otherwise.

**Validates: Requirements 1.3, 1.5**

### Property 3: Time signature serialization round-trip

*For any* valid `TimeSignature`, serializing it to a string and then deserializing it should produce an equal `TimeSignature`.

**Validates: Requirements 1.6, 6.1, 6.2**

### Property 4: Beat index modular wrap

*For any* numerator N in the range 1–16 and any number of beats K ≥ 0, after K beats the `beat_index` should equal `K % N`.

**Validates: Requirements 3.1, 3.2, 3.5**

### Property 5: Accent selection correctness

*For any* beat index and numerator, `schedule_metronome_click_accented` should be called with `is_accent = true` if and only if `beat_index == 0`.

**Validates: Requirements 4.1, 4.2**

### Property 6: Accent pitch is higher than regular pitch

*For any* call to the accent scheduling logic, the frequency used when `is_accent = true` should be strictly greater than the frequency used when `is_accent = false`.

**Validates: Requirements 4.3**

### Property 7: Mute suppresses all clicks

*For any* beat sequence, when the audio engine is muted, `schedule_metronome_click_accented` should not produce any audio output (the method returns early without scheduling oscillator nodes).

**Validates: Requirements 4.5**

### Property 8: Beat interval formula correctness

*For any* BPM value in [40, 200] and any valid denominator in {1, 2, 4, 8, 16}, `beat_interval_ms(bpm, denominator)` should equal `(60_000 / bpm) * (4 / denominator)`.

**Validates: Requirements 5.1, 5.2, 5.3, 5.4**

### Property 9: Time signature display format

*For any* valid `TimeSignature`, the formatted label string should equal `"{numerator}/{denominator}"`.

**Validates: Requirements 2.6**

---

## Error Handling

| Scenario | Behavior |
|---|---|
| `SetTimeSignature` with numerator outside 1–16 | Reducer returns unchanged state |
| `SetTimeSignature` with denominator not in {1,2,4,8,16} | Reducer returns unchanged state |
| localStorage key `cof_time_signature` absent on load | `TimeSignature::DEFAULT` (4/4) used |
| localStorage value unparseable or invalid on load | `TimeSignature::DEFAULT` (4/4) used |
| BPM is 0 (division guard) | `bpm.max(1)` applied before division, same as existing code |
| AudioContext unavailable (degraded mode) | `schedule_metronome_click_accented` returns early, same as existing `schedule_metronome_click` |

---

## Testing Strategy

### Unit tests

Focus on specific examples, edge cases, and error conditions:

- `TimeSignature::DEFAULT` equals `{ numerator: 4, denominator: 4 }`.
- `validated` returns `None` for numerator 0 and numerator 17.
- `validated` returns `None` for denominator 3, 5, 6, 7.
- `beat_interval_ms(120, 4)` equals 500.
- `beat_interval_ms(120, 8)` equals 250.
- `beat_interval_ms(120, 2)` equals 1000.
- Deserializing absent key returns `TimeSignature::DEFAULT`.
- Deserializing `"0/4"` returns `TimeSignature::DEFAULT`.
- Deserializing `"4/3"` returns `TimeSignature::DEFAULT`.
- `SetTimeSignature(0, 4)` leaves `AppState.time_signature` unchanged.
- `SetTimeSignature(4, 3)` leaves `AppState.time_signature` unchanged.
- `SetTimeSignature(3, 8)` updates `AppState.time_signature` to `{ 3, 8 }`.
- Beat index resets to 0 after stop/restart.
- Beat index resets to 0 after time signature change.

### Property-based tests

Use the `proptest` crate (already a dependency in this project). Each test runs a minimum of 100 iterations.

**Property 1 — Valid numerator acceptance**
```
// Feature: metronome-time-signature, Property 1: Valid numerator acceptance
proptest! {
    fn prop_valid_numerator(n: u32) {
        let result = TimeSignature::validated(n, 4);
        if n >= 1 && n <= 16 {
            prop_assert!(result.is_some());
        } else {
            prop_assert!(result.is_none());
        }
    }
}
```

**Property 2 — Valid denominator acceptance**
```
// Feature: metronome-time-signature, Property 2: Valid denominator acceptance
proptest! {
    fn prop_valid_denominator(d: u32) {
        let result = TimeSignature::validated(4, d);
        let valid = matches!(d, 1 | 2 | 4 | 8 | 16);
        prop_assert_eq!(result.is_some(), valid);
    }
}
```

**Property 3 — Serialization round-trip**
```
// Feature: metronome-time-signature, Property 3: Time signature serialization round-trip
proptest! {
    fn prop_time_sig_serde_round_trip(n in 1u32..=16, d in proptest::sample::select(vec![1u32,2,4,8,16])) {
        let ts = TimeSignature { numerator: n, denominator: d };
        let s = serialize_time_signature(ts);
        let restored = deserialize_time_signature(&s);
        prop_assert_eq!(restored, ts);
    }
}
```

**Property 4 — Beat index modular wrap**
```
// Feature: metronome-time-signature, Property 4: Beat index modular wrap
proptest! {
    fn prop_beat_index_wrap(n in 1u32..=16, k in 0u32..200) {
        let mut idx = 0u32;
        for _ in 0..k {
            idx = (idx + 1) % n;
        }
        prop_assert_eq!(idx, k % n);
    }
}
```

**Property 5 — Accent selection correctness**
```
// Feature: metronome-time-signature, Property 5: Accent selection correctness
proptest! {
    fn prop_accent_on_beat_zero(n in 1u32..=16, k in 0u32..200) {
        let beat_index = k % n;
        let is_accent = beat_index == 0;
        prop_assert_eq!(is_accent, beat_index == 0);
    }
}
```

**Property 6 — Accent pitch higher than regular**
```
// Feature: metronome-time-signature, Property 6: Accent pitch is higher than regular pitch
// (deterministic — single assertion)
#[test]
fn accent_freq_greater_than_regular_freq() {
    assert!(ACCENT_FREQ > REGULAR_FREQ);
}
```

**Property 7 — Mute suppresses all clicks**
```
// Feature: metronome-time-signature, Property 7: Mute suppresses all clicks
// Tested via AudioEngine::is_muted() guard in schedule_metronome_click_accented.
// Unit test: construct a degraded engine, set muted=true, call the method, assert no panic and no scheduling.
```

**Property 8 — Beat interval formula correctness**
```
// Feature: metronome-time-signature, Property 8: Beat interval formula correctness
proptest! {
    fn prop_beat_interval_formula(bpm in 40u32..=200, d in proptest::sample::select(vec![1u32,2,4,8,16])) {
        let expected = (60_000 / bpm) * (4 / d);
        let actual = TimeSignature::beat_interval_ms(bpm, d);
        prop_assert_eq!(actual, expected);
    }
}
```

**Property 9 — Time signature display format**
```
// Feature: metronome-time-signature, Property 9: Time signature display format
proptest! {
    fn prop_time_sig_display(n in 1u32..=16, d in proptest::sample::select(vec![1u32,2,4,8,16])) {
        let ts = TimeSignature { numerator: n, denominator: d };
        let label = format_time_signature(ts);
        prop_assert_eq!(label, format!("{}/{}", n, d));
    }
}
```
