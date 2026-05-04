use std::collections::HashSet;
use gloo_timers::callback::Timeout;
use yew::prelude::*;

use crate::midi::HeldNote;
use crate::music_theory::{chord_name, diatonic_chords, roman_numeral, PitchClass, Progression};

// ─────────────────────────── Pure logic ──────────────────────────────────────

/// Returns `true` when every pitch class in `target` is present in at least one
/// of the `held` notes (octave-agnostic).
///
/// An empty target is vacuously true (all zero elements are satisfied).
pub fn chord_fully_held(target: &[PitchClass], held: &[HeldNote]) -> bool {
    let held_pcs: Vec<PitchClass> = held.iter().map(|n| n.pitch_class).collect();
    target.iter().all(|pc| held_pcs.contains(pc))
}

// ─────────────────────────── Props ───────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct PlayAlongPanelProps {
    pub progression: Progression,
    pub current_chord_index: usize,
    pub chords_played: u32,
    pub showing_loop_cue: bool,
    pub held_notes: Vec<HeldNote>,
    pub on_stop: Callback<()>,
    pub on_chord_correct: Callback<()>,
    pub on_loop_cue_done: Callback<()>,
}

// ─────────────────────────── Component ───────────────────────────────────────

#[function_component(PlayAlongPanel)]
pub fn play_along_panel(props: &PlayAlongPanelProps) -> Html {
    let chord_count = props.progression.chords.len();
    let chords = diatonic_chords(props.progression.key);

    // Derive target PitchClasses from current chord index
    let target_pcs: Vec<PitchClass> = props.progression.chords
        .get(props.current_chord_index)
        .and_then(|&degree| chords.iter().find(|c| c.degree == degree))
        .map(|c| c.notes.to_vec())
        .unwrap_or_default();

    // ── 300ms debounce for chord detection ───────────────────────────────────
    {
        let on_chord_correct = props.on_chord_correct.clone();
        let target_pcs = target_pcs.clone();
        let held_notes = props.held_notes.clone();
        let current_chord_index = props.current_chord_index;

        use_effect_with((held_notes, current_chord_index), move |(held_notes, _idx)| {
            let timeout = if chord_fully_held(&target_pcs, held_notes) {
                let cb = on_chord_correct.clone();
                Some(Timeout::new(300, move || cb.emit(())))
            } else {
                None
            };
            move || drop(timeout)
        });
    }

    // ── 1.5s loop cue auto-clear ─────────────────────────────────────────────
    {
        let on_loop_cue_done = props.on_loop_cue_done.clone();
        let showing_loop_cue = props.showing_loop_cue;

        use_effect_with(showing_loop_cue, move |&showing| {
            let timeout = if showing {
                let cb = on_loop_cue_done.clone();
                Some(Timeout::new(1500, move || cb.emit(())))
            } else {
                None
            };
            move || drop(timeout)
        });
    }

    // ── Current chord label ───────────────────────────────────────────────────
    let current_chord = props.progression.chords
        .get(props.current_chord_index)
        .and_then(|&degree| chords.iter().find(|c| c.degree == degree));
    let chord_label = current_chord
        .map(|c| format!("{} ({})", chord_name(c.root, c.quality), roman_numeral(c.degree, c.quality)))
        .unwrap_or_default();

    let held_pcs: HashSet<PitchClass> =
        props.held_notes.iter().map(|n| n.pitch_class).collect();

    let notes_html: Html = target_pcs.iter().map(|pc| {
        let held = held_pcs.contains(pc);
        let class = if held { "play-along__note play-along__note--held" } else { "play-along__note" };
        html! { <span class={class}>{pc.name()}</span> }
    }).collect();

    let on_stop = props.on_stop.reform(|_: MouseEvent| ());

    html! {
        <div class="play-along-panel">
            <h2 class="play-along__title">{"Play Along"}</h2>

            if props.showing_loop_cue {
                <div class="play-along__loop-cue">{"↺ Loop!"}</div>
            }

            <div class="play-along__progress">
                {format!("Chord {} of {}", props.current_chord_index + 1, chord_count)}
            </div>

            <div class="play-along__target">
                <span class="play-along__chord-label">{chord_label}</span>
                <div class="play-along__notes">{notes_html}</div>
            </div>

            <button class="play-along__stop-btn" onclick={on_stop}>{"Stop"}</button>
        </div>
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::midi::HeldNote;
    use crate::music_theory::PitchClass;

    fn make_held(pitch_class: PitchClass, octave: i8) -> HeldNote {
        // midi_note is computed from octave: note = (octave + 1) * 12 + pc_index
        let midi_note = ((octave + 1) as u8) * 12 + pitch_class.to_index();
        HeldNote { midi_note, velocity: 64, pitch_class, octave }
    }

    // Feature: play-along-redesign, Property 5: empty target always returns true
    #[test]
    fn chord_fully_held_empty_target_returns_true() {
        let held = vec![make_held(PitchClass::C, 4)];
        assert!(chord_fully_held(&[], &held));
    }

    #[test]
    fn chord_fully_held_empty_target_no_held_returns_true() {
        assert!(chord_fully_held(&[], &[]));
    }

    #[test]
    fn chord_fully_held_all_target_pcs_present_returns_true() {
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        let held = vec![
            make_held(PitchClass::C, 4),
            make_held(PitchClass::E, 4),
            make_held(PitchClass::G, 4),
        ];
        assert!(chord_fully_held(&target, &held));
    }

    #[test]
    fn chord_fully_held_one_target_pc_missing_returns_false() {
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        let held = vec![
            make_held(PitchClass::C, 4),
            make_held(PitchClass::E, 4),
        ];
        assert!(!chord_fully_held(&target, &held));
    }

    #[test]
    fn chord_fully_held_non_empty_target_empty_held_returns_false() {
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        assert!(!chord_fully_held(&target, &[]));
    }

    #[test]
    fn chord_fully_held_octave_agnostic() {
        // Target notes played in different octaves than expected — still counts
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        let held = vec![
            make_held(PitchClass::C, 3),  // lower octave
            make_held(PitchClass::E, 5),  // higher octave
            make_held(PitchClass::G, 2),  // yet another octave
        ];
        assert!(chord_fully_held(&target, &held));
    }

    #[test]
    fn chord_fully_held_extra_notes_do_not_prevent_true() {
        // Holding extra notes beyond the target is fine
        let target = vec![PitchClass::C, PitchClass::E, PitchClass::G];
        let held = vec![
            make_held(PitchClass::C, 4),
            make_held(PitchClass::E, 4),
            make_held(PitchClass::G, 4),
            make_held(PitchClass::Bb, 4), // extra note
        ];
        assert!(chord_fully_held(&target, &held));
    }

    // Feature: play-along-redesign, Property 1: for any target PCs present in held (any octave), returns true
    mod proptest_chord_fully_held {
        use super::*;
        use proptest::prelude::*;

        fn arb_pitch_class() -> impl Strategy<Value = PitchClass> {
            (0u8..12).prop_map(PitchClass::from_index)
        }

        // Property 1: if all target PCs appear in held (possibly different octave), result is true
        proptest! {
            #[test]
            fn prop_all_targets_present_returns_true(
                target_indices in prop::collection::vec(0u8..12, 1..4),
                octave_offsets in prop::collection::vec(-2i8..=4i8, 1..4),
            ) {
                let target: Vec<PitchClass> = target_indices.iter()
                    .map(|&i| PitchClass::from_index(i))
                    .collect();

                // Build held notes: one per target PC (possibly in a different octave)
                let held: Vec<HeldNote> = target.iter().enumerate().map(|(i, &pc)| {
                    let oct = octave_offsets[i % octave_offsets.len()].clamp(0, 5);
                    make_held(pc, oct)
                }).collect();

                prop_assert!(chord_fully_held(&target, &held),
                    "Expected true when all target PCs are in held");
            }
        }

        // Property 5: empty target always returns true for any held notes
        proptest! {
            #[test]
            fn prop_empty_target_always_true(
                held_indices in prop::collection::vec((0u8..12, 3i8..6i8), 0..6),
            ) {
                let held: Vec<HeldNote> = held_indices.iter()
                    .map(|&(i, oct)| make_held(PitchClass::from_index(i), oct))
                    .collect();
                prop_assert!(chord_fully_held(&[], &held));
            }
        }

        // Complementary: missing target PC always returns false
        proptest! {
            #[test]
            fn prop_missing_target_pc_returns_false(
                root_idx in arb_pitch_class(),
            ) {
                // target = [root, third, fifth] but we only hold root
                let third = root_idx.add_semitones(4);
                let fifth = root_idx.add_semitones(7);
                let target = vec![root_idx, third, fifth];
                // Only hold root — third and fifth are absent
                let held = vec![make_held(root_idx, 4)];
                prop_assert!(!chord_fully_held(&target, &held));
            }
        }
    }
}
