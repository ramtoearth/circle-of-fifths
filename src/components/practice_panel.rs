use yew::prelude::*;

use crate::midi::{HeldNote, PracticeScore};
use crate::music_theory::{chord_display, chord_name, DiatonicChord, PitchClass, ScaleDegree};

// ── Props ─────────────────────────────────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct PracticePanelProps {
    pub target_chord: DiatonicChord,
    /// MIDI notes currently held — same slice passed to PianoPanel.
    pub held_notes: Vec<HeldNote>,
    pub score: PracticeScore,
    /// Fired when the user exits practice mode.
    pub on_exit: Callback<()>,
    /// Fired when all target PitchClasses are held (chord played correctly).
    pub on_advance: Callback<()>,
}

// ── Component ─────────────────────────────────────────────────────────────────

#[function_component(PracticePanel)]
pub fn practice_panel(props: &PracticePanelProps) -> Html {
    // Track the last chord degree we already fired `on_advance` for,
    // so we fire exactly once per chord regardless of how many renders occur
    // while the notes are still held.
    let last_advanced: UseStateHandle<Option<ScaleDegree>> = use_state(|| None);

    // Chord-completion detector: fires when all target PitchClasses are held
    // and we haven't already advanced for this chord.
    {
        let on_advance = props.on_advance.clone();
        let last_advanced = last_advanced.clone();
        let target_pcs: Vec<PitchClass> = props.target_chord.notes.to_vec();
        let target_degree = props.target_chord.degree;

        use_effect_with(
            (props.held_notes.clone(), target_degree),
            move |(held, degree): &(Vec<HeldNote>, ScaleDegree)| {
                // Skip if we already advanced for this exact chord
                if *last_advanced == Some(*degree) {
                    return;
                }
                let all_held = !target_pcs.is_empty()
                    && target_pcs
                        .iter()
                        .all(|t| held.iter().any(|h| h.pitch_class == *t));
                if all_held {
                    last_advanced.set(Some(*degree));
                    on_advance.emit(());
                }
            },
        );
    }

    // ── Derived display values ────────────────────────────────────────────────

    let name = chord_name(props.target_chord.root, props.target_chord.quality);
    let display = chord_display(&props.target_chord); // e.g. "vi - Am"

    // Per-note held status for visual feedback row
    let held_pcs: Vec<PitchClass> =
        props.held_notes.iter().map(|n| n.pitch_class).collect();

    let note_indicators: Html = props
        .target_chord
        .notes
        .iter()
        .map(|&pc| {
            let cls = if held_pcs.contains(&pc) {
                "practice-note practice-note--held"
            } else {
                "practice-note practice-note--waiting"
            };
            html! {
                <span class={cls}>{ pc.name() }</span>
            }
        })
        .collect();

    // Accuracy: guard divide-by-zero
    let accuracy_str = if props.score.total_notes_played > 0 {
        format!(
            "{:.0}%",
            100.0 * props.score.correct_notes as f32
                / props.score.total_notes_played as f32
        )
    } else {
        "–".to_string()
    };

    let on_exit = {
        let cb = props.on_exit.clone();
        Callback::from(move |_: MouseEvent| cb.emit(()))
    };

    html! {
        <div class="practice-panel">
            <div class="practice-panel__header">
                <h2 class="practice-panel__title">{"Practice Mode"}</h2>
                <button class="practice-btn practice-btn--exit" onclick={on_exit}>
                    {"Exit"}
                </button>
            </div>

            <div class="practice-panel__chord-card">
                <div class="practice-panel__chord-display">{ display }</div>
                <div class="practice-panel__chord-name">{ name }</div>
                <div class="practice-panel__target-notes">
                    { note_indicators }
                </div>
                <p class="practice-panel__hint">
                    {"Hold all highlighted notes on your MIDI keyboard."}
                </p>
            </div>

            <div class="practice-panel__score">
                <span class="practice-panel__score-label">{"Accuracy"}</span>
                <span class="practice-panel__score-value">{ accuracy_str }</span>
                if props.score.total_notes_played > 0 {
                    <span class="practice-panel__score-detail">
                        { format!("{} / {} notes", props.score.correct_notes, props.score.total_notes_played) }
                    </span>
                }
            </div>

            <p class="practice-panel__subhint">
                {"The piano keyboard below shows note feedback in real time."}
            </p>
        </div>
    }
}

// ── Helpers exposed for testing ───────────────────────────────────────────────

/// Returns `true` when every note in `target` has a matching PitchClass in `held`.
pub fn all_target_notes_held(target: &[PitchClass], held: &[HeldNote]) -> bool {
    !target.is_empty() && target.iter().all(|t| held.iter().any(|h| h.pitch_class == *t))
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_theory::{Key, PitchClass};

    fn make_held(pcs: &[PitchClass]) -> Vec<HeldNote> {
        pcs.iter()
            .map(|&pc| HeldNote {
                midi_note: pc.to_index() + 60,
                velocity: 64,
                pitch_class: pc,
                octave: 4,
            })
            .collect()
    }

    #[test]
    fn all_target_held_when_all_present() {
        let target = [PitchClass::C, PitchClass::E, PitchClass::G];
        let held = make_held(&[PitchClass::C, PitchClass::E, PitchClass::G]);
        assert!(all_target_notes_held(&target, &held));
    }

    #[test]
    fn not_all_target_held_when_missing_one() {
        let target = [PitchClass::C, PitchClass::E, PitchClass::G];
        let held = make_held(&[PitchClass::C, PitchClass::E]);
        assert!(!all_target_notes_held(&target, &held));
    }

    #[test]
    fn all_target_held_with_extra_notes() {
        // Extra notes beyond the target are OK — chord is still played
        let target = [PitchClass::C, PitchClass::E, PitchClass::G];
        let held = make_held(&[PitchClass::C, PitchClass::D, PitchClass::E, PitchClass::G]);
        assert!(all_target_notes_held(&target, &held));
    }

    #[test]
    fn empty_target_is_not_held() {
        let held = make_held(&[PitchClass::C]);
        assert!(!all_target_notes_held(&[], &held));
    }

    #[test]
    fn not_held_when_held_notes_empty() {
        let target = [PitchClass::C, PitchClass::E, PitchClass::G];
        assert!(!all_target_notes_held(&target, &[]));
    }

    // Property 14 (accuracy invariant) — unit version
    #[test]
    fn accuracy_ratio_in_bounds() {
        let score = PracticeScore {
            correct_notes: 5,
            total_notes_played: 10,
        };
        assert!(score.correct_notes <= score.total_notes_played);
        let ratio = score.correct_notes as f32 / score.total_notes_played as f32;
        assert!((0.0..=1.0).contains(&ratio));
    }

    #[test]
    fn accuracy_zero_total_guarded() {
        let score = PracticeScore::default();
        // Simulates the display guard: total == 0 → show "–"
        let display = if score.total_notes_played > 0 {
            format!("{:.0}%", 100.0 * score.correct_notes as f32 / score.total_notes_played as f32)
        } else {
            "–".to_string()
        };
        assert_eq!(display, "–");
    }

    #[test]
    fn chord_display_includes_roman_and_name() {
        let chords = crate::music_theory::diatonic_chords(Key::major(PitchClass::C));
        let display = chord_display(&chords[0]); // I - C
        assert!(display.contains("I"));
        assert!(display.contains("C"));
    }
}
