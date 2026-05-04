use yew::prelude::*;
use crate::midi::MidiStatus;
use crate::music_theory::{chord_name, diatonic_chords, roman_numeral, Key, ScaleDegree};

// ─────────────────────────── Props ───────────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct CustomProgressionBuilderProps {
    pub selected_key: Key,
    pub working_progression: Vec<ScaleDegree>,
    pub midi_status: MidiStatus,
    pub on_toggle: Callback<ScaleDegree>,
    pub on_shift_append: Callback<ScaleDegree>,
    pub on_reset: Callback<()>,
    pub on_start_play_along: Callback<()>,
    pub on_back: Callback<()>,
}

// ─────────────────────────── Component ───────────────────────────────────────

#[function_component(CustomProgressionBuilderPanel)]
pub fn custom_progression_builder_panel(props: &CustomProgressionBuilderProps) -> Html {
    let chords = diatonic_chords(props.selected_key);

    // ── Slots (working progression) ───────────────────────────────────────────
    let slots_html: Html = if props.working_progression.is_empty() {
        html! {
            <span class="builder-panel__placeholder">
                {"Click a chord below to start"}
            </span>
        }
    } else {
        props.working_progression.iter().map(|&degree| {
            let chord = chords.iter().find(|c| c.degree == degree);
            let label = chord
                .map(|c| format!("{} \u{2013} {}", roman_numeral(c.degree, c.quality), chord_name(c.root, c.quality)))
                .unwrap_or_default();
            html! {
                <span class="builder-panel__slot">{label}</span>
            }
        }).collect()
    };

    // ── Chord tiles ───────────────────────────────────────────────────────────
    let tiles_html: Html = chords.iter().map(|c| {
        let degree = c.degree;
        let rn = roman_numeral(c.degree, c.quality);
        let cn = chord_name(c.root, c.quality);
        let count = props.working_progression.iter().filter(|&&d| d == degree).count();

        let badge_html = if count > 0 {
            html! { <span class="chord-tile__badge">{count}</span> }
        } else {
            html! {}
        };

        let on_toggle = props.on_toggle.clone();
        let on_shift_append = props.on_shift_append.clone();

        let onclick = Callback::from(move |e: MouseEvent| {
            if e.shift_key() {
                on_shift_append.emit(degree);
            } else {
                on_toggle.emit(degree);
            }
        });

        html! {
            <button class="chord-tile" onclick={onclick}>
                {badge_html}
                <span class="chord-tile__roman">{rn}</span>
                <span class="chord-tile__name">{cn}</span>
            </button>
        }
    }).collect();

    // ── Start Play Along button ───────────────────────────────────────────────
    let _play_along_disabled = props.working_progression.is_empty()
        || props.midi_status != MidiStatus::Connected;

    let disabled_title = if props.working_progression.is_empty() {
        Some("Add at least one chord to start")
    } else if props.midi_status != MidiStatus::Connected {
        Some("Connect a MIDI keyboard to use Play Along")
    } else {
        None
    };

    let on_start = props.on_start_play_along.reform(|_: MouseEvent| ());
    let on_reset = props.on_reset.reform(|_: MouseEvent| ());
    let on_back  = props.on_back.reform(|_: MouseEvent| ());

    let start_btn_html = if let Some(title) = disabled_title {
        html! {
            <button class="builder-panel__start-btn"
                    disabled=true
                    title={title}
                    aria-disabled="true"
                    aria-label={title}>
                {"Start Play Along"}
            </button>
        }
    } else {
        html! {
            <button class="builder-panel__start-btn"
                    onclick={on_start}>
                {"Start Play Along"}
            </button>
        }
    };

    // ── Render ────────────────────────────────────────────────────────────────
    html! {
        <div class="builder-panel">
            <div class="builder-panel__header">
                <h2>{"Build Your Progression"}</h2>
                <button class="builder-back-btn" onclick={on_back}>{"← Back"}</button>
            </div>

            <div class="builder-panel__slots">
                {slots_html}
            </div>

            <div class="chord-tiles">
                {tiles_html}
            </div>

            <div class="builder-panel__actions">
                <button onclick={on_reset}>{"Reset"}</button>
                {start_btn_html}
            </div>
        </div>
    }
}
