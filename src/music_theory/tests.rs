#[cfg(test)]
mod unit_tests {
    use super::super::*;

    #[test]
    fn test_c_major_key_signature() {
        let sig = key_signature(Key::major(PitchClass::C));
        assert_eq!(sig.sharps, 0);
        assert_eq!(sig.flats, 0);
        assert!(sig.notes.is_empty());
    }

    #[test]
    fn test_g_major_key_signature() {
        let sig = key_signature(Key::major(PitchClass::G));
        assert_eq!(sig.sharps, 1);
        assert_eq!(sig.flats, 0);
        assert_eq!(sig.notes, vec![PitchClass::Gb]); // F# stored as Gb
    }

    #[test]
    fn test_f_major_key_signature() {
        let sig = key_signature(Key::major(PitchClass::F));
        assert_eq!(sig.sharps, 0);
        assert_eq!(sig.flats, 1);
        assert_eq!(sig.notes, vec![PitchClass::Bb]);
    }

    #[test]
    fn test_d_major_key_signature() {
        let sig = key_signature(Key::major(PitchClass::D));
        assert_eq!(sig.sharps, 2);
        assert_eq!(sig.flats, 0);
        assert_eq!(sig.notes, vec![PitchClass::Gb, PitchClass::Db]); // F#, C#
    }

    #[test]
    fn test_bb_major_key_signature() {
        let sig = key_signature(Key::major(PitchClass::Bb));
        assert_eq!(sig.sharps, 0);
        assert_eq!(sig.flats, 2);
        assert_eq!(sig.notes, vec![PitchClass::Bb, PitchClass::Eb]);
    }

    #[test]
    fn test_c_major_diatonic_chords() {
        let chords = diatonic_chords(Key::major(PitchClass::C));
        // I: C major
        assert_eq!(chords[0].root, PitchClass::C);
        assert_eq!(chords[0].quality, ChordQuality::Major);
        assert_eq!(chords[0].degree, ScaleDegree::I);
        // ii: D minor
        assert_eq!(chords[1].root, PitchClass::D);
        assert_eq!(chords[1].quality, ChordQuality::Minor);
        assert_eq!(chords[1].degree, ScaleDegree::II);
        // iii: E minor
        assert_eq!(chords[2].root, PitchClass::E);
        assert_eq!(chords[2].quality, ChordQuality::Minor);
        assert_eq!(chords[2].degree, ScaleDegree::III);
        // IV: F major
        assert_eq!(chords[3].root, PitchClass::F);
        assert_eq!(chords[3].quality, ChordQuality::Major);
        assert_eq!(chords[3].degree, ScaleDegree::IV);
        // V: G major
        assert_eq!(chords[4].root, PitchClass::G);
        assert_eq!(chords[4].quality, ChordQuality::Major);
        assert_eq!(chords[4].degree, ScaleDegree::V);
        // vi: A minor
        assert_eq!(chords[5].root, PitchClass::A);
        assert_eq!(chords[5].quality, ChordQuality::Minor);
        assert_eq!(chords[5].degree, ScaleDegree::VI);
        // vii°: B diminished
        assert_eq!(chords[6].root, PitchClass::B);
        assert_eq!(chords[6].quality, ChordQuality::Diminished);
        assert_eq!(chords[6].degree, ScaleDegree::VII);
    }

    #[test]
    fn test_relative_minor_of_c_major() {
        let rel = relative_minor(Key::major(PitchClass::C));
        assert_eq!(rel.root, PitchClass::A);
        assert_eq!(rel.mode, super::super::Mode::Minor);
    }

    #[test]
    fn test_relative_major_of_a_minor() {
        let rel = relative_major(Key::minor(PitchClass::A));
        assert_eq!(rel.root, PitchClass::C);
        assert_eq!(rel.mode, super::super::Mode::Major);
    }

    #[test]
    fn test_adjacent_keys_c_major() {
        let (cw, ccw) = adjacent_keys(Key::major(PitchClass::C));
        assert_eq!(cw.root, PitchClass::G);
        assert_eq!(cw.mode, super::super::Mode::Major);
        assert_eq!(ccw.root, PitchClass::F);
        assert_eq!(ccw.mode, super::super::Mode::Major);
    }

    #[test]
    fn test_opposite_key_c_major() {
        let opp = opposite_key(Key::major(PitchClass::C));
        // C + 6 semitones = Gb (tritone)
        assert_eq!(opp.root, PitchClass::Gb);
        assert_eq!(opp.mode, super::super::Mode::Major);
    }

