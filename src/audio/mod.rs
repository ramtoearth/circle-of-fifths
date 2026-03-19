use std::cell::RefCell;
use std::rc::Rc;

use crate::music_theory::{diatonic_chords, scale_notes, Key, PitchClass, Progression};

#[cfg(test)]
mod tests;

// ── Frequency helper ──────────────────────────────────────────────────────────

/// Convert pitch class + octave to frequency in Hz (equal temperament).
/// Middle C (C4) = MIDI 60 = ~261.63 Hz; A4 = MIDI 69 = 440 Hz.
pub fn pitch_to_freq(pitch: PitchClass, octave: i32) -> f32 {
    let semitone = pitch.to_index() as i32;
    // MIDI note: octave 4 starts at MIDI 60 (C4), so midi = (octave+1)*12 + semitone
    let midi = (octave + 1) * 12 + semitone;
    440.0 * 2_f32.powf((midi as f32 - 69.0) / 12.0)
}

// ── Pure note-sequence helpers (testable without AudioContext) ────────────────

/// Returns the ordered notes to play for a scale (ascending, one note per 300 ms).
/// Validates Property 19 (scale portion).
pub fn scale_note_sequence(key: Key) -> Vec<PitchClass> {
    scale_notes(key).to_vec()
}

/// Returns the notes to play simultaneously for a chord.
/// Validates Property 19 (chord portion).
pub fn chord_note_sequence(notes: &[PitchClass]) -> Vec<PitchClass> {
    notes.to_vec()
}

/// Returns (PitchClass, octave) pairs for all 8 scale notes (root to root one octave up).
/// Used to drive octave-aware visual highlights in the UI.
pub fn scale_note_sequence_with_octaves(key: Key) -> Vec<(PitchClass, i32)> {
    let notes = scale_note_sequence(key);
    let mut result = Vec::with_capacity(8);
    let mut octave = 4i32;
    let mut prev_semitone = notes.first().map(|p| p.to_index() as i32 - 1).unwrap_or(-1);
    for &pitch in &notes {
        let semitone = pitch.to_index() as i32;
        if semitone <= prev_semitone {
            octave += 1;
        }
        prev_semitone = semitone;
        result.push((pitch, octave));
    }
    // 8th note: root one octave up
    if let Some(&root) = notes.first() {
        let root_semitone = root.to_index() as i32;
        if root_semitone <= prev_semitone {
            octave += 1;
        }
        result.push((root, octave));
    }
    result
}

/// Returns, in order, the notes of each chord in a progression.
/// Validates Property 19 (progression portion).
pub fn progression_chord_sequences(progression: &Progression) -> Vec<Vec<PitchClass>> {
    let chords = diatonic_chords(progression.key);
    progression
        .chords
        .iter()
        .filter_map(|degree| chords.iter().find(|c| c.degree == *degree))
        .map(|c| c.notes.to_vec())
        .collect()
}

// ── AudioEngine ───────────────────────────────────────────────────────────────

pub struct AudioEngine {
    #[cfg(target_arch = "wasm32")]
    ctx: Option<web_sys::AudioContext>,
    muted: bool,
    pub error: Option<String>,
}

