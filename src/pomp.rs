//! Contains only core game logic for the Pomp game.
//!
use std::{collections::HashMap, time::Duration};

use rand::{distributions::Standard, prelude::Distribution, Rng};
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
    market: Vec<Option<Card>>,
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
    deck_1: Vec<Card>,
    deck_2: Vec<Card>,
    deck_3: Vec<Card>,
    market: Vec<Option<Card>>,
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
        let (mut deck_1, mut deck_2, mut deck_3) = Card::random_deck(inventories.len());

        let mut market = Vec::with_capacity(15);

        for _ in 0..5 {
            market.push(deck_1.pop());
        }
        for _ in 0..5 {
            market.push(deck_2.pop());
        }
        for _ in 0..5 {
            market.push(deck_3.pop());
        }

        GameState {
            players: inventories,
            deck_1,
            deck_2,
            deck_3,
            market,
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
            market: self.market.clone(),
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

#[derive(Debug, Clone, Serialize)]
struct Card {
    id: usize, // We need to tag the card to make it buyable.
    color: ElementColor,
    points: usize,
    fire_cost: usize,
    plant_cost: usize,
    water_cost: usize,
    earth_cost: usize,
    chaos_cost: usize,
}

impl Distribution<ElementColor> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> ElementColor {
        // match rng.gen_range(0, 3) { // rand 0.5, 0.6, 0.7
        match rng.gen_range(0..5) {
            0 => ElementColor::Fire,
            1 => ElementColor::Plant,
            2 => ElementColor::Water,
            3 => ElementColor::Earth,
            4 => ElementColor::Chaos,
            _ => panic!("Invalid element color"),
        }
    }
}

impl Card {
    fn random_deck(player_count: usize) -> (Vec<Card>, Vec<Card>, Vec<Card>) {
        // Each player is allocated 5 cards of level 1.
        // Each player is allocated 10 cards of level 2.
        // Each player is allocated 4 cards of level 3. (4 * 4 = 16 > 15 [win])
        let mut deck_1 = Vec::with_capacity(player_count * 5);
        let mut i = 0;
        for _ in 0..(player_count * 5) {
            deck_1.push(Card::random_level_1(i));
            i += 1;
        }
        let mut deck_2 = Vec::with_capacity(player_count * 10);
        for _ in 0..(player_count * 10) {
            deck_2.push(Card::random_level_2(i));
            i += 1;
        }
        let mut deck_3 = Vec::with_capacity(player_count * 4);
        for _ in 0..(player_count * 4) {
            deck_3.push(Card::random_level_3(i));
            i += 1;
        }
        (deck_1, deck_2, deck_3)
    }

    fn random_level_1(id: usize) -> Card {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..5);
        if x <= 0 {
            // Especially cheap basic card
            Self::fixed_cost_card(id, 0, 3)
        } else if x <= 2 {
            // Basic cost basic card
            Self::fixed_cost_card(id, 0, 4)
        } else if x <= 3 {
            // Cheap card with 1 victory point
            Self::fixed_cost_card(id, 1, 5)
        } else if x <= 4 {
            // cheap card with 2 victory points
            Self::fixed_cost_card(id, 2, 6)
        } else {
            panic!("Invalid card level");
        }
    }

    fn random_level_2(id: usize) -> Card {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..5);
        if x <= 0 {
            Self::fixed_cost_card(id, 2, 7)
        } else if x <= 2 {
            Self::fixed_cost_card(id, 2, 8)
        } else if x <= 3 {
            Self::fixed_cost_card(id, 3, 9)
        } else if x <= 4 {
            Self::fixed_cost_card(id, 3, 10)
        } else {
            panic!("Invalid card level");
        }
    }

    fn random_level_3(id: usize) -> Card {
        let mut rng = rand::thread_rng();
        let x = rng.gen_range(0..5);
        if x <= 0 {
            Self::fixed_cost_card(id, 4, 11)
        } else if x <= 2 {
            Self::fixed_cost_card(id, 4, 12)
        } else if x <= 3 {
            Self::fixed_cost_card(id, 5, 13)
        } else if x <= 4 {
            Self::fixed_cost_card(id, 5, 14)
        } else {
            panic!("Invalid card level");
        }
    }

    /// Randomly distributes a fixed cost across the card.
    /// The color of the card is also random.
    /// The victory points are given
    fn fixed_cost_card(id: usize, points: usize, cost: usize) -> Card {
        let mut card = Card {
            id,
            color: rand::random(),
            points,
            fire_cost: 0,
            plant_cost: 0,
            water_cost: 0,
            earth_cost: 0,
            chaos_cost: 0,
        };

        for _ in 0..cost {
            card.random_inc();
        }

        debug_assert_eq!(card.total_cost(), cost);
        card
    }

    fn random_inc(&mut self) {
        let color: ElementColor = rand::random();
        match color {
            ElementColor::Fire => self.fire_cost += 1,
            ElementColor::Plant => self.plant_cost += 1,
            ElementColor::Water => self.water_cost += 1,
            ElementColor::Earth => self.earth_cost += 1,
            ElementColor::Chaos => self.chaos_cost += 1,
        }
    }

    fn total_cost(&self) -> usize {
        self.fire_cost + self.plant_cost + self.water_cost + self.earth_cost + self.chaos_cost
    }
}

// Test module
#[cfg(test)]
mod test {
    use super::*;

    /// Tests that no debug asserts fail when randomly generating a card.
    #[test]
    fn test_card_random() {
        let mut rng = rand::thread_rng();
        for i in 0..100 {
            Card::fixed_cost_card(i, rng.gen_range(0..10), rng.gen_range(1..10));
            Card::random_level_1(i + 100);
        }
        Card::random_deck(2);
        Card::random_deck(3);
        Card::random_deck(4);
        Card::random_deck(5);
    }
}