    #[test]
    fn test_chord_display_format() {
        let chords = diatonic_chords(Key::major(PitchClass::C));
        // vi - Am
        let display = chord_display(&chords[5]);
        assert!(display.contains("vi"), "expected 'vi' in '{}'", display);
        assert!(display.contains("Am"), "expected 'Am' in '{}'", display);
        // I - C
        let display_i = chord_display(&chords[0]);
        assert!(display_i.contains("I"), "expected 'I' in '{}'", display_i);
        assert!(display_i.contains('C'), "expected 'C' in '{}'", display_i);
        // vii° - Bdim
        let display_vii = chord_display(&chords[6]);
        assert!(display_vii.contains("vii°"), "expected 'vii°' in '{}'", display_vii);
        assert!(display_vii.contains("Bdim"), "expected 'Bdim' in '{}'", display_vii);
    }

    #[test]
    fn test_scale_notes_count() {
        let notes = scale_notes(Key::major(PitchClass::C));
        assert_eq!(notes.len(), 7);
    }

    #[test]
    fn test_c_major_scale_notes() {
        let notes = scale_notes(Key::major(PitchClass::C));
        assert_eq!(notes, [
            PitchClass::C, PitchClass::D, PitchClass::E,
            PitchClass::F, PitchClass::G, PitchClass::A,
            PitchClass::B,
        ]);
    }
}

#[cfg(test)]
mod property_tests {
    use super::super::*;
    use proptest::prelude::*;

    proptest! {
        // Feature: circle-of-fifths, Property 3: Circle geometry correctness
        #[test]
        fn test_circle_geometry(root_idx in 0u8..12u8) {
            let key = Key::major(PitchClass::from_index(root_idx));
            let sig = key_signature(key);
            // accidentals 0-7
            prop_assert!(sig.sharps <= 7 || sig.flats <= 7);
            prop_assert_eq!(sig.sharps == 0 || sig.flats == 0, true); // not both

            let (cw, ccw) = adjacent_keys(key);
            // clockwise is a perfect fifth up (+7 semitones)
            prop_assert_eq!(cw.root, key.root.add_semitones(7));
            // counterclockwise is a perfect fourth up (+5 semitones)
            prop_assert_eq!(ccw.root, key.root.add_semitones(5));

            // opposite is tritone (+6 semitones)
            let opp = opposite_key(key);
            prop_assert_eq!(opp.root, key.root.add_semitones(6));
        }
    }

    proptest! {
        // Feature: circle-of-fifths, Property 4: Diatonic chord correctness
        #[test]
        fn test_diatonic_chord_correctness(root_idx in 0u8..12u8) {
            let key = Key::major(PitchClass::from_index(root_idx));
            let chords = diatonic_chords(key);
            prop_assert_eq!(chords.len(), 7);
            prop_assert_eq!(chords[0].quality, ChordQuality::Major);     // I
            prop_assert_eq!(chords[1].quality, ChordQuality::Minor);     // ii
            prop_assert_eq!(chords[2].quality, ChordQuality::Minor);     // iii
            prop_assert_eq!(chords[3].quality, ChordQuality::Major);     // IV
            prop_assert_eq!(chords[4].quality, ChordQuality::Major);     // V
            prop_assert_eq!(chords[5].quality, ChordQuality::Minor);     // vi
            prop_assert_eq!(chords[6].quality, ChordQuality::Diminished); // vii°
            // Each chord has exactly 3 notes
            for chord in &chords {
                prop_assert_eq!(chord.notes.len(), 3);
            }
        }
    }

    proptest! {
        // Feature: circle-of-fifths, Property 5: Chord display format
        #[test]
        fn test_chord_display_format(root_idx in 0u8..12u8, degree_idx in 0u8..7u8) {
            let key = Key::major(PitchClass::from_index(root_idx));
            let chords = diatonic_chords(key);
            let chord = &chords[degree_idx as usize];
            let display = chord_display(chord);
            // Must contain both roman numeral and chord name
            let rn = roman_numeral(chord.degree, chord.quality);
            let cn = chord_name(chord.root, chord.quality);
            prop_assert!(display.contains(rn), "display '{}' missing roman numeral '{}'", display, rn);
            prop_assert!(display.contains(&cn), "display '{}' missing chord name '{}'", display, cn);
        }
    }
}
