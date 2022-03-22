//! Helper module to work around a restriction in rust_elm_typegen.
//! Right now, there can't be any non-exportable types in the module.

use std::{collections::HashMap, time::Instant};

use rust_elm_typegen::ElmExport;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Espoused {
    Bride,
    Groom,
}

impl ElmExport for Espoused {}

#[derive(Serialize, Clone)]
pub struct QuestionView {
    pub text: String,
    pub state: QuestionState,
    pub bride_guesses: usize,
    pub groom_guesses: usize,
}

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum QuestionState {
    GuestsCanVote,
    VotingClosed,
    Answered(Espoused),
    ConflictAnswer, // When the Bride and the Groom don't agree on an option.
}

impl QuestionState {
    pub fn can_guess(self) -> bool {
        match self {
            QuestionState::GuestsCanVote => true,
            QuestionState::VotingClosed => false,
            QuestionState::Answered(_) => false,
            QuestionState::ConflictAnswer => false,
        }
    }

    pub fn to_espoused(self) -> Option<Espoused> {
        match self {
            QuestionState::Answered(Espoused::Bride) => Some(Espoused::Bride),
            QuestionState::Answered(Espoused::Groom) => Some(Espoused::Groom),
            _ => None,
        }
    }
}

impl ElmExport for QuestionView {}
impl ElmExport for QuestionState {}

#[derive(Serialize)]
pub enum WeddingView {
    SignUp,
    Guest(GuestView),
    Host(HostView),
    Projector(ProjectorView),
}

#[derive(Serialize)]
pub struct GuestView {
    pub name: String,
    pub question: String,
    pub guess: Option<Espoused>,
    pub state: QuestionState,
    pub score: usize,
}

#[derive(Serialize)]
pub struct HostView {
    pub questions: Vec<QuestionView>,
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

#[derive(Serialize)]
pub struct ProjectorView {
    pub question: Option<QuestionView>,
    pub connected_users: Vec<String>,
}

impl ElmExport for ProjectorView {}
