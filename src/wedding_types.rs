//! Helper module to work around a restriction in rust_elm_typegen.
//! Right now, there can't be any non-exportable types in the module.

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
    pub answer: Option<Espoused>,
}

impl ElmExport for Question {}

#[derive(Serialize)]
pub enum WeddingView {
    SignUp,
    Guest {
        name: String,
        question: String,
        answer: Option<Espoused>,
    },
    Host {
        questions: Vec<Question>,
        current_question: Option<usize>,
    },
}

impl ElmExport for WeddingView {}

#[derive(Deserialize)]
pub enum WeddingEvent {
    SetName(String),    // Also used to determine who is projector and host.
    SetGuess(Espoused), // Guests can only "guess", the host can "answer".
    SetQuestion(Option<usize>),
}

impl ElmExport for WeddingEvent {}
