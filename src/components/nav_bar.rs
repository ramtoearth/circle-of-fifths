use web_sys::HtmlInputElement;
use yew::prelude::*;

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
    pub on_enter_quiz: Callback<()>,
}

#[function_component(NavBar)]
pub fn nav_bar(props: &NavBarProps) -> Html {
    let on_toggle_theme = props.on_toggle_theme.reform(|_: MouseEvent| ());
    let on_toggle_mute = props.on_toggle_mute.reform(|_: MouseEvent| ());
    let on_enter_quiz = props.on_enter_quiz.reform(|_: MouseEvent| ());

    let theme_label = match props.theme {
        Theme::Dark => "Light Mode",
        Theme::Light => "Dark Mode",
    };

    let mute_label = if props.muted { "Unmute" } else { "Mute" };

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
                        min="60"
                        max="240"
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
                <button class="nav-bar__btn nav-bar__btn--quiz" onclick={on_enter_quiz}>
                    { "Quiz Mode" }
                </button>
            </div>
        </nav>
    }
}
