use serde::{Deserialize, Serialize};
use yew::Reducible;

use crate::music_theory::{Key, DiatonicChord, ChordHighlight, PitchClass, diatonic_chords};
use crate::midi::{
    HeldNote, KeySuggestion, MidiStatus, PlayAlongScore, PracticeScore, RecognizedChord,
};

// Re-export progression types that now live in music_theory, so that existing
// imports from `crate::state` continue to compile.
pub use crate::music_theory::{
    ProgressionId, BorrowedChord, ProgressionTag, Progression, ActiveProgression,
};

// ─────────────────────────── Quiz / app-level types ──────────────────────────

/// Quiz question types.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum QuestionType {
    KeySignatureAccidentals,
    RelativeMinor,
    ScaleNotes,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Question {
    pub q_type: QuestionType,
    pub key: Key,
}

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct BestScores {
    pub key_sig: Option<u32>,
    pub relative_minor: Option<u32>,
    pub scale_notes: Option<u32>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Theme { Dark, Light }

// ─────────────────────────── MIDI app-level types ────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppMode {
    Normal,
    Practice,
    PlayAlong,
}

impl Default for AppMode {
    fn default() -> Self { AppMode::Normal }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PracticeState {
    pub target_chord: DiatonicChord,
    pub score: PracticeScore,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PlayAlongState {
    pub progression_id: ProgressionId,
    pub current_chord_index: usize,
    pub score: PlayAlongScore,
    pub started_at_ms: f64,
    pub pre_play_along_metronome_active: bool,
}

/// Top-level app state.
#[derive(Clone, Debug)]
pub struct AppState {
    pub selected_key: Option<Key>,
    pub active_progression: Option<ActiveProgression>,
    pub favorites: Vec<ProgressionId>,
    pub highlighted_chord: Option<ChordHighlight>,
    pub show_note_labels: bool,
    pub octave_offset: i8,
    pub theme: Theme,
    pub muted: bool,
    pub bpm: u32,
    pub quiz_active: bool,
    pub best_scores: BestScores,
    pub audio_error: Option<String>,
    // ── MIDI fields ──────────────────────────────────────────────────────────
    pub midi_status: MidiStatus,
    pub device_names: Vec<String>,
    pub held_notes: Vec<HeldNote>,
    pub rolling_window: Vec<(PitchClass, f64)>,  // (pitch_class, timestamp_ms)
    pub recognized_chord: Option<RecognizedChord>,
    pub key_suggestions: Vec<KeySuggestion>,
    pub app_mode: AppMode,
    pub practice_state: Option<PracticeState>,
    pub play_along_state: Option<PlayAlongState>,
    pub metronome_active: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            selected_key: None,
            active_progression: None,
            favorites: Vec::new(),
            highlighted_chord: None,
            show_note_labels: false,
            octave_offset: 0,
            theme: Theme::Dark,
            muted: false,
            bpm: 120,
            quiz_active: false,
            best_scores: BestScores::default(),
            audio_error: None,
            // MIDI defaults
            midi_status: MidiStatus::Unavailable,
            device_names: Vec::new(),
            held_notes: Vec::new(),
            rolling_window: Vec::new(),
            recognized_chord: None,
            key_suggestions: Vec::new(),
            app_mode: AppMode::Normal,
            practice_state: None,
            play_along_state: None,
            metronome_active: false,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SessionResult {
    pub correct: u32,
    pub total: u32,
    pub q_type_scores: BestScores,
}

/// All state transitions.
#[derive(Clone, Debug)]
pub enum AppAction {
    SelectKey(Key),
    DeselectKey,
    SelectChord(DiatonicChord),
    SelectProgression(ProgressionId),
    NextChord,
    PrevChord,
    ToggleFavorite(ProgressionId),
    ToggleNoteLabels,
    ShiftOctave(i8),
    ToggleTheme,
    ToggleMute,
    EnterQuiz,
    ExitQuiz,
    RecordQuizResult(SessionResult),
    SetAudioError(Option<String>),
    SetBpm(u32),
    // ── MIDI actions ─────────────────────────────────────────────────────────
    MidiStatusChanged(MidiStatus),
    MidiDevicesChanged(Vec<String>),
    /// Note-on: (held_note, timestamp_ms). velocity=0 is treated as NoteOff.
    MidiNoteOn(HeldNote, f64),
    MidiNoteOff(u8),              // midi_note number
    UpdateRecognizedChord(Option<RecognizedChord>),
    UpdateKeySuggestions(Vec<KeySuggestion>),
    ClearRollingWindow,
    EnterPractice,
    ExitPractice,
    PracticeAdvance,
    EnterPlayAlong(ProgressionId),
    ExitPlayAlong,
    PlayAlongTick,
    RecordPlayAlongChordResult(crate::midi::ChordResult),
    ToggleMetronome,
}

// ─────────────────────────── Constants ───────────────────────────────────────

const OCTAVE_MIN: i8 = -2;
const OCTAVE_MAX: i8 = 2;

// ─────────────────────────── Private helpers ─────────────────────────────────

fn chord_highlight_at(progression: &Progression, index: usize) -> Option<ChordHighlight> {
    let degree = progression.chords.get(index)?;
    let chords = diatonic_chords(progression.key);
    chords.iter().find(|c| c.degree == *degree).map(|c| ChordHighlight {
        root: c.notes[0],
        third: c.notes[1],
        fifth: c.notes[2],
    })
}

// ─────────────────────────── Reducer ─────────────────────────────────────────

pub fn app_reducer(state: AppState, action: AppAction) -> AppState {
    match action {
        // ── Circle interaction ────────────────────────────────────────────
        AppAction::SelectKey(key) => {
            if state.selected_key == Some(key) {
                // Clicking the already-selected segment deselects it (Property 2).
                AppState {
                    selected_key: None,
                    active_progression: None,
                    highlighted_chord: None,
                    ..state
                }
            } else {
                AppState {
                    selected_key: Some(key),
                    active_progression: None,
                    highlighted_chord: None,
                    ..state
                }
            }
        }

        AppAction::DeselectKey => AppState {
            selected_key: None,
            active_progression: None,
            highlighted_chord: None,
            ..state
        },

        // ── Chord selection (Property 6) ──────────────────────────────────
        AppAction::SelectChord(chord) => AppState {
            highlighted_chord: Some(ChordHighlight {
                root: chord.notes[0],
                third: chord.notes[1],
                fifth: chord.notes[2],
            }),
            active_progression: None,
            ..state
        },

        // ── Progression controls (Properties 9, 10) ───────────────────────
        AppAction::SelectProgression(id) => {
            if let Some(progression) = crate::data::find_progression(id) {
                let highlighted_chord = chord_highlight_at(&progression, 0);
                AppState {
                    active_progression: Some(ActiveProgression { id, current_index: 0 }),
                    highlighted_chord,
                    ..state
                }
            } else {
                state
            }
        }

        AppAction::NextChord => {
            if let Some(ref active) = state.active_progression {
                if let Some(progression) = crate::data::find_progression(active.id) {
                    let len = progression.chords.len();
                    if len == 0 {
                        return state;
                    }
                    let new_index = (active.current_index + 1) % len;
                    let highlighted_chord = chord_highlight_at(&progression, new_index);
                    return AppState {
                        active_progression: Some(ActiveProgression {
                            id: active.id,
                            current_index: new_index,
                        }),
                        highlighted_chord,
                        ..state
                    };
                }
            }
            state // no-op: no active progression or progression not found
        }

        AppAction::PrevChord => {
            if let Some(ref active) = state.active_progression {
                if let Some(progression) = crate::data::find_progression(active.id) {
                    let len = progression.chords.len();
                    if len == 0 {
                        return state;
                    }
                    let new_index = if active.current_index == 0 {
                        len - 1
                    } else {
                        active.current_index - 1
                    };
                    let highlighted_chord = chord_highlight_at(&progression, new_index);
                    return AppState {
                        active_progression: Some(ActiveProgression {
                            id: active.id,
                            current_index: new_index,
                        }),
                        highlighted_chord,
                        ..state
                    };
                }
            }
            state // no-op: no active progression or progression not found
        }

        // ── Favorites (Property 11) ────────────────────────────────────────
        AppAction::ToggleFavorite(id) => {
            let mut favorites = state.favorites.clone();
            if let Some(pos) = favorites.iter().position(|&f| f == id) {
                favorites.remove(pos);
            } else {
                favorites.push(id);
            }
            AppState { favorites, ..state }
        }

        // ── Piano controls (Properties 13, 14) ───────────────────────────
        AppAction::ToggleNoteLabels => AppState {
            show_note_labels: !state.show_note_labels,
            ..state
        },

        AppAction::ShiftOctave(delta) => AppState {
            octave_offset: (state.octave_offset + delta).clamp(OCTAVE_MIN, OCTAVE_MAX),
            ..state
        },

        // ── Theme / audio (Properties 20, 21) ────────────────────────────
        AppAction::ToggleTheme => AppState {
            theme: match state.theme {
                Theme::Dark  => Theme::Light,
                Theme::Light => Theme::Dark,
            },
            ..state
        },

        AppAction::ToggleMute => AppState {
            muted: !state.muted,
            ..state
        },

        // ── Quiz ──────────────────────────────────────────────────────────
        AppAction::EnterQuiz => AppState {
            quiz_active: true,
            ..state
        },

        AppAction::ExitQuiz => AppState {
            quiz_active: false,
            ..state
        },

        AppAction::RecordQuizResult(result) => {
            let mut best = state.best_scores.clone();
            if let Some(score) = result.q_type_scores.key_sig {
                best.key_sig = Some(best.key_sig.map_or(score, |b| b.max(score)));
            }
            if let Some(score) = result.q_type_scores.relative_minor {
                best.relative_minor = Some(best.relative_minor.map_or(score, |b| b.max(score)));
            }
            if let Some(score) = result.q_type_scores.scale_notes {
                best.scale_notes = Some(best.scale_notes.map_or(score, |b| b.max(score)));
            }
            AppState { best_scores: best, ..state }
        }

        // ── Audio error ───────────────────────────────────────────────────
        AppAction::SetAudioError(err) => AppState {
            audio_error: err,
            ..state
        },

        AppAction::SetBpm(bpm) => AppState { bpm: bpm.clamp(40, 200), ..state },

        // ── MIDI ──────────────────────────────────────────────────────────

        AppAction::MidiStatusChanged(status) => AppState {
            midi_status: status,
            ..state
        },

        // Property 12: empty device list clears held notes
        AppAction::MidiDevicesChanged(names) => {
            let held_notes = if names.is_empty() {
                vec![]
            } else {
                state.held_notes.clone()
            };
            AppState { device_names: names, held_notes, ..state }
        }

        // Property 4: velocity=0 is treated as NoteOff
        AppAction::MidiNoteOn(note, timestamp_ms) => {
            if note.velocity == 0 {
                let held_notes = state.held_notes.iter()
                    .filter(|n| n.midi_note != note.midi_note)
                    .cloned()
                    .collect();
                AppState { held_notes, ..state }
            } else {
                // Replace existing entry for same midi_note (retrigger)
                let mut held_notes: Vec<HeldNote> = state.held_notes.iter()
                    .filter(|n| n.midi_note != note.midi_note)
                    .cloned()
                    .collect();
                held_notes.push(note);
                let mut rolling_window = state.rolling_window.clone();
                rolling_window.push((note.pitch_class, timestamp_ms));
                AppState { held_notes, rolling_window, ..state }
            }
        }

        // Property 1: NoteOff removes the note
        AppAction::MidiNoteOff(midi_note) => {
            let held_notes = state.held_notes.iter()
                .filter(|n| n.midi_note != midi_note)
                .cloned()
                .collect();
            AppState { held_notes, ..state }
        }

        AppAction::UpdateRecognizedChord(chord) => AppState {
            recognized_chord: chord,
            ..state
        },

        AppAction::UpdateKeySuggestions(suggestions) => AppState {
            key_suggestions: suggestions,
            ..state
        },

        // Property 11: clears rolling_window and key_suggestions
        AppAction::ClearRollingWindow => AppState {
            rolling_window: vec![],
            key_suggestions: vec![],
            ..state
        },

        // Practice mode — blocked by UI when not Connected; reducer just sets mode
        AppAction::EnterPractice => {
            if state.app_mode != AppMode::Normal {
                return state;
            }
            let key = match state.selected_key {
                Some(k) => k,
                None => return state,
            };
            let chords = diatonic_chords(key);
            let target_chord = chords[0].clone();
            AppState {
                app_mode: AppMode::Practice,
                practice_state: Some(PracticeState {
                    target_chord,
                    score: PracticeScore::default(),
                }),
                ..state
            }
        }

        AppAction::ExitPractice => AppState {
            app_mode: AppMode::Normal,
            practice_state: None,
            ..state
        },

        AppAction::PracticeAdvance => {
            if let Some(ref ps) = state.practice_state {
                let key = match state.selected_key {
                    Some(k) => k,
                    None => return state,
                };
                let chords = diatonic_chords(key);
                let current_degree = &ps.target_chord.degree;
                let current_idx = chords.iter().position(|c| c.degree == *current_degree)
                    .unwrap_or(0);
                let next_idx = (current_idx + 1) % chords.len();
                let mut new_ps = ps.clone();
                new_ps.target_chord = chords[next_idx].clone();
                AppState {
                    practice_state: Some(new_ps),
                    ..state
                }
            } else {
                state
            }
        }

        AppAction::EnterPlayAlong(progression_id) => {
            if state.app_mode != AppMode::Normal {
                return state;
            }
            let pre = state.metronome_active;
            AppState {
                app_mode: AppMode::PlayAlong,
                metronome_active: true,
                play_along_state: Some(PlayAlongState {
                    progression_id,
                    current_chord_index: 0,
                    score: PlayAlongScore::default(),
                    started_at_ms: 0.0,
                    pre_play_along_metronome_active: pre,
                }),
                ..state
            }
        }

        // Property 16: resets to Normal, restores metronome
        AppAction::ExitPlayAlong => {
            let metronome_active = state.play_along_state
                .as_ref()
                .map(|ps| ps.pre_play_along_metronome_active)
                .unwrap_or(state.metronome_active);
            AppState {
                app_mode: AppMode::Normal,
                play_along_state: None,
                metronome_active,
                ..state
            }
        }

        AppAction::PlayAlongTick => {
            if let Some(ref pa) = state.play_along_state {
                if let Some(progression) = crate::data::find_progression(pa.progression_id) {
                    let next_idx = pa.current_chord_index + 1;
                    if next_idx >= progression.chords.len() {
                        // Progression complete — stay on last chord, UI handles exit
                        return state;
                    }
                    let mut new_pa = pa.clone();
                    new_pa.current_chord_index = next_idx;
                    AppState { play_along_state: Some(new_pa), ..state }
                } else {
                    state
                }
            } else {
                state
            }
        }

        AppAction::RecordPlayAlongChordResult(result) => {
            if let Some(ref pa) = state.play_along_state {
                let mut new_pa = pa.clone();
                new_pa.score.chord_results.push(result);
                AppState { play_along_state: Some(new_pa), ..state }
            } else {
                state
            }
        }

        // Property 17: flip metronome_active
        AppAction::ToggleMetronome => AppState {
            metronome_active: !state.metronome_active,
            ..state
        },
    }
}

// ─────────────────────────── Yew integration ─────────────────────────────────

impl Reducible for AppState {
    type Action = AppAction;

    fn reduce(self: std::rc::Rc<Self>, action: Self::Action) -> std::rc::Rc<Self> {
        app_reducer((*self).clone(), action).into()
    }
}

// ─────────────────────────── Tests ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_theory::{PitchClass, Mode};

    fn c_major() -> Key { Key { root: PitchClass::C, mode: Mode::Major } }
    fn default_state() -> AppState { AppState::default() }

    // ── Task 5.11: unit tests for state transitions ───────────────────────

    #[test]
    fn enter_quiz_sets_quiz_active() {
        let s = app_reducer(default_state(), AppAction::EnterQuiz);
        assert!(s.quiz_active);
    }

    #[test]
    fn exit_quiz_clears_quiz_active() {
        let s0 = app_reducer(default_state(), AppAction::EnterQuiz);
        let s1 = app_reducer(s0, AppAction::ExitQuiz);
        assert!(!s1.quiz_active);
    }

    #[test]
    fn octave_clamp_at_max() {
        let mut s = default_state();
        for _ in 0..10 {
            s = app_reducer(s, AppAction::ShiftOctave(1));
        }
        assert_eq!(s.octave_offset, OCTAVE_MAX);
    }

    #[test]
    fn octave_clamp_at_min() {
        let mut s = default_state();
        for _ in 0..10 {
            s = app_reducer(s, AppAction::ShiftOctave(-1));
        }
        assert_eq!(s.octave_offset, OCTAVE_MIN);
    }

    #[test]
    fn next_chord_noop_when_no_active_progression() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::NextChord);
        assert_eq!(s1.active_progression, s0.active_progression);
        assert_eq!(s1.highlighted_chord, s0.highlighted_chord);
    }

