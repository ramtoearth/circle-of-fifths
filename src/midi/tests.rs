use super::*;
use crate::music_theory::PitchClass;

// ─────────────────────────── HeldNote::from_midi ──────────────────────────

#[test]
fn test_from_midi_c4() {
    // MIDI note 60 = C4 (octave 4)
    let h = HeldNote::from_midi(60, 100);
    assert_eq!(h.midi_note, 60);
    assert_eq!(h.velocity, 100);
    assert_eq!(h.pitch_class, PitchClass::C);
    assert_eq!(h.octave, 4);
}

#[test]
fn test_from_midi_a4() {
    // MIDI note 69 = A4
    let h = HeldNote::from_midi(69, 64);
    assert_eq!(h.pitch_class, PitchClass::A);
    assert_eq!(h.octave, 4);
}

#[test]
fn test_from_midi_lowest() {
    // MIDI note 0 = C-1
    let h = HeldNote::from_midi(0, 1);
    assert_eq!(h.pitch_class, PitchClass::C);
    assert_eq!(h.octave, -1);
}

#[test]
fn test_from_midi_highest() {
    // MIDI note 127 = G9
    let h = HeldNote::from_midi(127, 127);
    assert_eq!(h.pitch_class, PitchClass::G);
    assert_eq!(h.octave, 9);
}

#[test]
fn test_from_midi_pitch_class_formula() {
    // Check pitch_class matches PitchClass::from_index(note % 12) for all notes
    for note in 0u8..=127 {
        let h = HeldNote::from_midi(note, 64);
        assert_eq!(h.pitch_class, PitchClass::from_index(note % 12));
    }
}

#[test]
fn test_from_midi_octave_formula() {
    // Check octave matches (note / 12) as i8 - 1 for all notes
    for note in 0u8..=127 {
        let h = HeldNote::from_midi(note, 64);
        assert_eq!(h.octave, (note / 12) as i8 - 1);
    }
}

// ─────────────────────────── velocity_opacity ─────────────────────────────

#[test]
fn test_velocity_opacity_min() {
    let h = HeldNote::from_midi(60, 1);
    let opacity = h.velocity_opacity();
    assert!((opacity - 0.35).abs() < 1e-5, "Expected 0.35, got {}", opacity);
}

#[test]
fn test_velocity_opacity_max() {
    let h = HeldNote::from_midi(60, 127);
    let opacity = h.velocity_opacity();
    assert!((opacity - 1.0).abs() < 1e-5, "Expected 1.0, got {}", opacity);
}

#[test]
fn test_velocity_opacity_monotone() {
    let mut prev = HeldNote::from_midi(60, 1).velocity_opacity();
    for v in 2u8..=127 {
        let curr = HeldNote::from_midi(60, v).velocity_opacity();
        assert!(curr > prev, "opacity should increase: v={} gave {} <= {}", v, curr, prev);
        prev = curr;
    }
}

// ─────────────────────────── MidiStatus ───────────────────────────────────

#[test]
fn test_midi_status_variants() {
    assert_ne!(MidiStatus::Unavailable, MidiStatus::Connected);
    assert_ne!(MidiStatus::PermissionDenied, MidiStatus::NoDevices);
    assert_eq!(MidiStatus::Connected, MidiStatus::Connected);
}

// ─────────────────────────── Score types ──────────────────────────────────

#[test]
fn test_practice_score_default() {
    let s = PracticeScore::default();
    assert_eq!(s.correct_notes, 0);
    assert_eq!(s.total_notes_played, 0);
}

#[test]
fn test_play_along_score_default() {
    let s = PlayAlongScore::default();
    assert!(s.chord_results.is_empty());
}
