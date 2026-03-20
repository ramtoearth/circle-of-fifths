use super::*;
use crate::music_theory::{Key, Mode, PitchClass};

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

// ─────────────────────────── Chord Recognition (Task 3) ───────────────────────

fn make_held(pcs: &[PitchClass]) -> Vec<HeldNote> {
    pcs.iter()
        .enumerate()
        .map(|(i, &pc)| HeldNote {
            midi_note: 60 + pc.to_index(),
            velocity: 64,
            pitch_class: pc,
            octave: 4 + (i as i8 / 12),
        })
        .collect()
}

// Unit: C major triad → "C"
#[test]
fn chord_c_major_recognized() {
    let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "C");
}

// Unit: A minor triad → "Am"
#[test]
fn chord_a_minor_recognized() {
    let held = make_held(&[PitchClass::A, PitchClass::C, PitchClass::E]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "Am");
}

// Unit: G dominant 7 → "G7"
#[test]
fn chord_g_dominant_7_recognized() {
    let held = make_held(&[PitchClass::G, PitchClass::B, PitchClass::D, PitchClass::F]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "G7");
}

// Unit: fewer than 3 distinct PitchClasses → None
#[test]
fn chord_two_notes_returns_none() {
    let held = make_held(&[PitchClass::C, PitchClass::E]);
    assert!(recognize_chord(&held, None).is_none());
}

// Unit: zero notes → None
#[test]
fn chord_no_notes_returns_none() {
    assert!(recognize_chord(&[], None).is_none());
}

// Unit: unrecognized set returns note names without chord label
#[test]
fn chord_unrecognized_returns_note_names() {
    // C + F# + B doesn't form a standard chord
    let held = make_held(&[PitchClass::C, PitchClass::Gb, PitchClass::B]);
    let result = recognize_chord(&held, None).unwrap();
    // Name should contain individual note names, not a chord suffix like "m" or "maj7"
    assert!(result.name.contains("C") || result.name.contains("G") || result.name.contains("B"));
    assert!(!result.name.ends_with("m7b5") && !result.name.ends_with("maj7"));
}

// Unit: 4-note input matching a triad still works (with extra note)
#[test]
fn chord_c_major_with_extra_note() {
    // C-E-G-D: C major matches (score=3, extra=1 for D).
    // No other 3-note or 4-note chord covers C, E, G with D as a subset.
    let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G, PitchClass::D]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "C");
}

// Unit: 4-note match preferred over 3-note match (Cmaj7 > C major)
#[test]
fn chord_prefers_longer_match() {
    let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G, PitchClass::B]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "Cmaj7");
}

// Unit: C major 1st inversion (E-G-C) → still "C"
#[test]
fn chord_first_inversion_c_major() {
    let held = make_held(&[PitchClass::E, PitchClass::G, PitchClass::C]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "C");
}

// Unit: C major 2nd inversion (G-C-E) → still "C"
#[test]
fn chord_second_inversion_c_major() {
    let held = make_held(&[PitchClass::G, PitchClass::C, PitchClass::E]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.name, "C");
}

// Unit: vi in C major → roman numeral "vi", is_diatonic true
#[test]
fn chord_am_in_c_major_annotated() {
    let key = Key { root: PitchClass::C, mode: Mode::Major };
    let held = make_held(&[PitchClass::A, PitchClass::C, PitchClass::E]);
    let result = recognize_chord(&held, Some(key)).unwrap();
    assert_eq!(result.name, "Am");
    assert_eq!(result.roman_numeral, Some("vi".to_string()));
    assert_eq!(result.is_diatonic, Some(true));
}

// Unit: I in C major → roman numeral "I", is_diatonic true
#[test]
fn chord_c_in_c_major_annotated() {
    let key = Key { root: PitchClass::C, mode: Mode::Major };
    let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G]);
    let result = recognize_chord(&held, Some(key)).unwrap();
    assert_eq!(result.roman_numeral, Some("I".to_string()));
    assert_eq!(result.is_diatonic, Some(true));
}

// Unit: F# major in C major → not diatonic (F# is not in C major scale)
#[test]
fn chord_f_sharp_not_diatonic_in_c_major() {
    let key = Key { root: PitchClass::C, mode: Mode::Major };
    // Gb major = F# major: Gb-Bb-Db
    let held = make_held(&[PitchClass::Gb, PitchClass::Bb, PitchClass::Db]);
    let result = recognize_chord(&held, Some(key)).unwrap();
    assert_eq!(result.is_diatonic, Some(false));
}

// Unit: no key selected → roman_numeral and is_diatonic are None
#[test]
fn chord_no_key_no_annotation() {
    let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G]);
    let result = recognize_chord(&held, None).unwrap();
    assert_eq!(result.roman_numeral, None);
    assert_eq!(result.is_diatonic, None);
}

