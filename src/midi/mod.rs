use std::collections::HashSet;
use crate::music_theory::{
    ChordQuality, Key, Mode, PitchClass, ScaleDegree,
    diatonic_chords, roman_numeral as rn_fn, scale_notes,
};

// ─────────────────────────── Raw MIDI event ───────────────────────────────

/// Raw parsed MIDI event
#[derive(Clone, Debug, PartialEq)]
pub enum MidiEvent {
    NoteOn  { channel: u8, note: u8, velocity: u8 },
    NoteOff { channel: u8, note: u8 },
    Other,
}

// ─────────────────────────── HeldNote ─────────────────────────────────────

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

// ─────────────────────────── MidiStatus ───────────────────────────────────

/// MIDI subsystem connection state
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MidiStatus {
    Unavailable,      // browser doesn't support Web MIDI
    PermissionDenied, // user denied access
    NoDevices,        // access granted but no inputs
    Connected,        // at least one input active
}

// ─────────────────────────── Chord / Key suggestion types ─────────────────

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

// ─────────────────────────── Scoring types ────────────────────────────────

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

// ─────────────────────────── Chord Recognition (Task 3) ─────────────────────

/// Chord dictionary: (name_suffix, semitone_intervals_from_root).
/// Triads and seventh chords covering all common chord types.
pub static CHORD_DICTIONARY: &[(&str, &[u8])] = &[
    // Triads
    ("",     &[0, 4, 7]),        // Major
    ("m",    &[0, 3, 7]),        // Minor
    ("dim",  &[0, 3, 6]),        // Diminished
    ("aug",  &[0, 4, 8]),        // Augmented
    // Seventh chords
    ("maj7", &[0, 4, 7, 11]),    // Major 7
    ("m7",   &[0, 3, 7, 10]),    // Minor 7
    ("7",    &[0, 4, 7, 10]),    // Dominant 7
    ("m7b5", &[0, 3, 6, 10]),    // Half-diminished (m7♭5)
    ("dim7", &[0, 3, 6, 9]),     // Diminished 7
];

/// Map a chord name suffix to its base triad quality for diatonic analysis.
fn suffix_to_quality(suffix: &str) -> Option<ChordQuality> {
    match suffix {
        "" | "maj7" => Some(ChordQuality::Major),
        "m" | "m7"  => Some(ChordQuality::Minor),
        "dim" | "m7b5" | "dim7" => Some(ChordQuality::Diminished),
        "7" => Some(ChordQuality::Major), // dominant 7 has a major triad base
        _   => None,                       // "aug" and unknown → no standard diatonic quality
    }
}

/// Recognizes a chord from held notes.
///
/// Returns `None` when fewer than 3 distinct PitchClasses are held (Property 5).
/// Tries every held pitch class as a potential chord root (covers all inversions).
/// Prefers longer matches; breaks ties by fewest extra held notes.
/// If no dictionary entry matches, returns note names only (unrecognized chord).
/// When `selected_key` is provided, populates `roman_numeral` and `is_diatonic`.
pub fn recognize_chord(held: &[HeldNote], selected_key: Option<Key>) -> Option<RecognizedChord> {
    let held_set: HashSet<PitchClass> = held.iter().map(|n| n.pitch_class).collect();

    // Property 5: fewer than 3 distinct pitch classes → no chord
    if held_set.len() < 3 {
        return None;
    }

    let mut best_score: usize = 0;
    let mut best_extra: usize = usize::MAX;
    let mut best_name = String::new();
    let mut best_root = PitchClass::C;
    let mut best_suffix: &str = "";
    let mut found = false;

    // Try each held pitch class as a potential chord root
    for &pc in &held_set {
        for &(suffix, intervals) in CHORD_DICTIONARY {
            let chord_pcs: HashSet<PitchClass> =
                intervals.iter().map(|&i| pc.add_semitones(i)).collect();

            // All chord notes must be present in held notes (subset check)
            if !chord_pcs.iter().all(|c| held_set.contains(c)) {
                continue;
            }

            let score = chord_pcs.len();
            let extra = held_set.len() - score;

            // Prefer higher score; break ties by fewest extra notes
            let is_better = !found
                || score > best_score
                || (score == best_score && extra < best_extra);

            if is_better {
                found = true;
                best_score = score;
                best_extra = extra;
                best_name = format!("{}{}", pc.name(), suffix);
                best_root = pc;
                best_suffix = suffix;
            }
        }
    }

    // Stable sorted output for pitch_classes
    let mut pitch_classes: Vec<PitchClass> = held_set.into_iter().collect();
    pitch_classes.sort_by_key(|pc| pc.to_index());

    if !found {
        // Unrecognized: return note names joined by spaces
        let name = pitch_classes
            .iter()
            .map(|pc| pc.name())
            .collect::<Vec<_>>()
            .join(" ");
        return Some(RecognizedChord {
            name,
            pitch_classes,
            roman_numeral: None,
            is_diatonic: selected_key.map(|_| false),
        });
    }

    let (roman_numeral, is_diatonic) = selected_key
        .map(|key| annotate_chord(best_root, best_suffix, key))
        .unwrap_or((None, None));

    Some(RecognizedChord {
        name: best_name,
        pitch_classes,
        roman_numeral,
        is_diatonic,
    })
}

