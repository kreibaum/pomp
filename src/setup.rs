use serde::{Deserialize, Serialize};

use crate::{
    game::{LiveEffect, RemoteEvent, SharedLiveState, UserUuid, UserView},
    pomp,
};
/// Setting up a game of pomp. When you are done, you can forward all the
/// connected players to the pomp LiveState.
// Define which types get exported to Elm by rust_elm_typegen.
use rust_elm_typegen::ElmExport;
impl ElmExport for SetupPlayerView {}
impl ElmExport for PlayerSetupData {}
impl ElmExport for SetupEvent {}

#[derive(Debug, Default)]
pub struct GameState {
    // This is intentionally not a HashMap, because we need an ordering.
    pub data: Vec<(UserUuid, PlayerSetupData)>,
    is_started: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerSetupData {
    is_ready: bool,
    pub name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SetupPlayerView {
    data: Vec<PlayerSetupData>,
    my_index: isize,
}

impl UserView for SetupPlayerView {}

/// RemoteEvent custom type. This depents on the business logic we have.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum SetupEvent {
    SetName(String),
    SetReady(bool),
    StartGame,
}

impl RemoteEvent for SetupEvent {
    fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl SharedLiveState for GameState {
    type View = SetupPlayerView;
    type Event = SetupEvent;

    /// Extract information that is relevant for one player and hide the rest.
    fn user_view(&self, player: &UserUuid) -> SetupPlayerView {
        let mut my_index = -1;
        let mut data = Vec::new();
        for (i, (uuid, setup_data)) in self.data.iter().enumerate() {
            if uuid == player {
                my_index = i as isize;
            }
            data.push(setup_data.clone());
        }

        SetupPlayerView { data, my_index }
    }

    /// Process a remote event.
    fn process_remote_event(&mut self, event: SetupEvent, sender: UserUuid) -> LiveEffect {
        let data = self.data.iter_mut().find(|(uuid, _)| uuid == &sender);
        if let Some(data) = data {
            match event {
                SetupEvent::SetName(name) => data.1.name = name,
                SetupEvent::SetReady(ready) => data.1.is_ready = ready,
                SetupEvent::StartGame => {
                    if self.is_started {
                        return LiveEffect::None;
                    }
                    self.is_started = true;
                    let game = pomp::GameState::from_setup(self);
                    return LiveEffect::LiveRedirectInit("/pomp/1".to_owned(), Box::new(game));
                }
            }
        }
        LiveEffect::None
    }

    /// This happens every time a connection is established.
    fn join_user(&mut self, player: UserUuid) -> LiveEffect {
        if self.is_started {
            return LiveEffect::LiveRedirect("/pomp/1".to_owned());
        }

        // Check if this uuid is already inside.
        if self.data.iter().any(|(uuid, _)| uuid == &player) {
            return LiveEffect::None;
        }

        self.data.push((
            player,
            PlayerSetupData {
                is_ready: false,
                name: random_name(),
            },
        ));
        // TODO: Check if there is already a game running. If so, redirect the
        // player to the game.
        LiveEffect::None
    }

    fn route_id() -> &'static str {
        "setup"
    }
}

fn random_name() -> String {
    use rand::Rng;

    // Positive adjectives
    const ADJECTIVE: [&str; 21] = [
        "Active",
        "Busy",
        "Creative",
        "Daring",
        "Excited",
        "Focused",
        "Gentle",
        "Happy",
        "Kind",
        "Lovely",
        "Marvelous",
        "Nervous",
        "Opinionated",
        "Passionate",
        "Proud",
        "Relaxed",
        "Sincere",
        "Sneaky",
        "Tender",
        "Unique",
        "Victorious",
    ];
    const COLOR: [&str; 6] = ["Red", "Blue", "Green", "Yellow", "Purple", "Orange"];
    const ANIMAL: [&str; 11] = [
        "Bumblebee",
        "Butterfly",
        "Clownfish",
        "Fireant",
        "Hummingbird",
        "Jellyfish",
        "Kangaroo",
        "Lion",
        "Owl",
        "Penguin",
        "Seahorse",
    ];
    // Combine a random sentiment, color and animal into a single name.
    format!(
        "{} {} {}",
        ADJECTIVE[rand::thread_rng().gen_range(0..ADJECTIVE.len())],
        COLOR[rand::thread_rng().gen_range(0..COLOR.len())],
        ANIMAL[rand::thread_rng().gen_range(0..ANIMAL.len())]
    )
}

// Test module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_random_name() {
        assert!(!random_name().is_empty());
    }

    #[test]
    /// Helper test to help me write elm encoders correctly.
    fn test_encode_remote_event() {
        let e = SetupEvent::StartGame;
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!("\"StartGame\"", s);
    }
}
