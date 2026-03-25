use gloo_timers::callback::{Interval, Timeout};
use yew::prelude::*;

use crate::audio::AudioEngineHandle;
use crate::components::circle_view::CircleView;
use crate::components::key_info_panel::KeyInfoPanel;
use crate::components::midi_status_bar::MidiStatusBar;
use crate::components::nav_bar::NavBar;
use crate::components::piano_panel::PianoPanel;
use crate::components::progression_panel::ProgressionPanel;
use crate::components::play_along_panel::PlayAlongPanel;
use crate::midi::{detect_keys, recognize_chord, ChordResult, MidiEngine};
use crate::music_theory::DiatonicChord;
use crate::state::{AppAction, AppMode, AppState, ProgressionId, Theme};
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
            s.metronome_active = persisted.metronome_active;
            s.auto_playback_enabled = persisted.auto_playback_enabled;
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

    // ── Cancellable playback ──────────────────────────────────────────────────
    // Stores all Timeout handles for the active playback session.
    // Dropping the Vec cancels every pending callback.
    let animation_handles = use_mut_ref(|| Vec::<Timeout>::new());

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
                state.metronome_active,
                state.auto_playback_enabled,
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
        let animation_handles = animation_handles.clone();
        Callback::from(move |key| {
            // Cancel any active session first
            animation_handles.borrow_mut().clear();
            audio.stop();
            playing_note.set(None);
            state.dispatch(AppAction::SetPlaying(false));

            if state.selected_key == Some(key) {
                // Clicking the already-selected segment: cancel and deselect, no new session
                state.dispatch(AppAction::SelectKey(key));
                return;
            }

            // Static highlight only when auto-playback is disabled
            if !state.auto_playback_enabled {
                state.dispatch(AppAction::SelectKey(key));
                return;
            }

            // Start new session
            audio.play_scale(key, state.bpm);
            let notes = crate::audio::scale_note_sequence_with_octaves(key);
            let interval_ms = 60_000u32 / state.bpm.max(1);
            for (i, &(pitch, octave)) in notes.iter().enumerate() {
                let playing_note = playing_note.clone();
                let handle = Timeout::new(i as u32 * interval_ms, move || {
                    playing_note.set(Some((pitch, octave)));
                });
                animation_handles.borrow_mut().push(handle);
            }
            // Final cleanup timeout
            {
                let playing_note = playing_note.clone();
                let animation_handles_cb = animation_handles.clone();
                let state = state.clone();
                let handle = Timeout::new(notes.len() as u32 * interval_ms, move || {
                    playing_note.set(None);
                    animation_handles_cb.borrow_mut().clear();
                    state.dispatch(AppAction::SetPlaying(false));
                });
                animation_handles.borrow_mut().push(handle);
            }
            state.dispatch(AppAction::SetPlaying(true));
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
        let animation_handles = animation_handles.clone();
        let playing_note = playing_note.clone();
        Callback::from(move |id: ProgressionId| {
            // Cancel any active session first
            animation_handles.borrow_mut().clear();
            audio.stop();
            playing_note.set(None);
            state.dispatch(AppAction::SetPlaying(false));

            // If same progression is already active, only cancel — no restart
            if state.active_progression.as_ref().map(|a| a.id) == Some(id) {
                return;
            }

            // Static highlight only when auto-playback is disabled
            if !state.auto_playback_enabled {
                state.dispatch(AppAction::SelectProgression(id));
                return;
            }

            let prog = match crate::data::find_progression(id) {
                Some(p) => p,
                None => {
                    state.dispatch(AppAction::SelectProgression(id));
                    return;
                }
            };

            audio.play_progression(&prog);
            // SelectProgression sets index to 0 and highlights the first chord
            state.dispatch(AppAction::SelectProgression(id));

            // Schedule NextChord for each subsequent chord (i=0 is already set by SelectProgression)
            for i in 1..prog.chords.len() {
                let state_cb = state.clone();
                let handle = Timeout::new(i as u32 * 1000, move || {
                    state_cb.dispatch(AppAction::NextChord);
                });
                animation_handles.borrow_mut().push(handle);
            }

            // Final cleanup timeout
            {
                let animation_handles_cb = animation_handles.clone();
                let state_cb = state.clone();
                let handle = Timeout::new(prog.chords.len() as u32 * 1000, move || {
                    animation_handles_cb.borrow_mut().clear();
                    state_cb.dispatch(AppAction::SetPlaying(false));
                });
                animation_handles.borrow_mut().push(handle);
            }

            state.dispatch(AppAction::SetPlaying(true));
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

    let on_toggle_metronome = {
        let state = state.clone();
        Callback::from(move |_| state.dispatch(AppAction::ToggleMetronome))
    };

    let on_enter_play_along = {
        let state = state.clone();
        Callback::from(move |id: ProgressionId| state.dispatch(AppAction::EnterPlayAlong(id)))
    };

    let on_play_along_stop = {
        let state = state.clone();
        Callback::from(move |_: ()| state.dispatch(AppAction::ExitPlayAlong))
    };

    let on_play_along_tick = {
        let state = state.clone();
        Callback::from(move |_: ()| state.dispatch(AppAction::PlayAlongTick))
    };

    let on_play_along_record_result = {
        let state = state.clone();
        Callback::from(move |result: ChordResult| {
            state.dispatch(AppAction::RecordPlayAlongChordResult(result))
        })
    };

    let on_clear_window = {
        let state = state.clone();
        Callback::from(move |_: ()| state.dispatch(AppAction::ClearRollingWindow))
    };

    let on_stop = {
        let animation_handles = animation_handles.clone();
        let audio = audio.clone();
        let playing_note = playing_note.clone();
        let state = state.clone();
        Callback::from(move |_: MouseEvent| {
            animation_handles.borrow_mut().clear();
            audio.stop();
            playing_note.set(None);
            state.dispatch(AppAction::SetPlaying(false));
        })
    };

    let on_toggle_auto_playback = {
        let animation_handles = animation_handles.clone();
        let audio = audio.clone();
        let playing_note = playing_note.clone();
        let state = state.clone();
        Callback::from(move |_: ()| {
            if state.is_playing {
                animation_handles.borrow_mut().clear();
                audio.stop();
                playing_note.set(None);
                state.dispatch(AppAction::SetPlaying(false));
            }
            state.dispatch(AppAction::ToggleAutoPlayback);
        })
    };

    let on_set_time_signature = {
        let state = state.clone();
        Callback::from(move |(n, d): (u32, u32)| state.dispatch(AppAction::SetTimeSignature(n, d)))
    };

    // ── Derived: practice_target for PianoPanel ───────────────────────────────
    let practice_target: Option<Vec<crate::music_theory::PitchClass>> =
        if let Some(ref pa) = state.play_along_state {
            crate::data::find_progression(pa.progression_id).and_then(|prog| {
                let chords = crate::music_theory::diatonic_chords(prog.key);
                prog.chords
                    .get(pa.current_chord_index)
                    .and_then(|&d| chords.iter().find(|c| c.degree == d))
                    .map(|c| c.notes.to_vec())
            })
        } else {
            None
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
                midi_status={state.midi_status}
                metronome_active={state.metronome_active}
                on_toggle_metronome={on_toggle_metronome}
                auto_playback_enabled={state.auto_playback_enabled}
                on_toggle_auto_playback={on_toggle_auto_playback}
                time_signature={state.time_signature}
                on_set_time_signature={on_set_time_signature}
            />

            <MidiStatusBar
                midi_status={state.midi_status}
                device_names={state.device_names.clone()}
                recognized_chord={state.recognized_chord.clone()}
                key_suggestions={state.key_suggestions.clone()}
                on_clear_window={on_clear_window}
            />

            if let Some(err) = &state.audio_error {
                <div class="audio-error-banner">
                    { format!("⚠ Audio unavailable: {err}") }
                </div>
            }

            if state.app_mode == AppMode::PlayAlong {
                if let Some(ref pa) = state.play_along_state {
                    if let Some(progression) = crate::data::find_progression(pa.progression_id) {
                        <PlayAlongPanel
                            progression={progression}
                            current_chord_index={pa.current_chord_index}
                            bpm={state.bpm}
                            held_notes={state.held_notes.clone()}
                            score={pa.score.clone()}
                            on_stop={on_play_along_stop}
                            on_tick={on_play_along_tick}
                            on_record_result={on_play_along_record_result}
                        />
                    }
                }
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
                if state.is_playing {
                    <button class="stop-btn" onclick={on_stop} aria-label="Stop playback">{"■ Stop"}</button>
                }
                <PianoPanel
                    selected_key={state.selected_key}
                    highlighted_chord={state.highlighted_chord.clone()}
                    playing_note={*playing_note}
                    show_labels={state.show_note_labels}
                    octave_offset={state.octave_offset}
                    on_toggle_labels={on_toggle_labels}
                    on_octave_shift={on_octave_shift}
                    held_notes={state.held_notes.clone()}
                    practice_target={practice_target}
                />
            </div>
        </div>
    }
}