/// Compute roman numeral and diatonic status for a chord root+suffix in a key.
/// `is_diatonic` is always `Some(bool)` (not None) since a key is provided.
fn annotate_chord(root: PitchClass, suffix: &str, key: Key) -> (Option<String>, Option<bool>) {
    let notes = scale_notes(key);
    let chords = diatonic_chords(key);
    const DEGREES: [ScaleDegree; 7] = [
        ScaleDegree::I, ScaleDegree::II, ScaleDegree::III, ScaleDegree::IV,
        ScaleDegree::V, ScaleDegree::VI, ScaleDegree::VII,
    ];

    let quality_opt = suffix_to_quality(suffix);
    let degree_idx = notes.iter().position(|&n| n == root);

    // Roman numeral only when root is in the scale and quality is recognized
    let roman = degree_idx.and_then(|idx| {
        quality_opt.map(|q| rn_fn(DEGREES[idx], q).to_string())
    });

    // is_diatonic is always Some(bool) when a key is selected
    let is_dia = Some(match (degree_idx, quality_opt) {
        (Some(idx), Some(quality)) => chords[idx].quality == quality,
        _ => false, // root not in scale, or augmented → not diatonic
    });

    (roman, is_dia)
}

// ─────────────────────────── Key detection (Task 4) ──────────────────────────

/// Returns only entries where `now_ms - timestamp_ms <= 10_000.0`.
pub fn filter_rolling_window(entries: &[(PitchClass, f64)], now_ms: f64) -> Vec<(PitchClass, f64)> {
    entries
        .iter()
        .filter(|(_, ts)| now_ms - ts <= 10_000.0)
        .copied()
        .collect()
}

/// Scores all 24 keys against the distinct PitchClasses in the rolling window
/// and returns the top 3 by score (descending).
/// Returns an empty Vec when fewer than 4 distinct PitchClasses are present.
pub fn detect_keys(window: &[(PitchClass, f64)], now_ms: f64) -> Vec<KeySuggestion> {
    let filtered = filter_rolling_window(window, now_ms);

    let distinct: HashSet<PitchClass> = filtered.iter().map(|(pc, _)| *pc).collect();

    if distinct.len() < 4 {
        return vec![];
    }

    let all_roots = [
        PitchClass::C,  PitchClass::Db, PitchClass::D,  PitchClass::Eb,
        PitchClass::E,  PitchClass::F,  PitchClass::Gb, PitchClass::G,
        PitchClass::Ab, PitchClass::A,  PitchClass::Bb, PitchClass::B,
    ];

    let mut suggestions: Vec<KeySuggestion> = Vec::with_capacity(24);
    for root in all_roots {
        for mode in [Mode::Major, Mode::Minor] {
            let key = Key { root, mode };
            let notes = scale_notes(key);
            let score = distinct.iter().filter(|pc| notes.contains(pc)).count() as u8;
            suggestions.push(KeySuggestion { key, score });
        }
    }

    suggestions.sort_by(|a, b| b.score.cmp(&a.score));
    suggestions.truncate(3);
    suggestions
}

// ─────────────────────────── MIDI message parsing (Task 6) ──────────────────

/// Parse raw MIDI bytes into a `MidiEvent`.
/// A NoteOn with velocity=0 is normalised to `NoteOff` (MIDI Running Status convention).
pub fn parse_midi_message(data: &[u8]) -> MidiEvent {
    if data.is_empty() {
        return MidiEvent::Other;
    }
    let status   = data[0];
    let msg_type = status & 0xF0;
    let channel  = status & 0x0F;
    match msg_type {
        0x80 if data.len() >= 3 => MidiEvent::NoteOff { channel, note: data[1] & 0x7F },
        0x90 if data.len() >= 3 => {
            let note     = data[1] & 0x7F;
            let velocity = data[2] & 0x7F;
            if velocity == 0 {
                MidiEvent::NoteOff { channel, note }
            } else {
                MidiEvent::NoteOn { channel, note, velocity }
            }
        }
        _ => MidiEvent::Other,
    }
}

// ─────────────────────────── MidiEngine – non-WASM stub ──────────────────────

/// In non-WASM builds (e.g. `cargo test`) `MidiEngine` is an inert stub.
/// All live browser-API logic lives in the `#[cfg(target_arch = "wasm32")]`
/// impl block below.
#[cfg(not(target_arch = "wasm32"))]
pub struct MidiEngine;