impl AudioEngine {
    /// Create a new AudioEngine, attempting to open a WebAudio context.
    /// On failure the engine enters degraded mode and records the error message.
    pub fn new() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            match web_sys::AudioContext::new() {
                Ok(ctx) => AudioEngine { ctx: Some(ctx), muted: false, error: None },
                Err(e) => {
                    let err = format!("AudioContext init failed: {:?}", e);
                    web_sys::console::warn_1(&err.clone().into());
                    AudioEngine { ctx: None, muted: false, error: Some(err) }
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        AudioEngine { muted: false, error: None }
    }

    /// Construct an engine already in degraded mode (useful for testing).
    pub fn new_degraded(error: String) -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            AudioEngine { ctx: None, muted: false, error: Some(error) }
        }
        #[cfg(not(target_arch = "wasm32"))]
        AudioEngine { muted: false, error: Some(error) }
    }

    /// Returns true when the AudioContext could not be created.
    pub fn is_degraded(&self) -> bool {
        self.error.is_some()
    }

    pub fn set_muted(&mut self, muted: bool) {
        self.muted = muted;
    }

    pub fn is_muted(&self) -> bool {
        self.muted
    }

    /// Play all 8 scale notes (root to root one octave up) in ascending order at the given BPM.
    /// Requirement 7.1
    pub fn play_scale(&self, key: Key, bpm: u32) {
        if self.muted {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(ctx) = &self.ctx {
            let _ = ctx.resume();
            let notes = scale_note_sequence(key);
            let now = ctx.current_time();
            let interval = 60.0 / bpm as f64;
            let duration = interval * 0.85;
            let mut octave = 4i32;
            let mut prev_semitone = notes.first().map(|p| p.to_index() as i32 - 1).unwrap_or(-1);
            for (i, &pitch) in notes.iter().enumerate() {
                let semitone = pitch.to_index() as i32;
                if semitone <= prev_semitone {
                    octave += 1;
                }
                prev_semitone = semitone;
                let start = now + (i as f64) * interval;
                let freq = pitch_to_freq(pitch, octave);
                self.schedule_note(ctx, freq, start, duration);
            }
            // 8th note: root one octave up to complete the scale
            if !notes.is_empty() {
                let root = notes[0];
                let root_semitone = root.to_index() as i32;
                if root_semitone <= prev_semitone {
                    octave += 1;
                }
                let start = now + 7.0 * interval;
                let freq = pitch_to_freq(root, octave);
                self.schedule_note(ctx, freq, start, duration);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = (key, bpm);
    }

    /// Play all notes of a chord simultaneously.
    /// Requirement 7.2
    pub fn play_chord(&self, notes: &[PitchClass]) {
        if self.muted {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(ctx) = &self.ctx {
            let _ = ctx.resume();
            let now = ctx.current_time();
            for &pitch in notes {
                let freq = pitch_to_freq(pitch, 4);
                self.schedule_note(ctx, freq, now, 0.9);
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = notes;
    }

    /// Play each chord in the progression for 1 second in sequence.
    /// Requirement 7.3
    pub fn play_progression(&self, progression: &Progression) {
        if self.muted {
            return;
        }
        #[cfg(target_arch = "wasm32")]
        if let Some(ctx) = &self.ctx {
            let _ = ctx.resume();
            let sequences = progression_chord_sequences(progression);
            let now = ctx.current_time();
            for (i, chord_notes) in sequences.iter().enumerate() {
                let start = now + (i as f64) * 1.0;
                for &pitch in chord_notes {
                    let freq = pitch_to_freq(pitch, 4);
                    self.schedule_note(ctx, freq, start, 0.9);
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = progression;
    }

    /// Suspend audio playback immediately.
    /// Requirement 7.6
    pub fn stop(&self) {
        #[cfg(target_arch = "wasm32")]
        if let Some(ctx) = &self.ctx {
            let _ = ctx.suspend();
        }
    }

    // ── Internal ──────────────────────────────────────────────────────────────

    #[cfg(target_arch = "wasm32")]
    fn schedule_note(
        &self,
        ctx: &web_sys::AudioContext,
        freq: f32,
        start: f64,
        duration: f64,
    ) {
        use web_sys::OscillatorType;

        let oscillator = match ctx.create_oscillator() {
            Ok(o) => o,
            Err(_) => return,
        };
        let gain_node = match ctx.create_gain() {
            Ok(g) => g,
            Err(_) => return,
        };

        oscillator.set_type(OscillatorType::Sine);
        oscillator.frequency().set_value(freq);

        // Envelope: short attack, decay to sustain, release at end
        let gain_param = gain_node.gain();
        let _ = gain_param.set_value_at_time(0.0, start);
        let _ = gain_param.linear_ramp_to_value_at_time(0.3, start + 0.01);
        let _ = gain_param.linear_ramp_to_value_at_time(0.0, start + duration);

        if oscillator
            .connect_with_audio_node(&gain_node)
            .and_then(|_| gain_node.connect_with_audio_node(&ctx.destination()))
            .is_ok()
        {
            let _ = oscillator.start_with_when(start);
            let _ = oscillator.stop_with_when(start + duration + 0.05);
        }
    }
}

// ── Yew context handle ────────────────────────────────────────────────────────

/// Thread-local handle to `AudioEngine`, safe to clone and pass via Yew context.
#[derive(Clone)]
pub struct AudioEngineHandle(pub Rc<RefCell<AudioEngine>>);

impl AudioEngineHandle {
    pub fn new() -> Self {
        AudioEngineHandle(Rc::new(RefCell::new(AudioEngine::new())))
    }

    /// Returns the initialization error message, if any.
    pub fn error(&self) -> Option<String> {
        self.0.borrow().error.clone()
    }

    pub fn play_scale(&self, key: Key, bpm: u32) {
        self.0.borrow().play_scale(key, bpm);
    }

    pub fn play_chord(&self, notes: &[PitchClass]) {
        self.0.borrow().play_chord(notes);
    }

    pub fn play_progression(&self, progression: &Progression) {
        self.0.borrow().play_progression(progression);
    }

    pub fn stop(&self) {
        self.0.borrow().stop();
    }

    pub fn set_muted(&self, muted: bool) {
        self.0.borrow_mut().set_muted(muted);
    }

    pub fn is_muted(&self) -> bool {
        self.0.borrow().is_muted()
    }
}

impl PartialEq for AudioEngineHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}