// Unit: vii° in C major → diminished, is_diatonic true
#[test]
fn chord_b_dim_in_c_major_annotated() {
    let key = Key { root: PitchClass::C, mode: Mode::Major };
    let held = make_held(&[PitchClass::B, PitchClass::D, PitchClass::F]);
    let result = recognize_chord(&held, Some(key)).unwrap();
    assert_eq!(result.name, "Bdim");
    assert_eq!(result.roman_numeral, Some("vii°".to_string()));
    assert_eq!(result.is_diatonic, Some(true));
}

// ─────────────────────────── Property Tests (Task 3.1) ────────────────────────

#[cfg(test)]
mod property_tests {
    use super::*;
    use crate::music_theory::{diatonic_chords, scale_notes};
    use proptest::prelude::*;
    use std::collections::HashSet;

    fn any_pitch_class() -> impl Strategy<Value = PitchClass> {
        (0u8..12).prop_map(PitchClass::from_index)
    }

    fn any_key() -> impl Strategy<Value = Key> {
        (any_pitch_class(), prop_oneof![Just(Mode::Major), Just(Mode::Minor)])
            .prop_map(|(root, mode)| Key { root, mode })
    }

    fn held_from_set(pcs: &HashSet<PitchClass>) -> Vec<HeldNote> {
        pcs.iter()
            .map(|&pc| HeldNote { midi_note: pc.to_index() + 60, velocity: 64, pitch_class: pc, octave: 4 })
            .collect()
    }

    // Non-augmented chord types: (suffix, intervals) — deterministic root recognition
    static NON_AMBIGUOUS_CHORDS: &[(&str, &[u8])] = &[
        ("",     &[0, 4, 7]),
        ("m",    &[0, 3, 7]),
        ("maj7", &[0, 4, 7, 11]),
        ("m7",   &[0, 3, 7, 10]),
        ("7",    &[0, 4, 7, 10]),
        ("m7b5", &[0, 3, 6, 10]),
    ];

    // Feature: midi-keyboard-integration, Property 2: MIDI note to PitchClass/Octave derivation
    proptest! {
        #[test]
        fn prop_midi_note_pitch_class_and_octave(
            note in 0u8..=127u8,
            vel in 1u8..=127u8,
        ) {
            let held = HeldNote::from_midi(note, vel);
            prop_assert_eq!(held.pitch_class, PitchClass::from_index(note % 12));
            prop_assert_eq!(held.octave, (note / 12) as i8 - 1);
        }
    }

    // Feature: midi-keyboard-integration, Property 3: Velocity opacity is monotonically increasing
    proptest! {
        #[test]
        fn prop_velocity_opacity_monotone(
            v1 in 1u8..127u8,  // 1..127 so v2 = v1+1 stays in 1..=127
        ) {
            let v2 = v1 + 1;
            let op1 = HeldNote::from_midi(60, v1).velocity_opacity();
            let op2 = HeldNote::from_midi(60, v2).velocity_opacity();
            prop_assert!(op1 < op2, "opacity({}) = {} should be < opacity({}) = {}", v1, op1, v2, op2);
        }
    }

    // Feature: midi-keyboard-integration, Property 3: Velocity opacity boundary values
    proptest! {
        #[test]
        fn prop_velocity_opacity_boundaries(note in 0u8..=127u8) {
            let op_min = HeldNote::from_midi(note, 1).velocity_opacity();
            let op_max = HeldNote::from_midi(note, 127).velocity_opacity();
            prop_assert!((op_min - 0.35_f32).abs() < 1e-5, "opacity(1) should be 0.35, got {}", op_min);
            prop_assert!((op_max - 1.0_f32).abs() < 1e-5, "opacity(127) should be 1.0, got {}", op_max);
        }
    }

    // Feature: midi-keyboard-integration, Property 5: Chord recognition requires 3+ distinct PitchClasses
    proptest! {
        #[test]
        fn prop_fewer_than_3_pcs_returns_none(
            pcs in proptest::collection::hash_set(any_pitch_class(), 0usize..3usize)
        ) {
            let held = held_from_set(&pcs);
            prop_assert!(recognize_chord(&held, None).is_none());
        }
    }

