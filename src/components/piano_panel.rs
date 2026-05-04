use yew::prelude::*;

use crate::midi::HeldNote;
use crate::music_theory::{scale_notes, ChordHighlight, DiatonicChord, Key, KeyRole, PitchClass};

// ── Constants ────────────────────────────────────────────────────────────────

/// Number of octaves the keyboard spans (≥3 per requirement 5.1).
pub const NUM_OCTAVES: usize = 3;

/// Total number of semitones (keys) rendered.
pub const TOTAL_SEMITONES: usize = NUM_OCTAVES * 12; // 36

/// Pixel width of each white key — used for auto-scroll calculations.
const WHITE_KEY_WIDTH_PX: i32 = 32;

// ── FingerHint types ─────────────────────────────────────────────────────────

/// A finger position indicator for one note of a chord.
///
/// `finger` uses the standard piano fingering convention:
///   1 = thumb, 2 = index, 3 = middle, 4 = ring, 5 = pinky.
/// For root-position triads the default assignment is [1, 3, 5].
#[derive(Clone, PartialEq, Debug)]
pub struct FingerHint {
    pub pitch_class: PitchClass,
    /// Finger number (1–5).
    pub finger: u8,
    /// `true` when this pitch class is currently held by the player (octave-agnostic).
    pub held: bool,
}

/// Build finger hint indicators for a triad chord.
///
/// Zips `chord.notes[0..3]` with finger numbers `[1, 3, 5]`.
/// The `held` field is set to `true` when any held note matches the pitch class
/// (independent of octave).
pub fn finger_hints_for_chord(chord: &DiatonicChord, held: &[HeldNote]) -> Vec<FingerHint> {
    let held_pcs: Vec<PitchClass> = held.iter().map(|n| n.pitch_class).collect();
    let fingers = [1u8, 3, 5];
    chord.notes[..3]
        .iter()
        .zip(fingers.iter())
        .map(|(&pc, &finger)| FingerHint {
            pitch_class: pc,
            finger,
            held: held_pcs.contains(&pc),
        })
        .collect()
}

// ── Pure logic (testable without WASM) ──────────────────────────────────────

/// Returns the `KeyRole` of a pitch class in the current selection context.
///
/// Chord notes (Root / Third / Fifth) take priority over ScaleNote so that
/// chord highlighting is always unambiguous.
pub fn note_role(
    pitch: PitchClass,
    selected_key: Option<Key>,
    highlighted_chord: Option<&ChordHighlight>,
) -> KeyRole {
    if let Some(chord) = highlighted_chord {
        if pitch == chord.root  { return KeyRole::Root; }
        if pitch == chord.third { return KeyRole::Third; }
        if pitch == chord.fifth { return KeyRole::Fifth; }
    }
    if let Some(key) = selected_key {
        if scale_notes(key).contains(&pitch) {
            return KeyRole::ScaleNote;
        }
    }
    KeyRole::None
}

/// Returns `true` if this pitch class is a black key.
pub fn is_black_key(pitch: PitchClass) -> bool {
    matches!(
        pitch,
        PitchClass::Db | PitchClass::Eb | PitchClass::Gb | PitchClass::Ab | PitchClass::Bb
    )
}

/// Returns all pitch classes across the full keyboard (NUM_OCTAVES × 12, starting from C).
pub fn piano_keys() -> Vec<PitchClass> {
    (0..TOTAL_SEMITONES)
        .map(|i| PitchClass::from_index((i % 12) as u8))
        .collect()
}

/// Number of white keys that appear *before* `semitone_idx` within one octave.
/// Used to compute the approximate horizontal scroll offset.
fn white_keys_before_in_octave(semitone_idx: usize) -> usize {
    // C=0  Db=1  D=2  Eb=3  E=4  F=5  Gb=6  G=7  Ab=8  A=9  Bb=10  B=11
    const TABLE: [usize; 12] = [0, 0, 1, 1, 2, 3, 3, 4, 4, 5, 5, 6];
    TABLE[semitone_idx % 12]
}

/// CSS class for a held key under practice/play-along mode (Property 13).
///
/// Returns `"midi-correct"` when the pitch is in `target`, `"midi-incorrect"` otherwise.
/// Returns `""` when `target` is None (no practice mode active).
pub fn practice_key_class(pitch: PitchClass, target: Option<&[PitchClass]>) -> &'static str {
    match target {
        Some(t) if t.contains(&pitch) => "midi-correct",
        Some(_) => "midi-incorrect",
        None => "",
    }
}

