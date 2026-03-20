use crate::music_theory::{Key, PitchClass};

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

// ─────────────────────────── Tests ────────────────────────────────────────

#[cfg(test)]
mod tests;
