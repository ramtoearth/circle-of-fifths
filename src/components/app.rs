use yew::prelude::*;

use crate::audio::AudioEngineHandle;
use crate::components::circle_view::CircleView;
use crate::components::key_info_panel::KeyInfoPanel;
use crate::components::nav_bar::NavBar;
use crate::components::piano_panel::PianoPanel;
use crate::components::progression_panel::ProgressionPanel;
use crate::components::quiz_panel::QuizPanel;
use crate::music_theory::DiatonicChord;
use crate::state::{AppAction, AppState, ProgressionId, SessionResult, Theme};
use crate::storage::{load_state, save_state};

#[function_component(App)]
pub fn app() -> Html {
    // ── State ────────────────────────────────────────────────────────────────
    let state = {
        let persisted = load_state();
        use_reducer(move || {
            let mut s = AppState::default();
            s.theme = persisted.theme;
            s.muted = persisted.muted;
            s.favorites = persisted.favorites;
            s.best_scores = persisted.best_scores;
            s
        })
    };

    // ── Audio engine ─────────────────────────────────────────────────────────
    let audio = use_memo((), |_| AudioEngineHandle::new());

    // Sync audio error into state on mount
    {
        let state = state.clone();
        let audio = audio.clone();
        use_effect_with((), move |_| {
            if let Some(err) = audio.error() {
                state.dispatch(AppAction::SetAudioError(Some(err)));
            }
        });
    }

    // Sync mute state to audio engine whenever it changes
    {
        let audio = audio.clone();
        let muted = state.muted;
        use_effect_with(muted, move |&m| {
            audio.set_muted(m);
        });
    }

    // Persist to localStorage whenever relevant fields change
    {
        let state_val = (*state).clone();
        use_effect_with(
            (
                state.theme,
                state.muted,
                state.favorites.clone(),
                state.best_scores.clone(),
            ),
            move |_| {
                save_state(&state_val);
            },
        );
    }

    // ── Callbacks ────────────────────────────────────────────────────────────
    let on_segment_click = {
        let state = state.clone();
        Callback::from(move |key| state.dispatch(AppAction::SelectKey(key)))
    };

    let on_chord_click = {
        let state = state.clone();
        let audio = audio.clone();
        Callback::from(move |chord: DiatonicChord| {
            audio.play_chord(&chord.notes);
            state.dispatch(AppAction::SelectChord(chord));
        })
    };

    let on_progression_click = {
        let state = state.clone();
        Callback::from(move |id: ProgressionId| state.dispatch(AppAction::SelectProgression(id)))
    };

    let on_next = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::NextChord))
    };

    let on_prev = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::PrevChord))
    };

    let on_favorite_toggle = {
        let state = state.clone();
        Callback::from(move |id: ProgressionId| state.dispatch(AppAction::ToggleFavorite(id)))
    };

    let on_toggle_labels = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ToggleNoteLabels))
    };

    let on_octave_shift = {
        let state = state.clone();
        Callback::from(move |delta: i8| state.dispatch(AppAction::ShiftOctave(delta)))
    };

    let on_toggle_theme = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ToggleTheme))
    };

    let on_toggle_mute = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ToggleMute))
    };

    let on_enter_quiz = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::EnterQuiz))
    };

    let on_quiz_exit = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ExitQuiz))
    };

    let on_session_end = {
        let state = state.clone();
        Callback::from(move |result: SessionResult| {
            state.dispatch(AppAction::RecordQuizResult(result));
            state.dispatch(AppAction::ExitQuiz);
        })
    };

    // ── Theme class ──────────────────────────────────────────────────────────
    let theme_class = match state.theme {
        Theme::Dark => "app theme-dark",
        Theme::Light => "app theme-light",
    };

    // ── Render ───────────────────────────────────────────────────────────────
    html! {
        <div class={theme_class}>
            <NavBar
                theme={state.theme}
                muted={state.muted}
                on_toggle_theme={on_toggle_theme}
                on_toggle_mute={on_toggle_mute}
                on_enter_quiz={on_enter_quiz}
            />

            if let Some(err) = &state.audio_error {
                <div class="audio-error-banner">
                    { format!("⚠ Audio unavailable: {err}") }
                </div>
            }

            if state.quiz_active {
                <QuizPanel
                    best_scores={state.best_scores.clone()}
                    on_session_end={on_session_end}
                    on_exit={on_quiz_exit}
                />
            } else {
                <div class="main-layout">
                    <CircleView
                        selected_key={state.selected_key}
                        on_segment_click={on_segment_click}
                    />
                    <div class="side-panel">
                        <KeyInfoPanel
                            selected_key={state.selected_key}
                            on_chord_click={on_chord_click}
                        />
                        <ProgressionPanel
                            selected_key={state.selected_key}
                            active_progression={state.active_progression.clone()}
                            favorites={state.favorites.clone()}
                            on_progression_click={on_progression_click}
                            on_next={on_next}
                            on_prev={on_prev}
                            on_favorite_toggle={on_favorite_toggle}
                        />
                    </div>
                </div>
            }

            <PianoPanel
                selected_key={state.selected_key}
                highlighted_chord={state.highlighted_chord.clone()}
                show_labels={state.show_note_labels}
                octave_offset={state.octave_offset}
                on_toggle_labels={on_toggle_labels}
                on_octave_shift={on_octave_shift}
            />
        </div>
    }
}