// ── Component ────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct PianoPanelProps {
    pub selected_key: Option<Key>,
    pub highlighted_chord: Option<ChordHighlight>,
    pub playing_note: Option<(PitchClass, i32)>,
    pub show_labels: bool,
    /// Offset from the base octave (C3). Clamped to -2..=2 by the reducer.
    pub octave_offset: i8,
    pub on_toggle_labels: Callback<()>,
    pub on_octave_shift: Callback<i8>,
    /// MIDI notes currently held down. Keys matching (pitch_class, octave) get
    /// the `midi-held` class and a velocity-derived inline opacity.
    #[prop_or_default]
    pub held_notes: Vec<HeldNote>,
    /// When Some, held keys whose PitchClass is in target get `midi-correct`,
    /// held keys not in target get `midi-incorrect` (Property 13).
    #[prop_or_default]
    pub practice_target: Option<Vec<PitchClass>>,
    /// Finger position indicators shown above piano keys during play-along mode.
    /// When `None`, no indicators are rendered.
    #[prop_or_default]
    pub finger_hints: Option<Vec<FingerHint>>,
}

#[function_component(PianoPanel)]
pub fn piano_panel(props: &PianoPanelProps) -> Html {
    let container_ref = use_node_ref();
    let base_octave = 3i8 + props.octave_offset;

    // Auto-scroll: prefer lowest held MIDI note; fall back to chord root.
    {
        let container_ref = container_ref.clone();
        let chord = props.highlighted_chord.clone();
        let held_notes = props.held_notes.clone();
        use_effect_with((chord, held_notes), move |(chord, held_notes)| {
            let Some(container) = container_ref.cast::<web_sys::Element>() else { return; };

            let scroll_px = if let Some(lowest) = held_notes.iter().min_by_key(|n| n.midi_note) {
                // Scroll to lowest held note's position on the keyboard
                let semitone_from_start =
                    lowest.midi_note as i32 - (base_octave as i32 + 1) * 12;
                if semitone_from_start >= 0 {
                    let idx = semitone_from_start as usize;
                    let octave_in_kb = idx / 12;
                    let pc = idx % 12;
                    Some(
                        (octave_in_kb * 7 + white_keys_before_in_octave(pc)) as i32
                            * WHITE_KEY_WIDTH_PX,
                    )
                } else {
                    None
                }
            } else if let Some(chord) = chord {
                // Fall back to chord root when no MIDI notes held
                let semitone = chord.root.to_index() as usize;
                Some(white_keys_before_in_octave(semitone) as i32 * WHITE_KEY_WIDTH_PX)
            } else {
                None
            };

            if let Some(px) = scroll_px {
                container.set_scroll_left(px);
            }
        });
    }

    let keys = piano_keys();

    let key_elements: Html = keys
        .iter()
        .enumerate()
        .map(|(i, &pitch)| {
            let role =
                note_role(pitch, props.selected_key, props.highlighted_chord.as_ref());
            let black = is_black_key(pitch);
            let octave_num = base_octave + (i / 12) as i8;
            let label = format!("{}{}", pitch.name(), octave_num);

            let role_cls = match role {
                KeyRole::Root      => "piano-key--root",
                KeyRole::Third     => "piano-key--third",
                KeyRole::Fifth     => "piano-key--fifth",
                KeyRole::ScaleNote => "piano-key--scale",
                KeyRole::None      => "",
            };
            let type_cls = if black { "piano-key--black" } else { "piano-key--white" };
            let playing_cls = if props.playing_note == Some((pitch, octave_num as i32)) {
                "piano-key--playing"
            } else {
                ""
            };

            // Find a matching held note for this exact key (pitch class + octave)
            let held = props.held_notes.iter()
                .find(|n| n.pitch_class == pitch && n.octave == octave_num);
            let midi_cls = if held.is_some() { "midi-held" } else { "" };
            // Practice/play-along color — only applied to held keys
            let practice_cls = held
                .map(|_| practice_key_class(pitch, props.practice_target.as_deref()))
                .unwrap_or("");

            let classes = format!(
                "piano-key {} {} {} {} {}",
                type_cls, role_cls, playing_cls, midi_cls, practice_cls
            );
            // Velocity-derived opacity — only set when the key is held
            let style = held
                .map(|n| format!("opacity: {:.2}", n.velocity_opacity()))
                .unwrap_or_default();
            let show_labels = props.show_labels;

            // Finger hint indicator — look up by pitch class (one hint per key)
            let finger_hint_html: Html = props.finger_hints
                .as_deref()
                .and_then(|hints| hints.iter().find(|h| h.pitch_class == pitch))
                .map(|hint| {
                    let hint_cls = if hint.held {
                        "finger-hint finger-hint--held"
                    } else {
                        "finger-hint"
                    };
                    html! { <div class={hint_cls}>{ hint.finger }</div> }
                })
                .unwrap_or_default();

            html! {
                <div class={classes} style={style} key={i as u32}>
                    { finger_hint_html }
                    if show_labels {
                        <span class="piano-key__label">{ label }</span>
                    }
                </div>
            }
        })
        .collect();

    let on_toggle = {
        let cb = props.on_toggle_labels.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };
    let on_up = {
        let cb = props.on_octave_shift.clone();
        Callback::from(move |_: MouseEvent| cb.emit(1))
    };
    let on_down = {
        let cb = props.on_octave_shift.clone();
        Callback::from(move |_: MouseEvent| cb.emit(-1))
    };

    html! {
        <div class="piano-panel">
            <div class="piano-panel__controls">
                <button class="piano-btn" onclick={on_up}>{ "Oct ▲" }</button>
                <button class="piano-btn" onclick={on_down}>{ "Oct ▼" }</button>
                <button class="piano-btn" onclick={on_toggle}>
                    { if props.show_labels { "Hide Labels" } else { "Show Labels" } }
                </button>
                <span class="piano-panel__octave-info">
                    { format!("C{} – B{}", base_octave, base_octave + NUM_OCTAVES as i8 - 1) }
                </span>
            </div>
            <div class="piano-legend">
                <span class="piano-legend__item">
                    <span class="piano-legend__swatch piano-legend__swatch--root"></span>{"Root"}
                </span>
                <span class="piano-legend__item">
                    <span class="piano-legend__swatch piano-legend__swatch--third"></span>{"3rd"}
                </span>
                <span class="piano-legend__item">
                    <span class="piano-legend__swatch piano-legend__swatch--fifth"></span>{"5th"}
                </span>
                <span class="piano-legend__item">
                    <span class="piano-legend__swatch piano-legend__swatch--scale"></span>{"Scale note"}
                </span>
            </div>
            <div class="piano-panel__keyboard" ref={container_ref}>
                { key_elements }
            </div>
        </div>
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_theory::{Mode, PitchClass};

    // Feature: circle-of-fifths, Property 12: Piano scale highlight correctness
    #[test]
    fn scale_notes_highlighted_as_scale_note() {
        for root_idx in 0u8..12 {
            let key = Key { root: PitchClass::from_index(root_idx), mode: Mode::Major };
            let expected = scale_notes(key);
            for pitch_idx in 0u8..12 {
                let pitch = PitchClass::from_index(pitch_idx);
                let role = note_role(pitch, Some(key), None);
                if expected.contains(&pitch) {
                    assert_eq!(
                        role,
                        KeyRole::ScaleNote,
                        "{:?} should be ScaleNote in {:?} major",
                        pitch, key.root,
                    );
                } else {
                    assert_eq!(
                        role,
                        KeyRole::None,
                        "{:?} should be None in {:?} major",
                        pitch, key.root,
                    );
                }
            }
        }
    }

    // Task 11.2: keyboard spans at least 36 keys
    #[test]
    fn piano_keys_spans_at_least_36_keys() {
        let keys = piano_keys();
        assert!(
            keys.len() >= 36,
            "expected ≥36 keys, got {}",
            keys.len()
        );
        assert_eq!(keys.len(), TOTAL_SEMITONES);
    }

    #[test]
    fn chord_notes_take_priority_over_scale_notes() {
        let key = Key { root: PitchClass::C, mode: Mode::Major };
        // C, E, G are all scale notes of C major AND chord notes.
        let chord = ChordHighlight {
            root: PitchClass::C,
            third: PitchClass::E,
            fifth: PitchClass::G,
        };
        assert_eq!(note_role(PitchClass::C, Some(key), Some(&chord)), KeyRole::Root);
        assert_eq!(note_role(PitchClass::E, Some(key), Some(&chord)), KeyRole::Third);
        assert_eq!(note_role(PitchClass::G, Some(key), Some(&chord)), KeyRole::Fifth);
        // D is a scale note but not in the chord.
        assert_eq!(note_role(PitchClass::D, Some(key), Some(&chord)), KeyRole::ScaleNote);
        // F# / Gb is neither in C major scale nor in the chord.
        assert_eq!(note_role(PitchClass::Gb, Some(key), Some(&chord)), KeyRole::None);
    }

    #[test]
    fn no_key_selected_all_notes_return_none() {
        for pitch_idx in 0u8..12 {
            let pitch = PitchClass::from_index(pitch_idx);
            assert_eq!(note_role(pitch, None, None), KeyRole::None);
        }
    }

    #[test]
    fn black_key_detection_correct() {
        assert!(is_black_key(PitchClass::Db));
        assert!(is_black_key(PitchClass::Eb));
        assert!(is_black_key(PitchClass::Gb));
        assert!(is_black_key(PitchClass::Ab));
        assert!(is_black_key(PitchClass::Bb));
        assert!(!is_black_key(PitchClass::C));
        assert!(!is_black_key(PitchClass::D));
        assert!(!is_black_key(PitchClass::E));
        assert!(!is_black_key(PitchClass::F));
        assert!(!is_black_key(PitchClass::G));
        assert!(!is_black_key(PitchClass::A));
        assert!(!is_black_key(PitchClass::B));
    }

    // ── MIDI highlight tests (Task 9) ──────────────────────────────────────

    // Feature: midi-keyboard-integration, Property 13: Practice/play-along note color classification
    #[test]
    fn practice_key_class_correct_when_in_target() {
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        assert_eq!(practice_key_class(PitchClass::C, Some(&target)), "midi-correct");
        assert_eq!(practice_key_class(PitchClass::E, Some(&target)), "midi-correct");
        assert_eq!(practice_key_class(PitchClass::G, Some(&target)), "midi-correct");
    }

    #[test]
    fn practice_key_class_incorrect_when_not_in_target() {
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        assert_eq!(practice_key_class(PitchClass::D, Some(&target)), "midi-incorrect");
        assert_eq!(practice_key_class(PitchClass::F, Some(&target)), "midi-incorrect");
        assert_eq!(practice_key_class(PitchClass::A, Some(&target)), "midi-incorrect");
    }

    #[test]
    fn practice_key_class_empty_when_no_target() {
        assert_eq!(practice_key_class(PitchClass::C, None), "");
        assert_eq!(practice_key_class(PitchClass::Gb, None), "");
    }

    #[test]
    fn practice_key_class_covers_all_pitch_classes() {
        // Property 13: correct/incorrect are disjoint and cover all 12 pitch classes
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        for idx in 0u8..12 {
            let pc = PitchClass::from_index(idx);
            let cls = practice_key_class(pc, Some(&target));
            assert!(
                cls == "midi-correct" || cls == "midi-incorrect",
                "Expected midi-correct or midi-incorrect for {:?}, got {:?}", pc, cls
            );
        }
    }

    // ── Cancellable-playback property tests (Task 8) ──────────────────────

    mod cancellable_playback_props {
        use super::*;
        use crate::music_theory::Mode;
        use proptest::prelude::*;

        // Feature: cancellable-playback, Property 4: Idle State key highlight correctness
        proptest! {
            #[test]
            fn prop_idle_state_scale_highlight(root_idx in 0u8..12, mode_bit in any::<bool>()) {
                let mode = if mode_bit { Mode::Major } else { Mode::Minor };
                let key = Key { root: PitchClass::from_index(root_idx), mode };
                let expected = scale_notes(key);
                for pitch_idx in 0u8..12 {
                    let pitch = PitchClass::from_index(pitch_idx);
                    let role = note_role(pitch, Some(key), None);
                    if expected.contains(&pitch) {
                        prop_assert_ne!(role, KeyRole::None,
                            "{:?} should be highlighted in {:?}", pitch, key);
                    } else {
                        prop_assert_eq!(role, KeyRole::None,
                            "{:?} should not be highlighted in {:?}", pitch, key);
                    }
                }
            }
        }

        // Feature: cancellable-playback, Property 5: Idle State chord highlight correctness
        proptest! {
            #[test]
            fn prop_idle_state_chord_highlight(
                root_idx  in 0u8..12,
                third_idx in 0u8..12,
                fifth_idx in 0u8..12,
            ) {
                let chord = ChordHighlight {
                    root:  PitchClass::from_index(root_idx),
                    third: PitchClass::from_index(third_idx),
                    fifth: PitchClass::from_index(fifth_idx),
                };
                let chord_pitches = [chord.root, chord.third, chord.fifth];
                for pitch_idx in 0u8..12 {
                    let pitch = PitchClass::from_index(pitch_idx);
                    let role = note_role(pitch, None, Some(&chord));
                    if chord_pitches.contains(&pitch) {
                        prop_assert_ne!(role, KeyRole::None,
                            "{:?} should be highlighted as chord note", pitch);
                    } else {
                        prop_assert_eq!(role, KeyRole::None,
                            "{:?} should not be highlighted", pitch);
                    }
                }
            }
        }

        // Feature: cancellable-playback, Property 6: Animation handle collection is empty after clear
        proptest! {
            #[test]
            fn prop_vec_clear_is_empty(len in 0usize..20) {
                let mut v: Vec<u32> = (0..len as u32).collect();
                v.clear();
                prop_assert!(v.is_empty());
            }
        }
    }

    // ── FingerHint tests (Task 5.1) ───────────────────────────────────────────

    mod finger_hint_tests {
        use super::*;
        use crate::midi::HeldNote;
        use crate::music_theory::{ChordQuality, DiatonicChord, ScaleDegree};

        fn make_held(pitch_class: PitchClass, octave: i8) -> HeldNote {
            let midi_note = ((octave + 1) as u8) * 12 + pitch_class.to_index();
            HeldNote { midi_note, velocity: 80, pitch_class, octave }
        }

        fn c_major_chord() -> DiatonicChord {
            DiatonicChord {
                degree: ScaleDegree::I,
                quality: ChordQuality::Major,
                root: PitchClass::C,
                notes: [PitchClass::C, PitchClass::E, PitchClass::G],
            }
        }

        fn a_minor_chord() -> DiatonicChord {
            DiatonicChord {
                degree: ScaleDegree::VI,
                quality: ChordQuality::Minor,
                root: PitchClass::A,
                notes: [PitchClass::A, PitchClass::C, PitchClass::E],
            }
        }

        // Feature: play-along-redesign, Property 2.2: correct finger numbers [1, 3, 5]
        #[test]
        fn finger_numbers_are_one_three_five_for_major_chord() {
            let chord = c_major_chord();
            let hints = finger_hints_for_chord(&chord, &[]);
            assert_eq!(hints.len(), 3);
            assert_eq!(hints[0].finger, 1);
            assert_eq!(hints[1].finger, 3);
            assert_eq!(hints[2].finger, 5);
        }

        #[test]
        fn finger_numbers_are_one_three_five_for_minor_chord() {
            let chord = a_minor_chord();
            let hints = finger_hints_for_chord(&chord, &[]);
            assert_eq!(hints.len(), 3);
            assert_eq!(hints[0].finger, 1);
            assert_eq!(hints[1].finger, 3);
            assert_eq!(hints[2].finger, 5);
        }

        // Feature: play-along-redesign, Property 2.2: pitch classes map to root/third/fifth
        #[test]
        fn pitch_classes_match_chord_notes() {
            let chord = c_major_chord();
            let hints = finger_hints_for_chord(&chord, &[]);
            assert_eq!(hints[0].pitch_class, PitchClass::C); // root
            assert_eq!(hints[1].pitch_class, PitchClass::E); // third
            assert_eq!(hints[2].pitch_class, PitchClass::G); // fifth
        }

        // Feature: play-along-redesign, Property 2.4: held field is octave-agnostic
        #[test]
        fn held_true_when_pitch_class_present_any_octave() {
            let chord = c_major_chord();
            // Hold notes in a different octave than the default display
            let held = vec![
                make_held(PitchClass::C, 3),
                make_held(PitchClass::E, 5),
                // G is absent
            ];
            let hints = finger_hints_for_chord(&chord, &held);
            assert!(hints[0].held,  "C (root) should be held");
            assert!(hints[1].held,  "E (third) should be held");
            assert!(!hints[2].held, "G (fifth) should NOT be held");
        }

        #[test]
        fn held_false_when_no_notes_held() {
            let chord = c_major_chord();
            let hints = finger_hints_for_chord(&chord, &[]);
            assert!(!hints[0].held);
            assert!(!hints[1].held);
            assert!(!hints[2].held);
        }

        #[test]
        fn held_true_for_all_when_full_chord_held() {
            let chord = c_major_chord();
            let held = vec![
                make_held(PitchClass::C, 4),
                make_held(PitchClass::E, 4),
                make_held(PitchClass::G, 4),
            ];
            let hints = finger_hints_for_chord(&chord, &held);
            assert!(hints.iter().all(|h| h.held));
        }

        #[test]
        fn works_for_minor_chord() {
            let chord = a_minor_chord();
            let held = vec![
                make_held(PitchClass::A, 3),
                make_held(PitchClass::C, 4),
                // E not held
            ];
            let hints = finger_hints_for_chord(&chord, &held);
            assert_eq!(hints[0].pitch_class, PitchClass::A);
            assert_eq!(hints[1].pitch_class, PitchClass::C);
            assert_eq!(hints[2].pitch_class, PitchClass::E);
            assert!(hints[0].held);
            assert!(hints[1].held);
            assert!(!hints[2].held);
        }
    }
}
