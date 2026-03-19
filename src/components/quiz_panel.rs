use yew::prelude::*;
use wasm_bindgen::JsCast;
use crate::music_theory::{key_signature, relative_minor, scale_notes, Key, Mode, PitchClass};
use crate::state::{BestScores, Question, QuestionType, SessionResult};

// ─────────────────────────── Pure helper functions ────────────────────────────

/// Build the full 36-question pool: 12 major keys × 3 question types.
pub fn build_question_pool() -> Vec<Question> {
    let mut pool = Vec::with_capacity(36);
    for i in 0u8..12 {
        let key = Key { root: PitchClass::from_index(i), mode: Mode::Major };
        pool.push(Question { q_type: QuestionType::KeySignatureAccidentals, key });
        pool.push(Question { q_type: QuestionType::RelativeMinor, key });
        pool.push(Question { q_type: QuestionType::ScaleNotes, key });
    }
    pool
}

/// Canonical answer string for a question.
pub fn correct_answer(q: &Question) -> String {
    match q.q_type {
        QuestionType::KeySignatureAccidentals => {
            let ks = key_signature(q.key);
            let total = ks.sharps + ks.flats;
            total.to_string()
        }
        QuestionType::RelativeMinor => {
            let minor_key = relative_minor(q.key);
            format!("{} minor", minor_key.root.name())
        }
        QuestionType::ScaleNotes => {
            let notes = scale_notes(q.key);
            notes.iter().map(|p| p.name()).collect::<Vec<_>>().join(" ")
        }
    }
}

/// Case-insensitive answer evaluation.
pub fn evaluate_answer(q: &Question, user_answer: &str) -> bool {
    correct_answer(q).to_lowercase() == user_answer.trim().to_lowercase()
}

/// Human-readable question prompt.
pub fn question_prompt(q: &Question) -> String {
    let key_name = q.key.root.name();
    match q.q_type {
        QuestionType::KeySignatureAccidentals => {
            format!("How many accidentals (sharps or flats) are in the {} major key signature?", key_name)
        }
        QuestionType::RelativeMinor => {
            format!("What is the relative minor of {} major?", key_name)
        }
        QuestionType::ScaleNotes => {
            format!("List the notes of the {} major scale (space-separated):", key_name)
        }
    }
}

/// Fisher-Yates shuffle — only runs random permutation on wasm32; identity on native.
pub fn shuffle_questions(v: Vec<Question>) -> Vec<Question> {
    #[cfg(target_arch = "wasm32")]
    {
        let mut v = v;
        let n = v.len();
        for i in (1..n).rev() {
            let j = (js_sys::Math::random() * (i + 1) as f64) as usize;
            v.swap(i, j);
        }
        return v;
    }
    #[allow(unreachable_code)]
    v
}

// ─────────────────────────── SessionScores ───────────────────────────────────

#[derive(Clone, Default, Debug, PartialEq)]
pub struct SessionScores {
    pub key_sig_correct: u32,
    pub key_sig_total: u32,
    pub rel_minor_correct: u32,
    pub rel_minor_total: u32,
    pub scale_notes_correct: u32,
    pub scale_notes_total: u32,
}

impl SessionScores {
    pub fn record(&mut self, q_type: QuestionType, correct: bool) {
        match q_type {
            QuestionType::KeySignatureAccidentals => {
                self.key_sig_total += 1;
                if correct { self.key_sig_correct += 1; }
            }
            QuestionType::RelativeMinor => {
                self.rel_minor_total += 1;
                if correct { self.rel_minor_correct += 1; }
            }
            QuestionType::ScaleNotes => {
                self.scale_notes_total += 1;
                if correct { self.scale_notes_correct += 1; }
            }
        }
    }

    pub fn total_correct(&self) -> u32 {
        self.key_sig_correct + self.rel_minor_correct + self.scale_notes_correct
    }

    pub fn total_answered(&self) -> u32 {
        self.key_sig_total + self.rel_minor_total + self.scale_notes_total
    }

