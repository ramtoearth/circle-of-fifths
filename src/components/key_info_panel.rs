use yew::prelude::*;

use crate::music_theory::{
    chord_name, diatonic_chords, key_signature, roman_numeral, scale_notes, ChordQuality,
    DiatonicChord, Key, Mode,
};

#[derive(Properties, PartialEq)]
pub struct KeyInfoPanelProps {
    pub selected_key: Option<Key>,
    pub on_chord_click: Callback<DiatonicChord>,
}

#[function_component(KeyInfoPanel)]
pub fn key_info_panel(props: &KeyInfoPanelProps) -> Html {
    let key = match props.selected_key {
        Some(k) => k,
        None => {
            return html! {
                <div class="key-info-panel key-info-panel--empty">
                    <p class="key-info-panel__placeholder">
                        { "Select a key from the circle to see details." }
                    </p>
                </div>
            }
        }
    };

    // ── Key name ─────────────────────────────────────────────────────────────
    let mode_str = match key.mode {
        Mode::Major => "major",
        Mode::Minor => "minor",
    };
    let key_name = format!("{} {}", key.root.name(), mode_str);

    // ── Key signature ─────────────────────────────────────────────────────────
    let sig = key_signature(key);
    let sig_label = if sig.sharps > 0 {
        let names: Vec<&str> = sig.notes.iter().map(|n| n.sharp_name()).collect();
        format!(
            "{} sharp{}: {}",
            sig.sharps,
            if sig.sharps == 1 { "" } else { "s" },
            names.join(", ")
        )
    } else if sig.flats > 0 {
        let names: Vec<&str> = sig.notes.iter().map(|n| n.name()).collect();
        format!(
            "{} flat{}: {}",
            sig.flats,
            if sig.flats == 1 { "" } else { "s" },
            names.join(", ")
        )
    } else {
        "No accidentals".to_string()
    };

    // ── Scale notes ───────────────────────────────────────────────────────────
    let notes = scale_notes(key);

    // ── Diatonic chords ───────────────────────────────────────────────────────
    let chords = diatonic_chords(key);

    html! {
        <div class="key-info-panel">
            <h2 class="key-info-panel__title">{ key_name }</h2>

            <section class="key-info-panel__section">
                <h3 class="key-info-panel__section-title">{ "Key Signature" }</h3>
                <p class="key-info-panel__sig">{ sig_label }</p>
            </section>

            <section class="key-info-panel__section">
                <h3 class="key-info-panel__section-title">{ "Scale Notes" }</h3>
                <div class="key-info-panel__notes">
                    { notes.iter().map(|pc| html! {
                        <span class="key-info-panel__note">{
                            if sig.sharps > 0 { pc.sharp_name() } else { pc.name() }
                        }</span>
                    }).collect::<Html>() }
                </div>
            </section>

            <section class="key-info-panel__section">
                <h3 class="key-info-panel__section-title">{ "Diatonic Chords" }</h3>
                <div class="chord-legend">
                    <span class="chord-legend__item">
                        <span class="chord-badge chord-badge--major">{"Major"}</span>
                        {" — happy, stable"}
                    </span>
                    <span class="chord-legend__item">
                        <span class="chord-badge chord-badge--minor">{"Minor"}</span>
                        {" — sad, emotional"}
                    </span>
                    <span class="chord-legend__item">
                        <span class="chord-badge chord-badge--diminished">{"dim"}</span>
                        {" — tense, unstable"}
                    </span>
                </div>
                <ul class="key-info-panel__chords">
                    { chords.iter().map(|chord| {
                        let chord_for_cb = chord.clone();
                        let on_click = props.on_chord_click.reform(move |_: MouseEvent| chord_for_cb.clone());
                        let roman = roman_numeral(chord.degree, chord.quality);
                        let name  = chord_name(chord.root, chord.quality);
                        let (quality_class, quality_label) = match chord.quality {
                            ChordQuality::Major      => ("chord-badge--major",      "Major"),
                            ChordQuality::Minor      => ("chord-badge--minor",      "Minor"),
                            ChordQuality::Diminished => ("chord-badge--diminished", "dim"),
                        };
                        html! {
                            <li class="key-info-panel__chord" onclick={ on_click }>
                                <span class="key-info-panel__chord-roman">{ roman }</span>
                                <span class="key-info-panel__chord-name">{ name }</span>
                                <span class={classes!("chord-badge", quality_class)}>{ quality_label }</span>
                            </li>
                        }
                    }).collect::<Html>() }
                </ul>
            </section>
        </div>
    }
}
