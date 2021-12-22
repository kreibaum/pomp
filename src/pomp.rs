use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::game::{GameStateTrait, LiveStateTrait, PlayerUuid, RemoteEventTrait};

/// Contains only core game logic for the Pomp game.
/// But since "what can a player see" is game logic, the LiveState type is also
/// defined in this module. (At least until we put any additional data into it
/// that is not game logic.)

const TICKS_PER_ENERGY: u8 = 10;

/// Shared state for one player
#[derive(Debug, Default, Clone, Serialize)]
pub struct LiveState {
    energy: u32,
    fire: u32,
    plant: u32,
    water: u32,
    earth: u32,
    chaos: u32,
}

impl LiveStateTrait for LiveState {}

/// Total state of the whole game.
#[derive(Debug, Default)]
pub struct GameState {
    players: HashSet<PlayerUuid>,
    inventories: HashMap<PlayerUuid, PlayerInventory>,
}

#[derive(Debug, Default)]
struct PlayerInventory {
    enery_fraction_ticks: u8,
    energy: u32,
    fire: u32,
    plant: u32,
    water: u32,
    earth: u32,
    chaos: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementColor {
    Fire,
    Plant,
    Water,
    Earth,
    Chaos,
}

/// RemoteEvent custom type. This depents on the business logic we have.
#[derive(Debug, Clone, Deserialize)]
pub enum RemoteEvent {
    Buy(ElementColor),
}

impl RemoteEventTrait for RemoteEvent {
    fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl GameStateTrait for GameState {
    type L = LiveState;
    type R = RemoteEvent;

    /// Extract information that is relevant for one player and hide the rest.
    fn restrict(&self, player: &PlayerUuid) -> LiveState {
        let inventory = self
            .inventories
            .get(player)
            .expect("Player inventory not found");
        LiveState {
            energy: inventory.energy,
            fire: inventory.fire,
            plant: inventory.plant,
            water: inventory.water,
            earth: inventory.earth,
            chaos: inventory.chaos,
        }
    }

    /// Process a remote event.
    fn process_remote_event(&mut self, event: RemoteEvent, sender: PlayerUuid) {
        match event {
            RemoteEvent::Buy(color) => {
                let inventory = self.inventories.get_mut(&sender).unwrap();
                inventory.buy(color);
            }
        }
    }

    /// Adds a player to the game.
    fn join_player(&mut self, player: PlayerUuid) {
        if !self.players.contains(&player) {
            self.players.insert(player.clone());
            self.inventories.insert(player, PlayerInventory::default());
        }
    }

    /// Processes a game logic tick.
    fn process_tick(&mut self) {
        for player in self.players.iter() {
            let inventory = self
                .inventories
                .get_mut(player)
                .expect("Player inventory not found");
            inventory.enery_fraction_ticks += 1;
            if inventory.enery_fraction_ticks >= TICKS_PER_ENERGY {
                inventory.enery_fraction_ticks = 0;
                inventory.energy += 1;
            }
        }
    }

    fn route_id() -> &'static str {
        "pomp"
    }
}

impl PlayerInventory {
    fn buy(&mut self, color: ElementColor) {
        if self.energy >= 1 {
            self.energy -= 1;
            match color {
                ElementColor::Fire => self.fire += 1,
                ElementColor::Plant => self.plant += 1,
                ElementColor::Water => self.water += 1,
                ElementColor::Earth => self.earth += 1,
                ElementColor::Chaos => self.chaos += 1,
            }
        }
    }
}
