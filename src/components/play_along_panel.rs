use gloo_timers::callback::Interval;
use yew::prelude::*;

use crate::midi::{ChordResult, HeldNote, PlayAlongScore};
use crate::music_theory::{chord_name, diatonic_chords, roman_numeral, PitchClass, Progression};

// ─────────────────────────── Props ───────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct PlayAlongPanelProps {
    pub progression: Progression,
    pub current_chord_index: usize,
    /// BPM read from AppState; slider in NavBar is the only control.
    pub bpm: u32,
    pub held_notes: Vec<HeldNote>,
    pub score: PlayAlongScore,
    pub on_stop: Callback<()>,
    pub on_tick: Callback<()>,
    pub on_record_result: Callback<ChordResult>,
}

// ─────────────────────────── Component ───────────────────────────────────────

#[function_component(PlayAlongPanel)]
pub fn play_along_panel(props: &PlayAlongPanelProps) -> Html {
    let chord_count = props.progression.chords.len();
    let chords = diatonic_chords(props.progression.key);

    // Shared mutable snapshot of the values the interval closure needs.
    // Updated on every render so the closure always sees the latest state.
    let tick_state = use_mut_ref(|| (props.current_chord_index, props.held_notes.clone()));
    *tick_state.borrow_mut() = (props.current_chord_index, props.held_notes.clone());

    // Beat timer — one interval per BPM value.
    // Dropped (and cleared) when bpm changes or the component unmounts.
    {
        let on_tick = props.on_tick.clone();
        let on_record_result = props.on_record_result.clone();
        let tick_state = tick_state.clone();
        let key = props.progression.key;

        use_effect_with(props.bpm, move |&bpm| {
            let interval_ms = 60_000_u32 / bpm.max(1);
            let chords = diatonic_chords(key);

            let handle = Interval::new(interval_ms, move || {
                let (current_chord_index, held_notes) = tick_state.borrow().clone();

                // Evaluate whether all target PitchClasses were held this beat.
                if let Some(chord) = chords.get(current_chord_index) {
                    let held_pcs: Vec<PitchClass> =
                        held_notes.iter().map(|n| n.pitch_class).collect();
                    let correct = chord.notes.iter().all(|pc| held_pcs.contains(pc));
                    on_record_result.emit(ChordResult { chord_index: current_chord_index, correct });
                }

                on_tick.emit(());
            });

            move || drop(handle)
        });
    }

    let on_stop = props.on_stop.reform(|_: MouseEvent| ());
    let completed = props.score.chord_results.len() >= chord_count;

    if completed {
        // ── Results summary ──────────────────────────────────────────────────
        let correct_count = props.score.chord_results.iter().filter(|r| r.correct).count();
        let accuracy = if chord_count > 0 {
            correct_count as f32 / chord_count as f32 * 100.0
        } else {
            0.0
        };

        let results_html: Html = props.score.chord_results.iter().map(|r| {
            let label = props.progression.chords
                .get(r.chord_index)
                .and_then(|&d| chords.iter().find(|c| c.degree == d))
                .map(|c| format!("{} ({})", chord_name(c.root, c.quality), roman_numeral(c.degree, c.quality)))
                .unwrap_or_default();
            let class = if r.correct {
                "play-along__result play-along__result--correct"
            } else {
                "play-along__result play-along__result--incorrect"
            };
            let icon = if r.correct { "✓" } else { "✗" };
            html! { <li class={class}>{format!("{} {}", icon, label)}</li> }
        }).collect();

        html! {
            <div class="play-along-panel play-along-panel--complete">
                <h2 class="play-along__title">{"Play Along Complete!"}</h2>
                <p class="play-along__accuracy">
                    {format!("Accuracy: {:.0}%  ({}/{} chords correct)", accuracy, correct_count, chord_count)}
                </p>
                <ul class="play-along__results">{results_html}</ul>
                <button class="play-along__stop-btn" onclick={on_stop}>{"Done"}</button>
            </div>
        }
    } else {
        // ── Active play-along ────────────────────────────────────────────────
        let current_chord = chords.get(props.current_chord_index);

        let chord_label = current_chord.map(|c| {
            format!("{} ({})", chord_name(c.root, c.quality), roman_numeral(c.degree, c.quality))
        }).unwrap_or_default();

        let target_pcs: Vec<PitchClass> = current_chord
            .map(|c| c.notes.to_vec())
            .unwrap_or_default();

        let held_pcs: Vec<PitchClass> =
            props.held_notes.iter().map(|n| n.pitch_class).collect();

        let notes_html: Html = target_pcs.iter().map(|pc| {
            let held = held_pcs.contains(pc);
            let class = if held {
                "play-along__note play-along__note--held"
            } else {
                "play-along__note"
            };
            html! { <span class={class}>{pc.name()}</span> }
        }).collect();

        let correct_so_far = props.score.chord_results.iter().filter(|r| r.correct).count();
        let total_so_far = props.score.chord_results.len();

        html! {
            <div class="play-along-panel">
                <h2 class="play-along__title">{"Play Along"}</h2>
                <div class="play-along__progress">
                    {format!("Chord {} / {}  —  {} BPM", props.current_chord_index + 1, chord_count, props.bpm)}
                </div>
                <div class="play-along__target">
                    <span class="play-along__chord-label">{chord_label}</span>
                    <div class="play-along__notes">{notes_html}</div>
                </div>
                if total_so_far > 0 {
                    <div class="play-along__score">
                        {format!("Score so far: {}/{}", correct_so_far, total_so_far)}
                    </div>
                }
                <button class="play-along__stop-btn" onclick={on_stop}>{"Stop"}</button>
            </div>
        }
    }
}
