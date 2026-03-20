use gloo_timers::callback::{Interval, Timeout};
use yew::prelude::*;

use crate::audio::AudioEngineHandle;
use crate::components::circle_view::CircleView;
use crate::components::key_info_panel::KeyInfoPanel;
use crate::components::nav_bar::NavBar;
use crate::components::piano_panel::PianoPanel;
use crate::components::progression_panel::ProgressionPanel;
use crate::components::practice_panel::PracticePanel;
use crate::components::quiz_panel::QuizPanel;
use crate::midi::{detect_keys, recognize_chord, MidiEngine};
use crate::music_theory::DiatonicChord;
use crate::state::{AppAction, AppMode, AppState, ProgressionId, SessionResult, Theme};
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
            s.metronome_active = persisted.metronome_active;
            s
        })
    };

    // ── Audio engine ─────────────────────────────────────────────────────────
    let audio = use_memo((), |_| AudioEngineHandle::new());

    // Tracks which pitch+octave is currently being played in the scale animation
    let playing_note = use_state(|| None::<(crate::music_theory::PitchClass, i32)>);

    // ── MIDI engine — keep alive for component lifetime ───────────────────────
    // Option<MidiEngine> stored in a ref so closures (JS callbacks) are not dropped.
    let midi_engine = use_mut_ref(|| Option::<MidiEngine>::None);

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

    // Initialize MidiEngine on mount — request browser MIDI access.
    // The engine is stored in `midi_engine` ref so closures outlive this render.
    {
        let state = state.clone();
        let midi_engine = midi_engine.clone();
        use_effect_with((), move |_| {
            let dispatch_handle = state.clone();
            let callback = Callback::from(move |action: AppAction| {
                dispatch_handle.dispatch(action);
            });
            let engine = MidiEngine::request_access(callback);
            *midi_engine.borrow_mut() = Some(engine);
        });
    }

    // Re-run chord recognition and key detection whenever held notes or
    // rolling window changes, and push results back into state.
    {
        let state = state.clone();
        let held_notes = state.held_notes.clone();
        let rolling_window = state.rolling_window.clone();
        let selected_key = state.selected_key;
        use_effect_with(
            (held_notes, rolling_window, selected_key),
            move |(held, window, key)| {
                let chord = recognize_chord(held, *key);
                state.dispatch(AppAction::UpdateRecognizedChord(chord));

                #[cfg(target_arch = "wasm32")]
                let now_ms = js_sys::Date::now();
                #[cfg(not(target_arch = "wasm32"))]
                let now_ms = 0.0_f64;

                let suggestions = detect_keys(window, now_ms);
                state.dispatch(AppAction::UpdateKeySuggestions(suggestions));
            },
        );
    }

    // Sync mute state to audio engine whenever it changes
    {
        let audio = audio.clone();
        let muted = state.muted;
        use_effect_with(muted, move |&m| {
            audio.set_muted(m);
        });
    }

    // Metronome: schedule clicks via an Interval recreated when bpm or active changes
    {
        let audio = audio.clone();
        let bpm = state.bpm;
        let metronome_active = state.metronome_active;
        use_effect_with((bpm, metronome_active), move |&(bpm, active)| {
            if !active {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            }
            let interval_ms = (60_000u32).saturating_div(bpm.max(1));
            let audio = audio.clone();
            let handle = Interval::new(interval_ms, move || {
                let start = audio.current_time() + 0.02; // 20 ms lookahead
                audio.schedule_metronome_click(start);
            });
            Box::new(move || drop(handle)) as Box<dyn FnOnce()>
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
                state.metronome_active,
            ),
            move |_| {
                save_state(&state_val);
            },
        );
    }

    // ── Callbacks ────────────────────────────────────────────────────────────
    let on_segment_click = {
        let state = state.clone();
        let audio = audio.clone();
        let playing_note = playing_note.clone();
        Callback::from(move |key| {
            if state.selected_key != Some(key) {
                audio.play_scale(key, state.bpm);
                // Schedule per-note visual highlight to match audio playback
                let notes = crate::audio::scale_note_sequence_with_octaves(key);
                let interval_ms = (60_000.0 / state.bpm as f64) as u32;
                for (i, &(pitch, octave)) in notes.iter().enumerate() {
                    let playing_note = playing_note.clone();
                    Timeout::new(i as u32 * interval_ms, move || {
                        playing_note.set(Some((pitch, octave)));
                    }).forget();
                }
                // Clear after all 8 notes
                let playing_note = playing_note.clone();
                Timeout::new(notes.len() as u32 * interval_ms, move || {
                    playing_note.set(None);
                }).forget();
            }
            state.dispatch(AppAction::SelectKey(key));
        })
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
        let audio = audio.clone();
        Callback::from(move |id: ProgressionId| {
            if let Some(ref p) = crate::data::find_progression(id) {
                audio.play_progression(p);
            }
            state.dispatch(AppAction::SelectProgression(id));
        })
    };

    let on_next = {
        let state = state.clone();
        let audio = audio.clone();
        Callback::from(move |_| {
            if let Some(ref active) = state.active_progression {
                if let Some(ref prog) = crate::data::find_progression(active.id) {
                    let len = prog.chords.len();
                    if len > 0 {
                        let next_idx = (active.current_index + 1) % len;
                        if let Some(&degree) = prog.chords.get(next_idx) {
                            let chords = crate::music_theory::diatonic_chords(prog.key);
                            if let Some(c) = chords.iter().find(|c| c.degree == degree) {
                                audio.play_chord(&c.notes);
                            }
                        }
                    }
                }
            }
            state.dispatch(AppAction::NextChord);
        })
    };

    let on_prev = {
        let state = state.clone();
        let audio = audio.clone();
        Callback::from(move |_| {
            if let Some(ref active) = state.active_progression {
                if let Some(ref prog) = crate::data::find_progression(active.id) {
                    let len = prog.chords.len();
                    if len > 0 {
                        let prev_idx = if active.current_index == 0 {
                            len - 1
                        } else {
                            active.current_index - 1
                        };
                        if let Some(&degree) = prog.chords.get(prev_idx) {
                            let chords = crate::music_theory::diatonic_chords(prog.key);
                            if let Some(c) = chords.iter().find(|c| c.degree == degree) {
                                audio.play_chord(&c.notes);
                            }
                        }
                    }
                }
            }
            state.dispatch(AppAction::PrevChord);
        })
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

    let on_set_bpm = {
        let state = state.clone();
        Callback::from(move |bpm: u32| state.dispatch(AppAction::SetBpm(bpm)))
    };

    let on_enter_quiz = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::EnterQuiz))
    };

    let on_enter_practice = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::EnterPractice))
    };

    let on_toggle_metronome = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ToggleMetronome))
    };

    let on_enter_play_along = {
        let state = state.clone();
        Callback::from(move |id: ProgressionId| state.dispatch(AppAction::EnterPlayAlong(id)))
    };

    let on_practice_exit = {
        let state = state.clone();
        Callback::from(move |_: ()| state.dispatch(AppAction::ExitPractice))
    };

    let on_practice_advance = {
        let state = state.clone();
        Callback::from(move |_: ()| state.dispatch(AppAction::PracticeAdvance))
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
        Theme::Dark => "app",
        Theme::Light => "app light",
    };

    // ── Render ───────────────────────────────────────────────────────────────
    html! {
        <div class={theme_class}>
            <NavBar
                theme={state.theme}
                muted={state.muted}
                selected_key={state.selected_key}
                bpm={state.bpm}
                on_set_bpm={on_set_bpm}
                on_toggle_theme={on_toggle_theme}
                on_toggle_mute={on_toggle_mute}
                on_enter_quiz={on_enter_quiz}
                midi_status={state.midi_status}
                metronome_active={state.metronome_active}
                on_enter_practice={on_enter_practice}
                on_toggle_metronome={on_toggle_metronome}
            />

            if let Some(err) = &state.audio_error {
                <div class="audio-error-banner">
                    { format!("⚠ Audio unavailable: {err}") }
                </div>
            }

            if state.app_mode == AppMode::Practice {
                if let Some(ref ps) = state.practice_state {
                    <PracticePanel
                        target_chord={ps.target_chord.clone()}
                        held_notes={state.held_notes.clone()}
                        score={ps.score.clone()}
                        on_exit={on_practice_exit}
                        on_advance={on_practice_advance}
                    />
                }
            } else if state.quiz_active {
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
                            midi_status={state.midi_status}
                            on_enter_play_along={on_enter_play_along}
                        />
                    </div>
                </div>
            }

            <div class="piano-footer">
                <PianoPanel
                    selected_key={state.selected_key}
                    highlighted_chord={state.highlighted_chord.clone()}
                    playing_note={*playing_note}
                    show_labels={state.show_note_labels}
                    octave_offset={state.octave_offset}
                    on_toggle_labels={on_toggle_labels}
                    on_octave_shift={on_octave_shift}
                    held_notes={state.held_notes.clone()}
                    practice_target={state.practice_state.as_ref().map(|ps| ps.target_chord.notes.to_vec())}
                />
            </div>
        </div>
    }
}