    #[test]
    fn prev_chord_noop_when_no_active_progression() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::PrevChord);
        assert_eq!(s1.active_progression, s0.active_progression);
    }

    // ── Property 1: Segment selection state transition ────────────────────

    // Feature: circle-of-fifths, Property 1: Segment selection state transition
    #[test]
    fn select_key_sets_selected_key() {
        for root_idx in 0u8..12 {
            let key = Key { root: PitchClass::from_index(root_idx), mode: Mode::Major };
            let s = app_reducer(default_state(), AppAction::SelectKey(key));
            assert_eq!(s.selected_key, Some(key));
        }
    }

    // Feature: circle-of-fifths, Property 2: Segment deselection round-trip
    #[test]
    fn select_key_twice_deselects() {
        let key = c_major();
        let s1 = app_reducer(default_state(), AppAction::SelectKey(key));
        let s2 = app_reducer(s1, AppAction::SelectKey(key));
        assert_eq!(s2.selected_key, None);
    }

    // Feature: circle-of-fifths, Property 6: Chord click updates piano highlight
    #[test]
    fn select_chord_sets_highlighted_chord() {
        let chords = crate::music_theory::diatonic_chords(c_major());
        for chord in &chords {
            let s = app_reducer(default_state(), AppAction::SelectChord(chord.clone()));
            let hl = s.highlighted_chord.unwrap();
            assert_eq!(hl.root, chord.notes[0]);
            assert_eq!(hl.third, chord.notes[1]);
            assert_eq!(hl.fifth, chord.notes[2]);
        }
    }

    // Feature: circle-of-fifths, Property 11: Favorite toggle round-trip
    #[test]
    fn favorite_toggle_round_trip() {
        let id: ProgressionId = 42;
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::ToggleFavorite(id));
        assert!(s1.favorites.contains(&id));
        let s2 = app_reducer(s1, AppAction::ToggleFavorite(id));
        assert!(!s2.favorites.contains(&id));
        assert_eq!(s2.favorites.len(), s0.favorites.len());
    }

    // Feature: circle-of-fifths, Property 13: Note label toggle idempotence
    #[test]
    fn toggle_note_labels_round_trip() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::ToggleNoteLabels);
        let s2 = app_reducer(s1, AppAction::ToggleNoteLabels);
        assert_eq!(s2.show_note_labels, s0.show_note_labels);
    }

    // Feature: circle-of-fifths, Property 14: Octave shift round-trip
    #[test]
    fn octave_shift_round_trip() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::ShiftOctave(1));
        let s2 = app_reducer(s1, AppAction::ShiftOctave(-1));
        assert_eq!(s2.octave_offset, s0.octave_offset);
    }

    // Feature: circle-of-fifths, Property 20: Mute toggle round-trip
    #[test]
    fn mute_toggle_round_trip() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::ToggleMute);
        let s2 = app_reducer(s1, AppAction::ToggleMute);
        assert_eq!(s2.muted, s0.muted);
    }

    // Feature: circle-of-fifths, Property 21: Theme toggle round-trip
    #[test]
    fn theme_toggle_round_trip() {
        let s0 = default_state();
        let s1 = app_reducer(s0.clone(), AppAction::ToggleTheme);
        let s2 = app_reducer(s1, AppAction::ToggleTheme);
        assert_eq!(s2.theme, s0.theme);
    }

    // Feature: circle-of-fifths, Property 9: Progression activation sets first chord
    #[test]
    fn select_progression_sets_index_zero() {
        // ID 0 = first progression for C major
        let id: ProgressionId = 0;
        let s = app_reducer(default_state(), AppAction::SelectProgression(id));
        if let Some(active) = s.active_progression {
            assert_eq!(active.id, id);
            assert_eq!(active.current_index, 0);
            assert!(s.highlighted_chord.is_some());
        }
    }

    // Feature: circle-of-fifths, Property 10: Progression navigation round-trip
    #[test]
    fn progression_navigation_round_trip() {
        let id: ProgressionId = 0;
        let s0 = app_reducer(default_state(), AppAction::SelectProgression(id));
        if s0.active_progression.is_none() {
            return;
        }
        let progression = crate::data::find_progression(id).unwrap();
        let n = progression.chords.len();
        let mut s = s0.clone();
        for _ in 0..n {
            s = app_reducer(s, AppAction::NextChord);
        }
        for _ in 0..n {
            s = app_reducer(s, AppAction::PrevChord);
        }
        let orig_idx = s0.active_progression.unwrap().current_index;
        let final_idx = s.active_progression.unwrap().current_index;
        assert_eq!(final_idx, orig_idx);
    }
}
