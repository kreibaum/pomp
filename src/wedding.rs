//! Shoe game implementation.
//!
//! This is mixed in with the other stuff for now until I can figure out a
//! library abstraction.

use std::collections::{HashMap, HashSet};

use crate::wedding_types::*;

use crate::game::{LiveEffect, RemoteEvent, SharedLiveState, UserUuid, UserView};

const BIG_CONSTANT: usize = 99999999;

pub struct WeddingData {
    players: HashMap<UserUuid, PlayerData>,
    hosts: HashSet<UserUuid>,
    questions: Vec<Question>,
    current_question: Option<usize>,
    guesses: HashMap<(UserUuid, usize), Espoused>, // Map of all guesses.
}

impl Default for WeddingData {
    fn default() -> Self {
        WeddingData {
            players: HashMap::new(),
            hosts: HashSet::new(),
            questions: vec![
                Question::new("Wer kann höher springen?"),
                Question::new("Wer kann schneller ein Zelt aufbauen?"),
                Question::new("Wer singt lauter?"),
            ],
            current_question: None,
            guesses: HashMap::new(),
        }
    }
}

impl Question {
    fn new(text: &'static str) -> Self {
        Question {
            text: text.to_owned(),
            state: QuestionState::NotAsked,
        }
    }
}

/// Stores name, score and other data for a player.
struct PlayerData {
    name: String,
}

impl UserView for WeddingView {}

impl RemoteEvent for WeddingEvent {
    fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl SharedLiveState for WeddingData {
    type View = WeddingView;
    type Event = WeddingEvent;

    fn user_view(&self, player: &UserUuid) -> Self::View {
        let player_data = self.players.get(player);
        if self.hosts.contains(player) {
            WeddingView::Host(HostView {
                questions: HostQuestion::transform(&self.questions, &self.guesses),
                current_question: self.current_question,
            })
        } else if let Some(player_data) = player_data {
            // Find current guess of the player in map.
            let key = (
                player.clone(),
                self.current_question.unwrap_or(BIG_CONSTANT),
            );
            let guess = self.guesses.get(&key).cloned();

            WeddingView::Guest(GuestView {
                name: player_data.name.clone(),
                question: if self.current_question.is_some() {
                    self.questions[self.current_question.unwrap()].text.clone()
                } else {
                    "Gleich geht es weiter!".to_string()
                },
                guess,
                state: if self.current_question.is_some() {
                    self.questions[self.current_question.unwrap()].state
                } else {
                    QuestionState::NotAsked
                },
            })
        } else {
            WeddingView::SignUp
        }
    }

    fn process_remote_event(&mut self, event: Self::Event, sender: UserUuid) -> LiveEffect {
        match event {
            WeddingEvent::SetName(new_name) => {
                // SetName also decides what kind of participant you are.
                if new_name == "host" {
                    self.hosts.insert(sender);
                } else {
                    self.hosts.remove(&sender); // Making sure we are not a host anymore.
                    if let Some(p) = self.players.get_mut(&sender) {
                        p.name = new_name;
                    } else {
                        self.players.insert(sender, PlayerData { name: new_name });
                    }
                }
            }
            WeddingEvent::SetGuess(espoused) => {
                // Get current question to check if it is still open
                if let Some(question) = self.current_question {
                    if !self.questions[question].state.can_guess() {
                        return LiveEffect::None;
                    }
                    let key = (
                        sender.clone(),
                        self.current_question.unwrap_or(BIG_CONSTANT),
                    );
                    self.guesses.insert(key, espoused);
                }
            }
            WeddingEvent::SetQuestion(id) => {
                self.current_question = id;
            }
            WeddingEvent::SetQuestionState(id, question_state) => {
                if let Some(question) = self.questions.get_mut(id) {
                    question.state = question_state;
                }
            }
        }
        LiveEffect::None
    }

    fn join_user(&mut self, _player: UserUuid) -> LiveEffect {
        // We only start tracking people after they've set their name.
        LiveEffect::None
    }

    fn route_id() -> &'static str {
        "wedding"
    }
}