    // Feature: midi-keyboard-integration, Property 6: Known chords recognized in all inversions
    proptest! {
        #[test]
        fn prop_known_chords_recognized(
            root in any_pitch_class(),
            chord_idx in 0usize..6usize,   // index into NON_AMBIGUOUS_CHORDS
        ) {
            let (suffix, intervals) = NON_AMBIGUOUS_CHORDS[chord_idx];
            let pcs: HashSet<PitchClass> = intervals.iter()
                .map(|&i| root.add_semitones(i))
                .collect();

            // Distinct PC count must be >= 3 (always true for our chord list, but guard anyway)
            if pcs.len() < 3 {
                return Ok(());
            }

            let held = held_from_set(&pcs);
            let result = recognize_chord(&held, None);
            prop_assert!(result.is_some(), "Expected chord {}{}  to be recognized", root.name(), suffix);

            let chord = result.unwrap();
            let expected = format!("{}{}", root.name(), suffix);
            prop_assert_eq!(&chord.name, &expected,
                "Wrong name: got {} expected {}", chord.name, expected);
        }
    }

    // Feature: midi-keyboard-integration, Property 6 (inversions):
    // Same pitch-class set presented in a different "order" (held note order) still recognized.
    proptest! {
        #[test]
        fn prop_chord_recognized_regardless_of_hold_order(
            root in any_pitch_class(),
            chord_idx in 0usize..6usize,
        ) {
            let (suffix, intervals) = NON_AMBIGUOUS_CHORDS[chord_idx];
            let pcs: Vec<PitchClass> = intervals.iter()
                .map(|&i| root.add_semitones(i))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect();

            if pcs.len() < 3 { return Ok(()); }

            // Build held notes in each cyclic "inversion" (rotate the note list)
            let expected_name = format!("{}{}", root.name(), suffix);
            for rotation in 0..pcs.len() {
                let rotated: Vec<PitchClass> = pcs[rotation..].iter()
                    .chain(pcs[..rotation].iter())
                    .cloned()
                    .collect();
                let held: Vec<HeldNote> = rotated.iter()
                    .map(|&pc| HeldNote { midi_note: pc.to_index() + 60, velocity: 64, pitch_class: pc, octave: 4 })
                    .collect();
                let result = recognize_chord(&held, None);
                prop_assert!(result.is_some());
                prop_assert_eq!(&result.unwrap().name, &expected_name);
            }
        }
    }

    // Feature: midi-keyboard-integration, Property 7: Chord-in-key annotation correctness
    proptest! {
        #[test]
        fn prop_is_diatonic_always_some_when_key_selected(
            root in any_pitch_class(),
            key in any_key(),
        ) {
            // Use a major triad rooted at `root`
            let pcs: HashSet<PitchClass> = [0u8, 4, 7].iter()
                .map(|&i| root.add_semitones(i))
                .collect();
            let held = held_from_set(&pcs);
            let result = recognize_chord(&held, Some(key));
            prop_assert!(result.is_some());
            // is_diatonic must be Some when key is provided
            prop_assert!(result.unwrap().is_diatonic.is_some());
        }
    }

    // Feature: midi-keyboard-integration, Property 7: is_diatonic true iff root+quality in diatonic chords
    proptest! {
        #[test]
        fn prop_is_diatonic_correct(
            key in any_key(),
            degree_idx in 0usize..7usize,
        ) {
            let chords = diatonic_chords(key);
            let diatonic_chord = &chords[degree_idx];

            // Build held notes from the diatonic chord's notes
            let pcs: HashSet<PitchClass> = diatonic_chord.notes.iter().cloned().collect();
            let held = held_from_set(&pcs);

            let result = recognize_chord(&held, Some(key));
            prop_assert!(result.is_some());
            let recognized = result.unwrap();

            // A diatonic triad must be recognized as diatonic
            prop_assert_eq!(recognized.is_diatonic, Some(true),
                "Diatonic chord {} should have is_diatonic=Some(true) in key {:?}",
                recognized.name, key);

            // Roman numeral must be Some for diatonic chords
            prop_assert!(recognized.roman_numeral.is_some(),
                "Diatonic chord should have a roman numeral in key {:?}", key);
        }
    }

    // Feature: midi-keyboard-integration, Property 7: non-diatonic root → is_diatonic Some(false)
    proptest! {
        #[test]
        fn prop_non_scale_root_not_diatonic(
            key in any_key(),
            root in any_pitch_class(),
        ) {
            let notes = scale_notes(key);
            // Only test roots NOT in the scale
            if notes.contains(&root) { return Ok(()); }

            let pcs: HashSet<PitchClass> = [0u8, 4, 7].iter()
                .map(|&i| root.add_semitones(i))
                .collect();
            let held = held_from_set(&pcs);

            let result = recognize_chord(&held, Some(key));
            prop_assert!(result.is_some());
            prop_assert_eq!(result.unwrap().is_diatonic, Some(false),
                "Root {:?} is not in scale of {:?}, should be non-diatonic", root, key);
        }
    }
}
