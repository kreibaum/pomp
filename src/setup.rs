use serde::{Deserialize, Serialize};

use crate::{
    game::{GameStateTrait, LiveEffect, LiveStateTrait, PlayerUuid, RemoteEventTrait},
    pomp,
};
/// Setting up a game of pomp. When you are done, you can forward all the
/// connected players to the pomp LiveState.

#[derive(Debug)]
pub struct GameState {
    // This is intentionally not a HashMap, because we need an ordering.
    pub data: Vec<(PlayerUuid, PlayerSetupData)>,
}

impl Default for GameState {
    fn default() -> Self {
        GameState { data: Vec::new() }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PlayerSetupData {
    is_ready: bool,
    name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct LiveState {
    data: Vec<PlayerSetupData>,
    my_index: isize,
}

impl LiveStateTrait for LiveState {}

/// RemoteEvent custom type. This depents on the business logic we have.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum RemoteEvent {
    SetName(String),
    SetReady(bool),
    StartGame,
}

impl RemoteEventTrait for RemoteEvent {
    fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl GameStateTrait for GameState {
    type L = LiveState;
    type R = RemoteEvent;

    /// Process a remote event.
    fn process_remote_event(&mut self, event: RemoteEvent, sender: PlayerUuid) -> LiveEffect {
        let data = self.data.iter_mut().find(|(uuid, _)| uuid == &sender);
        if let Some(data) = data {
            match event {
                RemoteEvent::SetName(name) => data.1.name = name,
                RemoteEvent::SetReady(ready) => data.1.is_ready = ready,
                RemoteEvent::StartGame => {
                    let game = pomp::GameState::from_setup(self);
                    return LiveEffect::LiveRedirect(Box::new(game));
                }
            }
        }
        return LiveEffect::None;
    }

    /// Extract information that is relevant for one player and hide the rest.
    fn restrict(&self, player: &PlayerUuid) -> LiveState {
        let mut my_index = -1;
        let mut data = Vec::new();
        for (i, (uuid, setup_data)) in self.data.iter().enumerate() {
            if uuid == player {
                my_index = i as isize;
            }
            data.push(setup_data.clone());
        }

        LiveState { data, my_index }
    }

    /// This happens every time a connection is established.
    fn join_player(&mut self, player: PlayerUuid) -> LiveEffect {
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

    fn process_tick(&mut self) -> LiveEffect {
        // Nothing to do, we don't respond to updates.
        // TODO: Make tick frequency configurable by page.
        LiveEffect::None
    }

    fn route_id() -> &'static str {
        "setup"
    }
}

fn random_name() -> String {
    use rand::Rng;

    const SENTIMENT: [&str; 6] = ["Happy", "Sad", "Angry", "Excited", "Bored", "Lonely"];
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
        SENTIMENT[rand::thread_rng().gen_range(0..SENTIMENT.len())],
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
        assert!(random_name().len() > 0);
    }

    #[test]
    /// Helper test to help me write elm encoders correctly.
    fn test_encode_remote_event() {
        let e = RemoteEvent::StartGame;
        let s = serde_json::to_string(&e).unwrap();
        assert_eq!("\"StartGame\"", s);
    }
}