#[cfg(not(target_arch = "wasm32"))]
impl MidiEngine {
    pub fn request_access(_dispatch: yew::Callback<crate::state::AppAction>) -> Self {
        Self
    }
    pub fn register_ports(&self, _dispatch: yew::Callback<crate::state::AppAction>) {}
    pub fn connected_device_names(&self) -> Vec<String> { vec![] }
}

// ─────────────────────────── MidiEngine – WASM implementation ────────────────

#[cfg(target_arch = "wasm32")]
use std::rc::Rc;
#[cfg(target_arch = "wasm32")]
use std::cell::RefCell;
#[cfg(target_arch = "wasm32")]
use js_sys::Reflect;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::{spawn_local, JsFuture};
#[cfg(target_arch = "wasm32")]
use yew::Callback;
#[cfg(target_arch = "wasm32")]
use crate::state::AppAction;

/// Owns the JS `MIDIAccess` object and every `Closure` registered on MIDI
/// ports.  Dropping this struct releases all JS callbacks.
#[cfg(target_arch = "wasm32")]
pub struct MidiEngine {
    midi_access: Rc<RefCell<Option<JsValue>>>,
    closures:    Rc<RefCell<Vec<Closure<dyn Fn(JsValue)>>>>,
}

#[cfg(target_arch = "wasm32")]
impl MidiEngine {
    /// Request Web MIDI access and return immediately.
    ///
    /// Dispatches `MidiStatusChanged` / `MidiDevicesChanged` asynchronously
    /// once the browser resolves (or rejects) the `requestMIDIAccess` promise.
    /// On any interop failure dispatches `MidiStatus::Unavailable` rather than
    /// panicking.
    pub fn request_access(dispatch: Callback<AppAction>) -> Self {
        let midi_access: Rc<RefCell<Option<JsValue>>> = Rc::new(RefCell::new(None));
        let closures: Rc<RefCell<Vec<Closure<dyn Fn(JsValue)>>>> =
            Rc::new(RefCell::new(vec![]));
        let engine = MidiEngine {
            midi_access: Rc::clone(&midi_access),
            closures:    Rc::clone(&closures),
        };

        // ── Locate navigator.requestMIDIAccess ───────────────────────────────
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::Unavailable));
                return engine;
            }
        };
        let navigator: JsValue = window.navigator().into();

        let rma_val = match Reflect::get(&navigator, &JsValue::from_str("requestMIDIAccess")) {
            Ok(v) if !v.is_undefined() && !v.is_null() => v,
            _ => {
                dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::Unavailable));
                return engine;
            }
        };
        let rma_fn = match rma_val.dyn_into::<js_sys::Function>() {
            Ok(f) => f,
            Err(_) => {
                dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::Unavailable));
                return engine;
            }
        };

        // ── Call requestMIDIAccess() → Promise ───────────────────────────────
        let promise_val = match rma_fn.call0(&navigator) {
            Ok(p) => p,
            Err(_) => {
                dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::Unavailable));
                return engine;
            }
        };
        let promise = match promise_val.dyn_into::<js_sys::Promise>() {
            Ok(p) => p,
            Err(_) => {
                dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::Unavailable));
                return engine;
            }
        };

        // ── Await the promise asynchronously ─────────────────────────────────
        let midi_access_async = Rc::clone(&midi_access);
        let closures_async    = Rc::clone(&closures);
        spawn_local(async move {
            match JsFuture::from(promise).await {
                Ok(access) => {
                    *midi_access_async.borrow_mut() = Some(access.clone());

                    let names  = midi_input_names(&access);
                    let status = if names.is_empty() {
                        MidiStatus::NoDevices
                    } else {
                        MidiStatus::Connected
                    };
                    dispatch.emit(AppAction::MidiStatusChanged(status));
                    dispatch.emit(AppAction::MidiDevicesChanged(names));

                    // Register onmidimessage on all current input ports
                    register_port_handlers(&access, &closures_async, &dispatch);

                    // Hot-plug: onstatechange fires on connect / disconnect
                    let dispatch_hplug = dispatch.clone();
                    let access_hplug   = access.clone();
                    let closures_hplug = Rc::clone(&closures_async);
                    let on_state_change: Closure<dyn Fn(JsValue)> =
                        Closure::wrap(Box::new(move |_event: JsValue| {
                            let names  = midi_input_names(&access_hplug);
                            let status = if names.is_empty() {
                                MidiStatus::NoDevices
                            } else {
                                MidiStatus::Connected
                            };
                            dispatch_hplug.emit(AppAction::MidiStatusChanged(status));
                            dispatch_hplug.emit(AppAction::MidiDevicesChanged(names));
                            // Re-register handlers for newly connected ports
                            register_port_handlers(&access_hplug, &closures_hplug, &dispatch_hplug);
                        }));
                    let _ = Reflect::set(
                        &access,
                        &JsValue::from_str("onstatechange"),
                        on_state_change.as_ref().unchecked_ref(),
                    );
                    closures_async.borrow_mut().push(on_state_change);
                }
                Err(_) => {
                    dispatch.emit(AppAction::MidiStatusChanged(MidiStatus::PermissionDenied));
                }
            }
        });

        engine
    }

    /// Register `onmidimessage` on all current input ports (public so `App`
    /// can call it after a hot-plug event if needed).
    pub fn register_ports(&self, dispatch: Callback<AppAction>) {
        if let Some(ref access) = *self.midi_access.borrow() {
            register_port_handlers(access, &self.closures, &dispatch);
        }
    }

    /// Returns the display names of all currently connected MIDI input devices.
    pub fn connected_device_names(&self) -> Vec<String> {
        match *self.midi_access.borrow() {
            Some(ref access) => midi_input_names(access),
            None => vec![],
        }
    }
}