    pub fn to_session_result(&self) -> SessionResult {
        let key_sig = if self.key_sig_total > 0 { Some(self.key_sig_correct) } else { None };
        let relative_minor = if self.rel_minor_total > 0 { Some(self.rel_minor_correct) } else { None };
        let scale_notes = if self.scale_notes_total > 0 { Some(self.scale_notes_correct) } else { None };
        SessionResult {
            correct: self.total_correct(),
            total: self.total_answered(),
            q_type_scores: BestScores { key_sig, relative_minor, scale_notes },
        }
    }
}

// ─────────────────────────── Component ───────────────────────────────────────

#[derive(Properties, PartialEq)]
pub struct QuizPanelProps {
    pub best_scores: BestScores,
    pub on_session_end: Callback<SessionResult>,
    pub on_exit: Callback<()>,
}

/// Internal UI state for the quiz session.
#[derive(Clone, PartialEq)]
enum QuizPhase {
    Answering,
    ShowingFeedback { was_correct: bool },
    Summary,
}

#[function_component(QuizPanel)]
pub fn quiz_panel(props: &QuizPanelProps) -> Html {
    // Questions are generated once on mount.
    let questions = use_state(|| shuffle_questions(build_question_pool()));
    let current_idx = use_state(|| 0usize);
    let input_value = use_state(String::new);
    let phase = use_state(|| QuizPhase::Answering);
    let scores = use_state(SessionScores::default);

    let total_questions = questions.len();
    let idx = *current_idx;

    // ── Summary screen ────────────────────────────────────────────────────────
    if *phase == QuizPhase::Summary {
        let result = scores.to_session_result();
        let on_exit = props.on_exit.clone();
        let on_session_end = props.on_session_end.clone();
        let result_clone = result.clone();

        let finish = {
            let on_session_end = on_session_end.clone();
            let on_exit = on_exit.clone();
            let result = result_clone.clone();
            Callback::from(move |_: MouseEvent| {
                on_session_end.emit(result.clone());
                on_exit.emit(());
            })
        };

        let best_key_sig = props.best_scores.key_sig
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string());
        let best_rel_minor = props.best_scores.relative_minor
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string());
        let best_scale = props.best_scores.scale_notes
            .map(|v| v.to_string())
            .unwrap_or_else(|| "—".to_string());

        return html! {
            <div class="quiz-panel quiz-summary">
                <h2>{"Quiz Complete!"}</h2>
                <p class="quiz-final-score">
                    {format!("Final Score: {}/{}", result.correct, result.total)}
                </p>
                <table class="quiz-score-table">
                    <thead>
                        <tr>
                            <th>{"Category"}</th>
                            <th>{"This Session"}</th>
                            <th>{"Best"}</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>{"Key Signatures"}</td>
                            <td>{format!("{}/{}", scores.key_sig_correct, scores.key_sig_total)}</td>
                            <td>{best_key_sig}</td>
                        </tr>
                        <tr>
                            <td>{"Relative Minor"}</td>
                            <td>{format!("{}/{}", scores.rel_minor_correct, scores.rel_minor_total)}</td>
                            <td>{best_rel_minor}</td>
                        </tr>
                        <tr>
                            <td>{"Scale Notes"}</td>
                            <td>{format!("{}/{}", scores.scale_notes_correct, scores.scale_notes_total)}</td>
                            <td>{best_scale}</td>
                        </tr>
                    </tbody>
                </table>
                <button onclick={finish} class="quiz-btn quiz-btn-primary">{"Finish"}</button>
            </div>
        };
    }

    // ── Current question ──────────────────────────────────────────────────────
    let question = questions[idx].clone();
    let answered = *phase != QuizPhase::Answering;
    let is_last = idx + 1 >= total_questions;

    let q_type_label = match question.q_type {
        QuestionType::KeySignatureAccidentals => "Key Signature",
        QuestionType::RelativeMinor => "Relative Minor",
        QuestionType::ScaleNotes => "Scale Notes",
    };

    // oninput: update controlled input value.
    let input_value_handle = input_value.clone();
    let oninput = Callback::from(move |e: InputEvent| {
        if let Some(target) = e.target() {
            if let Ok(el) = target.dyn_into::<web_sys::HtmlInputElement>() {
                input_value_handle.set(el.value());
            }
        }
    });

    // Submit answer.
    let phase_handle = phase.clone();
    let scores_handle = scores.clone();
    let question_for_submit = question.clone();
    let input_for_submit = input_value.clone();
    let onsubmit = Callback::from(move |_: MouseEvent| {
        if *phase_handle != QuizPhase::Answering {
            return;
        }
        let user_input = (*input_for_submit).clone();
        let correct = evaluate_answer(&question_for_submit, &user_input);
        let mut new_scores = (*scores_handle).clone();
        new_scores.record(question_for_submit.q_type, correct);
        scores_handle.set(new_scores);
        phase_handle.set(QuizPhase::ShowingFeedback { was_correct: correct });
    });

    // Next question / See Results.
    let phase_handle2 = phase.clone();
    let current_idx_handle = current_idx.clone();
    let input_value_handle2 = input_value.clone();
    let on_next = Callback::from(move |_: MouseEvent| {
        let next_idx = *current_idx_handle + 1;
        if next_idx >= total_questions {
            phase_handle2.set(QuizPhase::Summary);
        } else {
            current_idx_handle.set(next_idx);
            input_value_handle2.set(String::new());
            phase_handle2.set(QuizPhase::Answering);
        }
    });

    // Exit button.
    let on_exit = props.on_exit.clone();
    let on_exit_click = Callback::from(move |_: MouseEvent| {
        on_exit.emit(());
    });

    // Feedback content.
    let feedback_html = match *phase {
        QuizPhase::ShowingFeedback { was_correct } => {
            if was_correct {
                html! { <p class="quiz-feedback quiz-feedback-correct">{"Correct!"}</p> }
            } else {
                let answer = correct_answer(&question);
                html! {
                    <p class="quiz-feedback quiz-feedback-incorrect">
                        {format!("Incorrect. Correct answer: {}", answer)}
                    </p>
                }
            }
        }
        _ => html! {},
    };

    let next_btn_label = if is_last { "See Results" } else { "Next Question" };

    html! {
        <div class="quiz-panel">
            <div class="quiz-header">
                <span class="quiz-progress">
                    {format!("Question {} of {} | Score: {}/{}", idx + 1, total_questions, scores.total_correct(), scores.total_answered())}
                </span>
                <button onclick={on_exit_click} class="quiz-btn quiz-btn-exit">{"Exit Quiz"}</button>
            </div>

            <div class="quiz-question">
                <span class="quiz-type-label">{q_type_label}</span>
                <p class="quiz-prompt">{question_prompt(&question)}</p>
            </div>

            <div class="quiz-input-row">
                <input
                    type="text"
                    class="quiz-input"
                    value={(*input_value).clone()}
                    oninput={oninput}
                    disabled={answered}
                    placeholder="Type your answer..."
                />
                if !answered {
                    <button onclick={onsubmit} class="quiz-btn quiz-btn-primary">{"Submit"}</button>
                }
            </div>

            {feedback_html}

            if answered {
                <button onclick={on_next} class="quiz-btn quiz-btn-primary">
                    {next_btn_label}
                </button>
            }
        </div>
    }
}

