//! Shoe game implementation.
//!
//! This is mixed in with the other stuff for now until I can figure out a
//! library abstraction.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use crate::wedding_types::*;

use crate::game::{LiveEffect, RemoteEvent, SharedLiveState, UserUuid, UserView};

const BIG_CONSTANT: usize = 99999999;

pub struct WeddingData {
    players: HashMap<UserUuid, PlayerName>,
    hosts: HashSet<UserUuid>,
    projectors: HashSet<UserUuid>,
    questions: Vec<Question>,
    current_question: Option<usize>,
    scores: HashMap<UserUuid, usize>, // Map of all scores.
}

impl Default for WeddingData {
    fn default() -> Self {
        WeddingData {
            players: HashMap::new(),
            hosts: HashSet::new(),
            projectors: HashSet::new(),
            questions: vec![
                Question::new("Wer kann höher springen?"),
                Question::new("Wer kann schneller ein Zelt aufbauen?"),
                Question::new("Wer singt lauter?"),
            ],
            current_question: None,
            scores: HashMap::new(),
        }
    }
}

struct Question {
    pub text: String,
    pub state: QuestionState,
    guesses: HashMap<UserUuid, (Espoused, Instant)>,
    bride_guesses: usize, // Cached
    groom_guesses: usize, // Cached
}

impl Question {
    pub fn new(text: &'static str) -> Self {
        Question {
            text: text.to_owned(),
            state: QuestionState::GuestsCanVote,
            guesses: HashMap::new(),
            bride_guesses: 0,
            groom_guesses: 0,
        }
    }

    fn as_view(&self) -> QuestionView {
        QuestionView {
            text: self.text.clone(),
            state: self.state,
            bride_guesses: self.bride_guesses,
            groom_guesses: self.groom_guesses,
        }
    }

    fn set_guess(&mut self, user: UserUuid, guess: Espoused) {
        // Check if the user already guessed. If they have not changed their
        // guess, do nothing.
        if let Some((old_guess, _)) = self.guesses.get(&user) {
            if *old_guess == guess {
                return;
            }
            // If the user changed their guess, remove their old guess from the count.
            if *old_guess == Espoused::Bride {
                self.bride_guesses -= 1;
            } else {
                self.groom_guesses -= 1;
            }
        }
        // Add the new guess to the map and to the count.
        self.guesses.insert(user, (guess, Instant::now()));
        if guess == Espoused::Bride {
            self.bride_guesses += 1;
        } else {
            self.groom_guesses += 1;
        }
    }
}
/// Stores name, score and other data for a player.
struct PlayerName(String);

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
                questions: self.questions.iter().map(|q| q.as_view()).collect(),
                current_question: self.current_question,
            })
        } else if self.projectors.contains(player) {
            // Get the current question.
            let current_question_id = self.current_question.unwrap_or(BIG_CONSTANT);
            let current_question = self.questions.get(current_question_id);
            let current_question_view = current_question.map(|q| q.as_view());

            WeddingView::Projector(ProjectorView {
                question: current_question_view,
                connected_users: self
                    .players
                    .iter()
                    .map(|(_, name)| name.0.clone())
                    .collect(),
            })
        } else if let Some(player_name) = player_data {
            // Get current question
            let current_question_id = self.current_question.unwrap_or(BIG_CONSTANT);
            let score = self.scores.get(player).cloned().unwrap_or(0);
            if let Some(current_question) = self.questions.get(current_question_id) {
                WeddingView::Guest(GuestView {
                    name: player_name.0.clone(),
                    question: current_question.text.clone(),
                    guess: current_question
                        .guesses
                        .get(player)
                        .map(|&(guess, _)| guess),
                    state: current_question.state,
                    score,
                })
            } else {
                WeddingView::Guest(GuestView {
                    name: player_name.0.clone(),
                    question: "Gleich geht es weiter!".to_owned(),
                    guess: None,
                    state: QuestionState::GuestsCanVote,
                    score,
                })
            }
        } else {
            WeddingView::SignUp
        }
    }

    fn process_remote_event(&mut self, event: Self::Event, sender: UserUuid) -> LiveEffect {
        match event {
            WeddingEvent::SetName(new_name) => {
                self.hosts.remove(&sender); // Making sure we are not a host anymore.
                self.projectors.remove(&sender); // Making sure we are not a projector anymore.

                // SetName also decides what kind of participant you are.
                if new_name == "host" {
                    self.hosts.insert(sender);
                } else if new_name == "projector" {
                    self.projectors.insert(sender);
                } else if let Some(p) = self.players.get_mut(&sender) {
                    p.0 = new_name;
                } else {
                    self.players.insert(sender, PlayerName(new_name));
                }
            }
            WeddingEvent::SetGuess(new_guess) => {
                // Get current question to check if it is still open
                if let Some(question) = self.current_question {
                    let question = &mut self.questions[question];
                    if !question.state.can_guess() {
                        return LiveEffect::None;
                    }
                    question.set_guess(sender, new_guess);
                }
            }
            WeddingEvent::SetQuestion(id) => {
                self.current_question = id;
            }
            WeddingEvent::SetQuestionState(id, question_state) => {
                if let Some(question) = self.questions.get_mut(id) {
                    question.state = question_state;
                    self.scores = score_guesses(&self.questions);
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

/// Each question is worth 100 points to the first question that got the right
/// answer. The second person gets 99 points, then 98, etc.
/// If you guess incorrectly, you get 0 points.
fn score_guesses(questions: &[Question]) -> HashMap<UserUuid, usize> {
    let mut scores: HashMap<UserUuid, usize> = HashMap::new();
    for question in questions {
        if let Some(answer) = question.state.to_espoused() {
            // Create a copy of the user guesses and sort by the time they guessed.
            let mut guesses = question.guesses.iter().collect::<Vec<_>>();
            guesses.sort_by_key(|&(_, (_, time))| time);
            // Iterate over the guesses and give points everyone that got it right.
            let mut points_left_to_give = 100;
            for (user, (guess, _)) in guesses {
                if answer == *guess {
                    // Increase score of player or create new entry if they don't exist yet.
                    *scores.entry(user.clone()).or_insert(0) += points_left_to_give;
                    points_left_to_give -= 1;
                }
            }
        }
    }
    scores
}
