use yew::prelude::*;

use crate::midi::{KeySuggestion, MidiStatus, RecognizedChord};

#[derive(Properties, PartialEq)]
pub struct MidiStatusBarProps {
    pub midi_status: MidiStatus,
    pub device_names: Vec<String>,
    pub recognized_chord: Option<RecognizedChord>,
    pub key_suggestions: Vec<KeySuggestion>,
    pub on_clear_window: Callback<()>,
}

#[function_component(MidiStatusBar)]
pub fn midi_status_bar(props: &MidiStatusBarProps) -> Html {
    let on_clear = props.on_clear_window.reform(|_: MouseEvent| ());

    let (status_cls, status_text) = match props.midi_status {
        MidiStatus::Connected => ("midi-status__badge midi-status__badge--connected", "MIDI Connected"),
        MidiStatus::NoDevices => ("midi-status__badge midi-status__badge--warning", "No MIDI Devices"),
        MidiStatus::PermissionDenied => ("midi-status__badge midi-status__badge--error", "MIDI Permission Denied"),
        MidiStatus::Unavailable => ("midi-status__badge midi-status__badge--error", "MIDI Unavailable"),
    };

    html! {
        <div class="midi-status-bar">
            <span class={status_cls}>{ status_text }</span>

            // Device names
            if !props.device_names.is_empty() {
                <span class="midi-status__devices">
                    { props.device_names.join(", ") }
                </span>
            }

            // Notices for non-connected states
            if props.midi_status == MidiStatus::Unavailable {
                <span class="midi-status__notice">
                    { "Web MIDI is not supported in this browser." }
                </span>
            }
            if props.midi_status == MidiStatus::PermissionDenied {
                <span class="midi-status__notice">
                    { "MIDI access was denied. Allow MIDI in your browser settings and reload." }
                </span>
            }
            if props.midi_status == MidiStatus::NoDevices {
                <span class="midi-status__notice">
                    { "Connect a MIDI device and it will appear here." }
                </span>
            }

            // Recognized chord
            if let Some(chord) = &props.recognized_chord {
                <span class="midi-status__chord">
                    <strong>{ &chord.name }</strong>
                    if let Some(rn) = &chord.roman_numeral {
                        { " " }
                        <span class="midi-status__roman">{ rn }</span>
                        if let Some(diatonic) = chord.is_diatonic {
                            if diatonic {
                                <span class="midi-status__diatonic midi-status__diatonic--yes">{ " (diatonic)" }</span>
                            } else {
                                <span class="midi-status__diatonic midi-status__diatonic--no">{ " (borrowed)" }</span>
                            }
                        }
                    }
                </span>
            }

            // Key suggestions
            if !props.key_suggestions.is_empty() {
                <span class="midi-status__key-suggestions">
                    { "Keys: " }
                    { for props.key_suggestions.iter().take(3).map(|s| html! {
                        <span class="midi-status__key-chip">
                            { format!("{} {}", s.key.root.name(), match s.key.mode {
                                crate::music_theory::Mode::Major => "maj",
                                crate::music_theory::Mode::Minor => "min",
                            }) }
                            <span class="midi-status__key-score">{ format!(" ({})", s.score) }</span>
                        </span>
                    }) }
                </span>
            }

            // Clear rolling window button
            if props.midi_status == MidiStatus::Connected {
                <button class="midi-status__clear-btn" onclick={on_clear}>
                    { "Clear" }
                </button>
            }
        </div>
    }
}
