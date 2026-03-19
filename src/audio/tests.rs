use crate::music_theory::{diatonic_chords, scale_notes, Key, Mode, PitchClass};
use crate::data::find_progression;

use super::{
    chord_note_sequence, pitch_to_freq, progression_chord_sequences, scale_note_sequence,
    AudioEngine,
};

// ── Task 14.1 ─────────────────────────────────────────────────────────────────
// Feature: circle-of-fifths, Property 19: Audio note sequence correctness

#[test]
fn scale_note_sequence_matches_scale_notes_all_major_keys() {
    for root_idx in 0u8..12 {
        let key = Key { root: PitchClass::from_index(root_idx), mode: Mode::Major };
        let expected: Vec<PitchClass> = scale_notes(key).to_vec();
        let actual = scale_note_sequence(key);
        assert_eq!(actual, expected, "scale_note_sequence mismatch for {:?}", key.root);
    }
}

#[test]
fn scale_note_sequence_matches_scale_notes_all_minor_keys() {
    for root_idx in 0u8..12 {
        let key = Key { root: PitchClass::from_index(root_idx), mode: Mode::Minor };
        let expected: Vec<PitchClass> = scale_notes(key).to_vec();
        let actual = scale_note_sequence(key);
        assert_eq!(actual, expected, "scale_note_sequence mismatch for {:?} minor", key.root);
    }
}

#[test]
fn chord_note_sequence_returns_root_third_fifth() {
    let key = Key::major(PitchClass::C);
    let chords = diatonic_chords(key);
    for chord in &chords {
        let seq = chord_note_sequence(&chord.notes);
        assert_eq!(seq.len(), 3);
        assert_eq!(seq[0], chord.notes[0], "root mismatch for {:?}", chord.degree);
        assert_eq!(seq[1], chord.notes[1], "third mismatch for {:?}", chord.degree);
        assert_eq!(seq[2], chord.notes[2], "fifth mismatch for {:?}", chord.degree);
    }
}

#[test]
fn progression_chord_sequences_matches_progression_order() {
    // Test for all 12 major keys using progression ID 0..47 (4 per key × 12 keys)
    for id in 0u32..48 {
        if let Some(prog) = find_progression(id) {
            let sequences = progression_chord_sequences(&prog);
            assert_eq!(
                sequences.len(),
                prog.chords.len(),
                "chord count mismatch for progression id={}", id
            );
            let chords = diatonic_chords(prog.key);
            for (i, degree) in prog.chords.iter().enumerate() {
                let chord = chords.iter().find(|c| c.degree == *degree).unwrap();
                assert_eq!(
                    sequences[i],
                    chord.notes.to_vec(),
                    "chord notes mismatch at index {} for progression id={}", i, id
                );
            }
        }
    }
}

// ── Task 14.2 ─────────────────────────────────────────────────────────────────
// Unit tests for AudioEngine degraded mode

#[test]
fn degraded_engine_is_degraded() {
    let engine = AudioEngine::new_degraded("test error".to_string());
    assert!(engine.is_degraded());
    assert_eq!(engine.error, Some("test error".to_string()));
}

#[test]
fn fresh_degraded_engine_is_not_muted() {
    let engine = AudioEngine::new_degraded("err".to_string());
    assert!(!engine.is_muted());
}

#[test]
fn degraded_engine_play_scale_does_not_panic() {
    let engine = AudioEngine::new_degraded("err".to_string());
    let key = Key::major(PitchClass::C);
    engine.play_scale(key); // must not panic
}

#[test]
fn degraded_engine_play_chord_does_not_panic() {
    let engine = AudioEngine::new_degraded("err".to_string());
    let notes = [PitchClass::C, PitchClass::E, PitchClass::G];
    engine.play_chord(&notes); // must not panic
}

#[test]
fn degraded_engine_play_progression_does_not_panic() {
    let engine = AudioEngine::new_degraded("err".to_string());
    if let Some(prog) = find_progression(0) {
        engine.play_progression(&prog); // must not panic
    }
}

#[test]
fn degraded_engine_stop_does_not_panic() {
    let engine = AudioEngine::new_degraded("err".to_string());
    engine.stop(); // must not panic
}

#[test]
fn muted_engine_play_does_not_panic() {
    let mut engine = AudioEngine::new_degraded("err".to_string());
    engine.set_muted(true);
    assert!(engine.is_muted());
    let key = Key::major(PitchClass::C);
    engine.play_scale(key); // must not panic
}

// ── Frequency helper ──────────────────────────────────────────────────────────

#[test]
fn pitch_to_freq_a4_is_440hz() {
    let freq = pitch_to_freq(PitchClass::A, 4);
    assert!((freq - 440.0).abs() < 0.01, "A4 = {}, expected ~440", freq);
}

#[test]
fn pitch_to_freq_c4_is_middle_c() {
    let freq = pitch_to_freq(PitchClass::C, 4);
    assert!((freq - 261.63).abs() < 0.1, "C4 = {}, expected ~261.63", freq);
}

#[test]
fn pitch_to_freq_increases_with_octave() {
    let c4 = pitch_to_freq(PitchClass::C, 4);
    let c5 = pitch_to_freq(PitchClass::C, 5);
    assert!((c5 / c4 - 2.0).abs() < 0.001, "C5 should be exactly one octave above C4");
}
