use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::midi::MidiStatus;
use crate::music_theory::{Key, Mode};
use crate::state::Theme;

#[derive(Properties, PartialEq)]
pub struct NavBarProps {
    pub theme: Theme,
    pub muted: bool,
    pub selected_key: Option<Key>,
    pub bpm: u32,
    pub on_set_bpm: Callback<u32>,
    pub on_toggle_theme: Callback<()>,
    pub on_toggle_mute: Callback<()>,
    pub midi_status: MidiStatus,
    pub metronome_active: bool,
    pub on_toggle_metronome: Callback<()>,
    pub auto_playback_enabled: bool,
    pub on_toggle_auto_playback: Callback<()>,
}

#[function_component(NavBar)]
pub fn nav_bar(props: &NavBarProps) -> Html {
    let on_toggle_theme = props.on_toggle_theme.reform(|_: MouseEvent| ());
    let on_toggle_mute = props.on_toggle_mute.reform(|_: MouseEvent| ());
    let on_toggle_metronome = props.on_toggle_metronome.reform(|_: MouseEvent| ());
    let on_toggle_auto_playback = props.on_toggle_auto_playback.reform(|_: MouseEvent| ());

    let theme_label = match props.theme {
        Theme::Dark => "Light Mode",
        Theme::Light => "Dark Mode",
    };

    let mute_label = if props.muted { "Unmute" } else { "Mute" };
    let metronome_label = if props.metronome_active { "Metronome: On" } else { "Metronome: Off" };
    let auto_playback_label = if props.auto_playback_enabled { "Auto-Play: On" } else { "Auto-Play: Off" };
    let auto_playback_aria = if props.auto_playback_enabled { "Disable auto-playback" } else { "Enable auto-playback" };

    let key_label = props.selected_key.map(|k| {
        let mode_str = match k.mode {
            Mode::Major => "Major",
            Mode::Minor => "Minor",
        };
        format!("{} {}", k.root.name(), mode_str)
    }).unwrap_or_else(|| "\u{2013}".to_string());

    let on_bpm_input = {
        let on_set_bpm = props.on_set_bpm.clone();
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            if let Ok(val) = input.value().parse::<u32>() {
                on_set_bpm.emit(val);
            }
        })
    };

    html! {
        <nav class="nav-bar">
            <span class="nav-bar__title">{ "Circle of Fifths" }</span>
            <span class="nav-bar__key">{ key_label }</span>
            <div class="nav-bar__controls">
                <label class="nav-bar__bpm">
                    { format!("BPM: {}", props.bpm) }
                    <input
                        type="range"
                        min="40"
                        max="200"
                        value={props.bpm.to_string()}
                        oninput={on_bpm_input}
                    />
                </label>
                <button class="nav-bar__btn nav-bar__btn--theme" onclick={on_toggle_theme}>
                    { theme_label }
                </button>
                <button class="nav-bar__btn nav-bar__btn--mute" onclick={on_toggle_mute}>
                    { mute_label }
                </button>
                <button
                    class="nav-bar__btn nav-bar__btn--metronome"
                    onclick={on_toggle_metronome}
                >
                    { metronome_label }
                </button>
                <button
                    class="nav-bar__btn nav-bar__btn--auto-playback"
                    onclick={on_toggle_auto_playback}
                    aria-label={auto_playback_aria}
                    aria-pressed={props.auto_playback_enabled.to_string()}
                >
                    { auto_playback_label }
                </button>
            </div>
        </nav>
    }
}
