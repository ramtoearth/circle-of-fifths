use serde::{Deserialize, Serialize};
use crate::music_theory::{Key, DiatonicChord, ChordHighlight, ScaleDegree};

pub type ProgressionId = u32;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct BorrowedChord {
    pub degree: ScaleDegree,
    pub source_key: Key,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum ProgressionTag {
    Pop, Jazz, Blues, Classical, Melancholic, Uplifting,
    #[serde(other)]
    Custom,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Progression {
    pub id: ProgressionId,
    pub key: Key,
    pub chords: Vec<ScaleDegree>,
    pub tags: Vec<ProgressionTag>,
    pub borrowed_chord: Option<BorrowedChord>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ActiveProgression {
    pub id: ProgressionId,
    pub current_index: usize,
}

/// Quiz types
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

/// Top-level app state
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

/// All state transitions
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
