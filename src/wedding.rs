//! Shoe game implementation.
//!
//! This is mixed in with the other stuff for now until I can figure out a
//! library abstraction.

use std::collections::{HashMap, HashSet};

use crate::wedding_types::*;

use crate::game::{LiveEffect, RemoteEvent, SharedLiveState, UserUuid, UserView};

pub struct WeddingData {
    players: HashMap<UserUuid, PlayerData>,
    hosts: HashSet<UserUuid>,
    questions: Vec<Question>,
    current_question: Option<usize>,
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
        }
    }
}

impl Question {
    fn new(text: &'static str) -> Self {
        Question {
            text: text.to_owned(),
            answer: None,
        }
    }
}

/// Stores name, score and other data for a player.
struct PlayerData {
    name: String,
    answer: Option<Espoused>,
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
            WeddingView::Host {
                questions: self.questions.clone(),
                current_question: self.current_question,
            }
        } else if let Some(player_data) = player_data {
            WeddingView::Guest {
                name: player_data.name.clone(),
                question: if self.current_question.is_some() {
                    self.questions[self.current_question.unwrap()].text.clone()
                } else {
                    "Gleich geht es weiter!".to_string()
                },
                answer: player_data.answer,
            }
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
                        self.players.insert(
                            sender,
                            PlayerData {
                                name: new_name,
                                answer: None,
                            },
                        );
                    }
                }
            }
            WeddingEvent::SetGuess(espoused) => {
                if let Some(p) = self.players.get_mut(&sender) {
                    p.answer = Some(espoused);
                }
            }
            WeddingEvent::SetQuestion(id) => {
                self.current_question = id;
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
