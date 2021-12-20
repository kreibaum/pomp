use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::game::PlayerUuid;

/// Contains only core game logic for the Pomp game.
/// But since "what can a player see" is game logic, the LiveState type is also
/// defined in this module. (At least until we put any additional data into it
/// that is not game logic.)

/// Shared state for one player
#[derive(Debug, Default, Clone, Serialize)]
pub struct LiveState {
    count: i32,
    private_count: i32,
    time_elapsed: u64,
}

/// Total state of the whole game.
#[derive(Debug, Default)]
pub struct GameState {
    players: HashSet<PlayerUuid>,
    count: i32,
    player_private_count: HashMap<PlayerUuid, i32>,
    time_elapsed: u64,
}

/// RemoteEvent custom type. This depents on the business logic we have.
#[derive(Debug, Clone, Deserialize)]
pub enum RemoteEvent {
    Increment,
    Decrement,
}

impl GameState {
    /// Extract information that is relevant for one player and hide the rest.
    pub fn restrict(&self, player: &PlayerUuid) -> LiveState {
        LiveState {
            count: self.count,
            private_count: self.player_private_count.get(player).unwrap_or(&0).clone(),
            time_elapsed: self.time_elapsed,
        }
    }

    /// Processes a game logic tick.
    pub fn process_tick(&mut self) {
        self.time_elapsed += 1;
    }

    /// Process a remote event.
    pub fn process_remote_event(&mut self, event: RemoteEvent, sender: PlayerUuid) {
        match event {
            RemoteEvent::Increment => {
                self.count += 1;
                self.player_private_count
                    .entry(sender)
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
            }
            RemoteEvent::Decrement => {
                self.count -= 1;
                self.player_private_count
                    .entry(sender)
                    .and_modify(|v| *v -= 1)
                    .or_insert(-1);
            }
        }
    }

    /// Adds a player to the game.
    /// TODO: This should be done in some pre-game setup phase.
    /// For now is is easy enought that we can just mix it in here.
    pub fn join_player(&mut self, player: PlayerUuid) {
        self.players.insert(player);
    }
}
