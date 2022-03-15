//! Helper module to work around a restriction in rust_elm_typegen.
//! Right now, there can't be any non-exportable types in the module.

use std::collections::HashMap;

use rust_elm_typegen::ElmExport;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum Espoused {
    Bride,
    Groom,
}

impl ElmExport for Espoused {}

#[derive(Serialize, Clone)]
pub struct Question {
    pub text: String,
    pub state: QuestionState,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum QuestionState {
    GuestsCanVote,
    VotingClosed,
    Answered(Espoused),
}

impl QuestionState {
    pub fn can_guess(self) -> bool {
        match self {
            QuestionState::GuestsCanVote => true,
            QuestionState::VotingClosed => false,
            QuestionState::Answered(_) => false,
        }
    }
}

impl ElmExport for Question {}
impl ElmExport for QuestionState {}

#[derive(Serialize)]
pub enum WeddingView {
    SignUp,
    Guest(GuestView),
    Host(HostView),
}

#[derive(Serialize)]
pub struct GuestView {
    pub name: String,
    pub question: String,
    pub guess: Option<Espoused>,
    pub state: QuestionState,
}

#[derive(Serialize)]
pub struct HostQuestion {
    pub question: Question,
    pub bride_guesses: usize,
    pub groom_guesses: usize,
}

impl ElmExport for HostQuestion {}

impl HostQuestion {
    pub fn transform(
        questions: &[Question],
        guesses: &HashMap<(crate::game::UserUuid, usize), Espoused>,
    ) -> Vec<HostQuestion> {
        // First wrap everything with zero votes
        let mut result = questions
            .iter()
            .map(|question| HostQuestion {
                question: question.clone(),
                bride_guesses: 0,
                groom_guesses: 0,
            })
            .collect::<Vec<_>>();
        // Next, add the votes
        for ((_, question_index), guess) in guesses {
            let question = &mut result[*question_index];
            match guess {
                Espoused::Bride => question.bride_guesses += 1,
                Espoused::Groom => question.groom_guesses += 1,
            }
        }
        result
    }
}

#[derive(Serialize)]
pub struct HostView {
    pub questions: Vec<HostQuestion>,
    pub current_question: Option<usize>,
}

impl ElmExport for WeddingView {}
impl ElmExport for GuestView {}
impl ElmExport for HostView {}

#[derive(Deserialize)]
#[allow(clippy::enum_variant_names)] // Important to have good names also in Elm code.
pub enum WeddingEvent {
    SetName(String),    // Also used to determine who is projector and host.
    SetGuess(Espoused), // Guests can only "guess", the host can "answer".
    SetQuestion(Option<usize>),
    SetQuestionState(usize, QuestionState),
}

impl ElmExport for WeddingEvent {}