// ─────────────────────────── WASM private helpers ────────────────────────────

/// Iterate `MIDIAccess.inputs.values()` and return each port as a `JsValue`.
#[cfg(target_arch = "wasm32")]
fn midi_input_ports(midi_access: &JsValue) -> Vec<JsValue> {
    let inputs = match Reflect::get(midi_access, &JsValue::from_str("inputs")) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let values_method = match Reflect::get(&inputs, &JsValue::from_str("values")) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let values_fn = match values_method.dyn_into::<js_sys::Function>() {
        Ok(f) => f,
        Err(_) => return vec![],
    };
    let iter = match values_fn.call0(&inputs) {
        Ok(i) => i,
        Err(_) => return vec![],
    };

    let mut ports = vec![];
    loop {
        let next_method = match Reflect::get(&iter, &JsValue::from_str("next")) {
            Ok(v) => v,
            Err(_) => break,
        };
        let next_fn = match next_method.dyn_into::<js_sys::Function>() {
            Ok(f) => f,
            Err(_) => break,
        };
        let step = match next_fn.call0(&iter) {
            Ok(s) => s,
            Err(_) => break,
        };
        let done = Reflect::get(&step, &JsValue::from_str("done"))
            .map(|v| v.is_truthy())
            .unwrap_or(true);
        if done { break; }
        match Reflect::get(&step, &JsValue::from_str("value")) {
            Ok(v) if !v.is_undefined() && !v.is_null() => ports.push(v),
            _ => break,
        }
    }
    ports
}

/// Collect the `name` property from each input port.
#[cfg(target_arch = "wasm32")]
fn midi_input_names(midi_access: &JsValue) -> Vec<String> {
    midi_input_ports(midi_access)
        .into_iter()
        .filter_map(|port| {
            Reflect::get(&port, &JsValue::from_str("name"))
                .ok()
                .and_then(|v| v.as_string())
        })
        .collect()
}

/// Set `onmidimessage` on every input port returned by `midi_input_ports`.
/// Each new `Closure` is pushed into `closures` to keep it alive.
#[cfg(target_arch = "wasm32")]
fn register_port_handlers(
    midi_access: &JsValue,
    closures: &Rc<RefCell<Vec<Closure<dyn Fn(JsValue)>>>>,
    dispatch: &Callback<AppAction>,
) {
    for port in midi_input_ports(midi_access) {
        let dispatch_clone = dispatch.clone();
        let on_message: Closure<dyn Fn(JsValue)> = Closure::wrap(Box::new(move |event: JsValue| {
            let data_val = match Reflect::get(&event, &JsValue::from_str("data")) {
                Ok(v) => v,
                Err(_) => return,
            };
            let data_arr: js_sys::Uint8Array = match data_val.dyn_into() {
                Ok(a) => a,
                Err(_) => return,
            };
            let bytes = data_arr.to_vec();
            match parse_midi_message(&bytes) {
                MidiEvent::NoteOn { note, velocity, .. } => {
                    let held = HeldNote::from_midi(note, velocity);
                    let now  = js_sys::Date::now();
                    dispatch_clone.emit(AppAction::MidiNoteOn(held, now));
                }
                MidiEvent::NoteOff { note, .. } => {
                    dispatch_clone.emit(AppAction::MidiNoteOff(note));
                }
                MidiEvent::Other => {}
            }
        }));
        let _ = Reflect::set(
            &port,
            &JsValue::from_str("onmidimessage"),
            on_message.as_ref().unchecked_ref(),
        );
        closures.borrow_mut().push(on_message);
    }
}

// ─────────────────────────── Tests ────────────────────────────────────────

#[cfg(test)]
mod tests;
