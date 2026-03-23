use serde::{Deserialize, Serialize};
use yew::Reducible;

use crate::music_theory::{Key, DiatonicChord, ChordHighlight, PitchClass, diatonic_chords};
use crate::midi::{
    HeldNote, KeySuggestion, MidiStatus, PlayAlongScore, RecognizedChord,
};

// Re-export progression types that now live in music_theory, so that existing
// imports from `crate::state` continue to compile.
pub use crate::music_theory::{
    ProgressionId, BorrowedChord, ProgressionTag, Progression, ActiveProgression,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Theme { Dark, Light }

// ─────────────────────────── MIDI app-level types ────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AppMode {
    Normal,
    PlayAlong,
}

impl Default for AppMode {
    fn default() -> Self { AppMode::Normal }
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
    pub audio_error: Option<String>,
    // ── MIDI fields ──────────────────────────────────────────────────────────
    pub midi_status: MidiStatus,
    pub device_names: Vec<String>,
    pub held_notes: Vec<HeldNote>,
    pub rolling_window: Vec<(PitchClass, f64)>,  // (pitch_class, timestamp_ms)
    pub recognized_chord: Option<RecognizedChord>,
    pub key_suggestions: Vec<KeySuggestion>,
    pub app_mode: AppMode,
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
            audio_error: None,
            // MIDI defaults
            midi_status: MidiStatus::Unavailable,
            device_names: Vec::new(),
            held_notes: Vec::new(),
            rolling_window: Vec::new(),
            recognized_chord: None,
            key_suggestions: Vec::new(),
            app_mode: AppMode::Normal,
            play_along_state: None,
            metronome_active: false,
        }
    }
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

    // ── Task 1: Bug condition exploration tests ───────────────────────────
    // Feature: piano-chord-highlight-sync
    // Bug Condition: highlighted_chord does not advance after SelectProgression

    /// Deterministic "frozen highlight" test — documents the bug by absence of advancement.
    /// PASSES on unfixed code: confirms that after SelectProgression, highlighted_chord is
    /// stuck at index 0 with no mechanism to advance it further.
    /// COUNTEREXAMPLE: current_index stays 0 indefinitely — no AdvanceProgressionChord action exists.
    #[test]
    fn frozen_highlight_documents_the_bug() {
        let id: ProgressionId = 0; // I–V–vi–IV in C major (4 chords)
        let s0 = app_reducer(default_state(), AppAction::SelectProgression(id));
        assert!(s0.active_progression.is_some(), "progression should be active after SelectProgression");

        let progression = crate::data::find_progression(id).unwrap();
        assert!(progression.chords.len() > 1, "progression 0 must have more than 1 chord for this test to be meaningful");

        let expected_chord_0 = chord_highlight_at(&progression, 0);
        assert_eq!(s0.highlighted_chord, expected_chord_0,
            "highlighted_chord should equal chord at index 0 immediately after SelectProgression");
        assert_eq!(s0.active_progression.as_ref().unwrap().current_index, 0,
            "current_index should be 0 after SelectProgression");

        // No further actions are dispatched — simulating that audio plays but state never advances.
        // COUNTEREXAMPLE: highlighted_chord is still index 0 even though audio would be on chord 1+.
        // This documents the freeze: the highlighted_chord has no way to advance during playback.
        assert_eq!(s0.active_progression.unwrap().current_index, 0,
            "COUNTEREXAMPLE: current_index stays frozen at 0 — AdvanceProgressionChord action does not exist");
    }

    mod bug_condition_exploration {
        use super::*;
        use proptest::prelude::*;

        /// Bug condition exploration test — FAILS TO COMPILE on unfixed code.
        /// AdvanceProgressionChord(usize) variant does not exist in AppAction yet.
        /// Failure confirms: the action is missing, so highlighted_chord can never advance
        /// programmatically during audio playback.
        ///
        /// Task 1 complete: compile error was observed and documented (counterexample:
        /// "AdvanceProgressionChord variant does not exist; highlighted_chord stays frozen at index 0").
        /// Gated with #[cfg(any())] so Task 2 preservation tests can compile and run.
        /// Remove the cfg gate in Task 3.5 once AdvanceProgressionChord is implemented.
        ///
        /// EXPECTED OUTCOME on unfixed code: compile error (observed — Task 1 COMPLETE)
        /// EXPECTED OUTCOME after fix (Task 3.5): test passes
        #[cfg(any())] // Task 1 complete — re-enable in Task 3.5 after fix
        proptest! {
            #[test]
            fn prop_advance_progression_chord_updates_highlight(
                target_index in 1usize..4usize,
            ) {
                let id: ProgressionId = 0; // I–V–vi–IV in C major (4 chords)
                let progression = crate::data::find_progression(id).unwrap();
                prop_assume!(target_index < progression.chords.len());

                let s0 = app_reducer(default_state(), AppAction::SelectProgression(id));
                prop_assume!(s0.active_progression.is_some());

                // AdvanceProgressionChord does not exist on unfixed code — compile error here
                let s1 = app_reducer(s0, AppAction::AdvanceProgressionChord(target_index));

                let expected = chord_highlight_at(&progression, target_index);
                prop_assert_eq!(s1.highlighted_chord, expected,
                    "highlighted_chord should match chord at index {} after AdvanceProgressionChord", target_index);
                prop_assert_eq!(s1.active_progression.unwrap().current_index, target_index,
                    "current_index should be {} after AdvanceProgressionChord", target_index);
            }
        }
    }

    // ── Task 2: Preservation property tests ──────────────────────────────
    // Feature: piano-chord-highlight-sync
    // Property 2: Non-playback behaviors are unchanged by the fix
    // EXPECTED OUTCOME on unfixed code: ALL PASS (establishes baseline to preserve)

    mod preservation {
        use super::*;
        use proptest::prelude::*;

        // ── Deterministic observation tests ──────────────────────────────────

        /// Observe on unfixed code: SelectProgression(0) → NextChord → highlighted_chord equals chord at index 1
        #[test]
        fn next_chord_advances_highlight_from_index_zero() {
            let id: ProgressionId = 0;
            let s0 = app_reducer(default_state(), AppAction::SelectProgression(id));
            if s0.active_progression.is_none() { return; }
            let progression = crate::data::find_progression(id).unwrap();
            if progression.chords.len() < 2 { return; }

            let s1 = app_reducer(s0, AppAction::NextChord);
            let expected = chord_highlight_at(&progression, 1);
            assert_eq!(s1.highlighted_chord, expected,
                "NextChord from index 0 should set highlighted_chord to chord at index 1");
            assert_eq!(s1.active_progression.unwrap().current_index, 1);
        }

        /// Observe on unfixed code: SelectProgression(0) → PrevChord → highlighted_chord equals chord at last index
        #[test]
        fn prev_chord_wraps_to_last_from_index_zero() {
            let id: ProgressionId = 0;
            let s0 = app_reducer(default_state(), AppAction::SelectProgression(id));
            if s0.active_progression.is_none() { return; }
            let progression = crate::data::find_progression(id).unwrap();
            let last_idx = progression.chords.len() - 1;

            let s1 = app_reducer(s0, AppAction::PrevChord);
            let expected = chord_highlight_at(&progression, last_idx);
            assert_eq!(s1.highlighted_chord, expected,
                "PrevChord from index 0 should set highlighted_chord to last chord");
            assert_eq!(s1.active_progression.unwrap().current_index, last_idx);
        }

        /// Observe on unfixed code: SelectChord(c) → highlighted_chord equals c, active_progression is None
        #[test]
        fn select_chord_sets_highlighted_and_clears_progression() {
            let chords = crate::music_theory::diatonic_chords(c_major());
            for chord in &chords {
                let s0 = app_reducer(default_state(), AppAction::SelectProgression(0));
                let s1 = app_reducer(s0, AppAction::SelectChord(chord.clone()));
                assert_eq!(s1.active_progression, None,
                    "SelectChord should clear active_progression");
                let hl = s1.highlighted_chord.unwrap();
                assert_eq!(hl.root, chord.notes[0]);
                assert_eq!(hl.third, chord.notes[1]);
                assert_eq!(hl.fifth, chord.notes[2]);
            }
        }

        /// Observe on unfixed code: SelectKey(k) → active_progression is None, highlighted_chord is None
        #[test]
        fn select_key_clears_progression_and_highlight() {
            let s0 = app_reducer(default_state(), AppAction::SelectProgression(0));
            if s0.active_progression.is_none() { return; }

            let new_key = Key { root: PitchClass::G, mode: Mode::Major };
            let s1 = app_reducer(s0, AppAction::SelectKey(new_key));
            assert_eq!(s1.active_progression, None,
                "SelectKey should clear active_progression");
            assert_eq!(s1.highlighted_chord, None,
                "SelectKey should clear highlighted_chord");
        }

        // ── Property-based tests ──────────────────────────────────────────────

        /// For any start index in [0, len-1], NextChord advances current_index by 1 (mod len)
        /// and sets highlighted_chord to the chord at the new index.
        proptest! {
            #[test]
            fn prop_next_chord_preserves_correct_highlight(
                start_index in 0usize..3usize,
            ) {
                let id: ProgressionId = 0; // I–V–vi–IV in C major (4 chords)
                let progression = crate::data::find_progression(id).unwrap();
                let len = progression.chords.len();
                prop_assume!(start_index < len);

                // Navigate to start_index via repeated NextChord
                let mut s = app_reducer(default_state(), AppAction::SelectProgression(id));
                prop_assume!(s.active_progression.is_some());
                for _ in 0..start_index {
                    s = app_reducer(s, AppAction::NextChord);
                }
                prop_assume!(s.active_progression.as_ref().unwrap().current_index == start_index);

                let s_after = app_reducer(s, AppAction::NextChord);
                let expected_index = (start_index + 1) % len;
                let expected_chord = chord_highlight_at(&progression, expected_index);
                prop_assert_eq!(s_after.highlighted_chord, expected_chord,
                    "NextChord from index {} should set highlighted_chord to chord at {}",
                    start_index, expected_index);
                prop_assert_eq!(s_after.active_progression.unwrap().current_index, expected_index);
            }
        }

        /// For any start index in [1, len-1], PrevChord decrements current_index by 1
        /// and sets highlighted_chord to the chord at the new index.
        proptest! {
            #[test]
            fn prop_prev_chord_preserves_correct_highlight(
                start_index in 1usize..4usize,
            ) {
                let id: ProgressionId = 0; // I–V–vi–IV in C major (4 chords)
                let progression = crate::data::find_progression(id).unwrap();
                let len = progression.chords.len();
                prop_assume!(start_index < len);

                // Navigate to start_index via repeated NextChord
                let mut s = app_reducer(default_state(), AppAction::SelectProgression(id));
                prop_assume!(s.active_progression.is_some());
                for _ in 0..start_index {
                    s = app_reducer(s, AppAction::NextChord);
                }
                prop_assume!(s.active_progression.as_ref().unwrap().current_index == start_index);

                let s_after = app_reducer(s, AppAction::PrevChord);
                let expected_index = start_index - 1;
                let expected_chord = chord_highlight_at(&progression, expected_index);
                prop_assert_eq!(s_after.highlighted_chord, expected_chord,
                    "PrevChord from index {} should set highlighted_chord to chord at {}",
                    start_index, expected_index);
                prop_assert_eq!(s_after.active_progression.unwrap().current_index, expected_index);
            }
        }

        /// For any diatonic chord, SelectChord sets highlighted_chord and clears active_progression.
        proptest! {
            #[test]
            fn prop_select_chord_clears_active_progression(
                degree_idx in 0usize..7usize,
            ) {
                let chords = crate::music_theory::diatonic_chords(c_major());
                let chord = chords[degree_idx].clone();

                // Start with an active progression to ensure it gets cleared
                let s0 = app_reducer(default_state(), AppAction::SelectProgression(0));
                let s1 = app_reducer(s0, AppAction::SelectChord(chord.clone()));
                prop_assert_eq!(s1.active_progression, None,
                    "SelectChord should clear active_progression");
                let hl = s1.highlighted_chord.unwrap();
                prop_assert_eq!(hl.root, chord.notes[0]);
                prop_assert_eq!(hl.third, chord.notes[1]);
                prop_assert_eq!(hl.fifth, chord.notes[2]);
            }
        }

        /// For any key, SelectKey clears active_progression and highlighted_chord.
        proptest! {
            #[test]
            fn prop_select_key_clears_progression_and_highlight(
                root_idx in 0u8..12u8,
            ) {
                let key = Key { root: PitchClass::from_index(root_idx), mode: Mode::Major };

                // Start with an active progression to ensure SelectKey clears it
                let s0 = app_reducer(default_state(), AppAction::SelectProgression(0));
                prop_assume!(s0.active_progression.is_some());

                let s1 = app_reducer(s0, AppAction::SelectKey(key));
                prop_assert_eq!(s1.active_progression, None,
                    "SelectKey should clear active_progression");
                prop_assert_eq!(s1.highlighted_chord, None,
                    "SelectKey should clear highlighted_chord");
            }
        }
    }

    // ── MIDI reducer property tests (Task 2.1) ────────────────────────────

    mod property_tests {
        use super::*;
        use crate::midi::{HeldNote, KeySuggestion, PlayAlongScore};
        use proptest::prelude::*;

        fn any_midi_note() -> impl Strategy<Value = u8> { 0u8..=127u8 }
        fn any_velocity() -> impl Strategy<Value = u8> { 1u8..=127u8 }
        fn any_bpm() -> impl Strategy<Value = u32> { 0u32..=400u32 }

        fn make_note(midi_note: u8, velocity: u8) -> HeldNote {
            HeldNote::from_midi(midi_note, velocity)
        }

        fn play_along_state_in(metronome: bool) -> AppState {
            AppState {
                app_mode: AppMode::PlayAlong,
                metronome_active: true,
                play_along_state: Some(PlayAlongState {
                    progression_id: 0,
                    current_chord_index: 0,
                    score: PlayAlongScore::default(),
                    started_at_ms: 0.0,
                    pre_play_along_metronome_active: metronome,
                }),
                ..AppState::default()
            }
        }

        // Feature: midi-keyboard-integration, Property 1: NoteOn/NoteOff round-trip
        proptest! {
            #[test]
            fn prop_note_on_off_round_trip(
                note in any_midi_note(),
                vel  in any_velocity(),
            ) {
                let s0 = AppState::default(); // empty held_notes
                let s1 = app_reducer(s0.clone(), AppAction::MidiNoteOn(make_note(note, vel), 0.0));
                let s2 = app_reducer(s1, AppAction::MidiNoteOff(note));
                prop_assert_eq!(s2.held_notes, s0.held_notes,
                    "held_notes should be unchanged after NoteOn+NoteOff for note {}", note);
            }
        }

        // Feature: midi-keyboard-integration, Property 4: Velocity=0 treated as NoteOff
        proptest! {
            #[test]
            fn prop_velocity_zero_is_note_off(
                note in any_midi_note(),
                vel  in any_velocity(),
            ) {
                // Place the note in held_notes, then send NoteOn with velocity=0
                let s0 = app_reducer(AppState::default(), AppAction::MidiNoteOn(make_note(note, vel), 0.0));
                prop_assume!(s0.held_notes.iter().any(|n| n.midi_note == note));

                let zero_note = HeldNote {
                    midi_note: note,
                    velocity: 0,
                    pitch_class: crate::music_theory::PitchClass::from_index(note % 12),
                    octave: (note / 12) as i8 - 1,
                };
                let s1 = app_reducer(s0, AppAction::MidiNoteOn(zero_note, 0.0));
                prop_assert!(!s1.held_notes.iter().any(|n| n.midi_note == note),
                    "note {} should be removed when velocity=0 NoteOn is dispatched", note);
            }
        }

        // Feature: midi-keyboard-integration, Property 11: ClearRollingWindow resets state
        proptest! {
            #[test]
            fn prop_clear_rolling_window(
                note in any_midi_note(),
                vel  in any_velocity(),
            ) {
                // Build state with a non-empty rolling_window and key_suggestions
                let s0 = app_reducer(
                    AppState { key_suggestions: vec![KeySuggestion {
                        key: crate::music_theory::Key::major(crate::music_theory::PitchClass::C),
                        score: 5,
                    }], ..AppState::default() },
                    AppAction::MidiNoteOn(make_note(note, vel), 1000.0),
                );
                prop_assume!(!s0.rolling_window.is_empty());

                let s1 = app_reducer(s0, AppAction::ClearRollingWindow);
                prop_assert!(s1.rolling_window.is_empty(),
                    "rolling_window should be empty after ClearRollingWindow");
                prop_assert!(s1.key_suggestions.is_empty(),
                    "key_suggestions should be empty after ClearRollingWindow");
            }
        }

        // Feature: midi-keyboard-integration, Property 12: Device disconnection clears held notes
        proptest! {
            #[test]
            fn prop_empty_devices_clears_held_notes(
                note in any_midi_note(),
                vel  in any_velocity(),
            ) {
                let s0 = app_reducer(AppState::default(), AppAction::MidiNoteOn(make_note(note, vel), 0.0));
                prop_assume!(!s0.held_notes.is_empty());

                let s1 = app_reducer(s0, AppAction::MidiDevicesChanged(vec![]));
                prop_assert!(s1.held_notes.is_empty(),
                    "held_notes should be empty after MidiDevicesChanged([])");
            }
        }

        // Feature: midi-keyboard-integration, Property 15: BPM clamping
        proptest! {
            #[test]
            fn prop_set_bpm_clamped(bpm in any_bpm()) {
                let s = app_reducer(AppState::default(), AppAction::SetBpm(bpm));
                prop_assert!(s.bpm >= 40 && s.bpm <= 200,
                    "bpm {} should be clamped to [40, 200], got {}", bpm, s.bpm);
            }
        }

        // Feature: midi-keyboard-integration, Property 16: ExitPlayAlong resets mode
        proptest! {
            #[test]
            fn prop_exit_play_along_resets_mode(metronome in any::<bool>()) {
                let s0 = play_along_state_in(metronome);
                let s1 = app_reducer(s0, AppAction::ExitPlayAlong);
                prop_assert_eq!(s1.app_mode, AppMode::Normal);
                prop_assert!(s1.play_along_state.is_none());
            }
        }

        // Feature: midi-keyboard-integration, Property 17: Metronome toggle round-trip
        proptest! {
            #[test]
            fn prop_metronome_toggle_round_trip(initial in any::<bool>()) {
                let s0 = AppState { metronome_active: initial, ..AppState::default() };
                let s1 = app_reducer(s0.clone(), AppAction::ToggleMetronome);
                let s2 = app_reducer(s1, AppAction::ToggleMetronome);
                prop_assert_eq!(s2.metronome_active, s0.metronome_active,
                    "metronome_active should be unchanged after two ToggleMetronome dispatches");
            }
        }
    }
}
