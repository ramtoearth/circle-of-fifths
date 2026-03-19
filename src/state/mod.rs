use serde::{Deserialize, Serialize};
use yew::Reducible;

use crate::music_theory::{
    Key, DiatonicChord, ChordHighlight, diatonic_chords, Progression, ActiveProgression,
};

// Re-export progression types that now live in music_theory, so that existing
// imports from `crate::state` continue to compile.
pub use crate::music_theory::{
    ProgressionId, BorrowedChord, ProgressionTag,
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
    pub quiz_active: bool,
    pub best_scores: BestScores,
    pub audio_error: Option<String>,
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
            quiz_active: false,
            best_scores: BestScores::default(),
            audio_error: None,
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
