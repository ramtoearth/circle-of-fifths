use yew::prelude::*;

use crate::state::Theme;

#[derive(Properties, PartialEq)]
pub struct NavBarProps {
    pub theme: Theme,
    pub muted: bool,
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

    html! {
        <nav class="nav-bar">
            <span class="nav-bar__title">{ "Circle of Fifths" }</span>
            <div class="nav-bar__controls">
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
