//! Contains only core game logic for the Pomp game.
//!
use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    game::{LiveEffect, RemoteEvent, SharedLiveState, UserUuid, UserView},
    setup,
};

/// How many ticks the game progresses before a player gets energy.
const TICKS_PER_ENERGY: u8 = 10;

/// Shared state for one player
#[derive(Debug, Default, Clone, Serialize)]
pub struct PompPlayerView {
    my_inventory: PlayerInventoryView,
    others: Vec<PlayerInventoryView>,
}

#[derive(Debug, Default, Clone, Serialize)]
struct PlayerInventoryView {
    name: String,
    energy: u32,
    fire: u32,
    plant: u32,
    water: u32,
    earth: u32,
    chaos: u32,
}

impl PlayerInventoryView {
    fn public_info(inv: &PlayerData) -> Self {
        Self {
            name: inv.name.clone(),
            energy: inv.energy,
            fire: inv.fire,
            plant: inv.plant,
            water: inv.water,
            earth: inv.earth,
            chaos: inv.chaos,
        }
    }
}

impl UserView for PompPlayerView {}

/// Total state of the whole game.
#[derive(Debug, Default)]
pub struct GameState {
    players: HashMap<UserUuid, PlayerData>,
}

#[derive(Debug)]
struct PlayerData {
    name: String,
    enery_fraction_ticks: u8,
    energy: u32,
    fire: u32,
    plant: u32,
    water: u32,
    earth: u32,
    chaos: u32,
}

impl PlayerData {
    fn new(name: String) -> Self {
        Self {
            name,
            enery_fraction_ticks: 0,
            energy: 0,
            fire: 0,
            plant: 0,
            water: 0,
            earth: 0,
            chaos: 0,
        }
    }
}

impl GameState {
    pub fn from_setup(setup_data: &setup::GameState) -> Self {
        let mut inventories = HashMap::new();
        for (uuid, setup_data) in &setup_data.data {
            inventories.insert(uuid.clone(), PlayerData::new(setup_data.name.clone()));
        }
        GameState {
            players: inventories,
        }
    }
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
pub enum PompEvent {
    Buy(ElementColor),
}

impl RemoteEvent for PompEvent {
    fn deserialize(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }
}

impl SharedLiveState for GameState {
    type View = PompPlayerView;
    type Event = PompEvent;

    /// Extract information that is relevant for one player and hide the rest.
    fn user_view(&self, player: &UserUuid) -> PompPlayerView {
        // My inventory
        let my_data = self
            .players
            .get(player)
            .expect("Player inventory not found");

        let my_inventory = PlayerInventoryView {
            name: my_data.name.clone(),
            energy: my_data.energy,
            fire: my_data.fire,
            plant: my_data.plant,
            water: my_data.water,
            earth: my_data.earth,
            chaos: my_data.chaos,
        };

        let mut others = Vec::with_capacity(self.players.len() - 1);

        for (uuid, data) in self.players.iter() {
            if uuid != player {
                others.push(PlayerInventoryView::public_info(data));
            }
        }

        PompPlayerView {
            my_inventory,
            others,
        }
    }

    /// Process a remote event.
    fn process_remote_event(&mut self, event: PompEvent, sender: UserUuid) -> LiveEffect {
        match event {
            PompEvent::Buy(color) => {
                let inventory = self.players.get_mut(&sender).unwrap();
                inventory.buy(color);
            }
        }
        LiveEffect::None
    }

    fn tick_frequency(&self) -> Option<Duration> {
        // Game Loop runs at 5 fps
        Some(Duration::from_millis(200))
    }

    /// Processes a game logic tick.
    fn process_tick(&mut self) -> LiveEffect {
        for (_player, inventory) in self.players.iter_mut() {
            inventory.enery_fraction_ticks += 1;
            if inventory.enery_fraction_ticks >= TICKS_PER_ENERGY {
                inventory.enery_fraction_ticks = 0;
                inventory.energy += 1;
            }
        }
        LiveEffect::None
    }

    /// Adds a player to the game.
    fn join_user(&mut self, _player: UserUuid) -> LiveEffect {
        // Players can't join the game. This only happens in setup.
        // They turn into spectators. (TODO: Implement spectators)
        LiveEffect::None
    }

    fn route_id() -> &'static str {
        "pomp"
    }
}

impl PlayerData {
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