// ─────────────────────────── Tests ───────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::music_theory::{PitchClass, Mode};
    use crate::state::QuestionType;

    fn all_major_keys() -> Vec<Key> {
        (0u8..12).map(|i| Key { root: PitchClass::from_index(i), mode: Mode::Major }).collect()
    }

    // 13.4 Unit tests

    #[test]
    fn question_pool_has_36_questions() {
        assert_eq!(build_question_pool().len(), 36);
    }

    #[test]
    fn question_pool_covers_all_three_types() {
        let pool = build_question_pool();
        let has_key_sig = pool.iter().any(|q| q.q_type == QuestionType::KeySignatureAccidentals);
        let has_rel_minor = pool.iter().any(|q| q.q_type == QuestionType::RelativeMinor);
        let has_scale = pool.iter().any(|q| q.q_type == QuestionType::ScaleNotes);
        assert!(has_key_sig);
        assert!(has_rel_minor);
        assert!(has_scale);
    }

    #[test]
    fn question_pool_covers_all_12_major_keys() {
        let pool = build_question_pool();
        for key in all_major_keys() {
            assert!(pool.iter().any(|q| q.key == key), "Missing key: {:?}", key);
        }
    }

    #[test]
    fn canonical_answers_evaluate_as_correct() {
        for q in build_question_pool() {
            let answer = correct_answer(&q);
            assert!(
                evaluate_answer(&q, &answer),
                "canonical answer did not evaluate correctly for {:?}: '{}'",
                q,
                answer
            );
        }
    }

    #[test]
    fn nonsense_answer_evaluates_as_incorrect() {
        let q = Question { q_type: QuestionType::KeySignatureAccidentals, key: Key::major(PitchClass::C) };
        assert!(!evaluate_answer(&q, "xyzzy"));
    }

    #[test]
    fn evaluate_answer_case_insensitive() {
        let q = Question { q_type: QuestionType::RelativeMinor, key: Key::major(PitchClass::C) };
        // Canonical: "A minor"
        assert!(evaluate_answer(&q, "a minor"));
        assert!(evaluate_answer(&q, "A MINOR"));
        assert!(evaluate_answer(&q, "A Minor"));
    }

    #[test]
    fn c_major_key_sig_is_zero() {
        let q = Question { q_type: QuestionType::KeySignatureAccidentals, key: Key::major(PitchClass::C) };
        assert_eq!(correct_answer(&q), "0");
    }

    #[test]
    fn g_major_key_sig_is_one() {
        let q = Question { q_type: QuestionType::KeySignatureAccidentals, key: Key::major(PitchClass::G) };
        assert_eq!(correct_answer(&q), "1");
    }

    #[test]
    fn c_major_scale_notes() {
        let q = Question { q_type: QuestionType::ScaleNotes, key: Key::major(PitchClass::C) };
        assert_eq!(correct_answer(&q), "C D E F G A B");
    }

    #[test]
    fn relative_minor_of_c_major() {
        let q = Question { q_type: QuestionType::RelativeMinor, key: Key::major(PitchClass::C) };
        assert_eq!(correct_answer(&q), "A minor");
    }

    #[test]
    fn session_scores_correct_increments_by_one() {
        let mut scores = SessionScores::default();
        scores.record(QuestionType::KeySignatureAccidentals, true);
        assert_eq!(scores.key_sig_correct, 1);
        assert_eq!(scores.key_sig_total, 1);
        scores.record(QuestionType::KeySignatureAccidentals, false);
        assert_eq!(scores.key_sig_correct, 1);
        assert_eq!(scores.key_sig_total, 2);
    }

    // Feature: circle-of-fifths, Property 15: Question pool completeness and shuffle
    #[test]
    fn shuffled_pool_is_permutation_of_original() {
        let original = build_question_pool();
        // On non-wasm32 targets shuffle is a no-op, so we verify the pool is
        // identical in content (it will be in the same order but that's fine).
        let shuffled = shuffle_questions(original.clone());
        assert_eq!(shuffled.len(), original.len());
        // Every question in original must appear in shuffled.
        for q in &original {
            assert!(shuffled.contains(q), "shuffled pool is missing question: {:?}", q);
        }
    }

    // Feature: circle-of-fifths, Property 16: Answer evaluation correctness
    #[test]
    fn evaluate_answer_correct_iff_canonical() {
        for q in build_question_pool() {
            let canonical = correct_answer(&q);
            assert!(evaluate_answer(&q, &canonical), "canonical answer rejected for {:?}", q);
            // A clearly wrong answer should not match.
            assert!(!evaluate_answer(&q, "WRONG_ANSWER_XYZ"), "wrong answer accepted for {:?}", q);
        }
    }

    // Feature: circle-of-fifths, Property 17: Score tracking invariant
    #[test]
    fn score_tracking_invariant_holds() {
        let pool = build_question_pool();
        let mut scores = SessionScores::default();
        for (i, q) in pool.iter().enumerate() {
            let before_correct = scores.total_correct();
            let before_total = scores.total_answered();
            let is_correct = i % 2 == 0; // alternate correct / incorrect
            scores.record(q.q_type, is_correct);
            // total_answered increments by 1
            assert_eq!(scores.total_answered(), before_total + 1);
            // total_correct increments by 1 iff correct, else stays same
            if is_correct {
                assert_eq!(scores.total_correct(), before_correct + 1);
            } else {
                assert_eq!(scores.total_correct(), before_correct);
            }
            // invariant: correct <= total at all times
            assert!(scores.total_correct() <= scores.total_answered());
        }
    }
}
