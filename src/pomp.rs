//! Contains only core game logic for the Pomp game.
//!
use std::{collections::HashMap, mem, time::Duration};

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
    points: u32,
    energy: u32,
    elements: ElementVector,
    discount: ElementVector,
}

/// There are a lot of places where we need one number for each element.
/// This is a helper struct to make it easier to do that.
#[derive(Debug, Default, Clone, Serialize)]
struct ElementVector {
    fire: u32,
    plant: u32,
    water: u32,
    earth: u32,
    chaos: u32,
}

impl ElementVector {
    /// Increase value for the given element by the given amount.
    fn add_element_ip(&mut self, element: ElementColor, value: u32) {
        match element {
            ElementColor::Fire => self.fire += value,
            ElementColor::Plant => self.plant += value,
            ElementColor::Water => self.water += value,
            ElementColor::Earth => self.earth += value,
            ElementColor::Chaos => self.chaos += value,
        }
    }

    /// Calculate the total value of all elements.
    fn total(&self) -> u32 {
        self.fire + self.plant + self.water + self.earth + self.chaos
    }

    /// Returns if this vector is pointwise greater or equal than the other.
    /// Not that this only defines a partial ordering.
    fn geq(&self, other: &Self) -> bool {
        self.fire >= other.fire
            && self.plant >= other.plant
            && self.water >= other.water
            && self.earth >= other.earth
            && self.chaos >= other.chaos
    }

    /// Substracts a smaller vector from this vector.
    /// Only call this when you already know that this >= other.
    fn minus_ip(&mut self, subtrahend: &Self) {
        assert!(self.geq(subtrahend));
        self.fire -= subtrahend.fire;
        self.plant -= subtrahend.plant;
        self.water -= subtrahend.water;
        self.earth -= subtrahend.earth;
        self.chaos -= subtrahend.chaos;
    }

    /// Version of minus where it is assumed that we may go into negative numbers
    /// and those are converted to 0. Useful for discounts.
    fn restricted_minus(minuend: &Self, subtrahend: &Self) -> Self {
        Self {
            fire: minuend.fire.saturating_sub(subtrahend.fire),
            plant: minuend.plant.saturating_sub(subtrahend.plant),
            water: minuend.water.saturating_sub(subtrahend.water),
            earth: minuend.earth.saturating_sub(subtrahend.earth),
            chaos: minuend.chaos.saturating_sub(subtrahend.chaos),
        }
    }

    // Return the pointwise sum of the two vectors.
    fn sum(vector1: &ElementVector, vector2: &ElementVector) -> ElementVector {
        ElementVector {
            fire: vector1.fire + vector2.fire,
            plant: vector1.plant + vector2.plant,
            water: vector1.water + vector2.water,
            earth: vector1.earth + vector2.earth,
            chaos: vector1.chaos + vector2.chaos,
        }
    }
}

impl PlayerInventoryView {
    fn public_info(inv: &PlayerData) -> PlayerInventoryView {
        PlayerInventoryView {
            name: inv.name.clone(),
            points: inv.points,
            energy: inv.energy,
            elements: inv.elements.clone(),
            discount: inv.discount.clone(),
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
    points: u32,
    elements: ElementVector,
    discount: ElementVector,
}

impl PlayerData {
    fn new(name: String) -> Self {
        Self {
            name,
            enery_fraction_ticks: 0,
            energy: 0,
            points: 0,
            elements: ElementVector::default(),
            discount: ElementVector::default(),
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

        debug_assert_eq!(market.len(), 15);

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
    BuyCard(usize),
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

        let my_inventory = PlayerInventoryView::public_info(my_data);

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
            PompEvent::BuyCard(id) => {
                // First, check if this card is currently on the market.
                // If it isn't there, this can be a timing issue where two players
                // try to buy the same card at the same time.
                let inventory = self.players.get_mut(&sender).unwrap();
                let mut market_index = 999;
                for (i, c) in self.market.iter().enumerate() {
                    if let Some(c) = c {
                        if c.id == id
                            && ElementVector::sum(&inventory.elements, &inventory.discount)
                                .geq(&c.cost)
                        {
                            market_index = i;
                            break;
                        }
                    }
                }
                if market_index == 999 {
                    // This card is not on the market or not affordable.
                    return LiveEffect::None;
                }
                let mut new_card = if market_index < 5 {
                    self.deck_1.pop()
                } else if market_index < 10 {
                    self.deck_2.pop()
                } else {
                    self.deck_3.pop()
                };
                mem::swap(&mut self.market[market_index], &mut new_card);
                let new_card = new_card.unwrap();
                inventory
                    .elements
                    .minus_ip(&ElementVector::restricted_minus(
                        &new_card.cost,
                        &inventory.discount,
                    ));
                inventory.discount.add_element_ip(new_card.color, 1);
                inventory.points += new_card.points;
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
            self.elements.add_element_ip(color, 1);
        }
    }
}

#[derive(Debug, Clone, Serialize)]
struct Card {
    id: usize, // We need to tag the card to make it buyable.
    color: ElementColor,
    points: u32,
    cost: ElementVector,
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
            // card with 1 victory point
            Self::fixed_cost_card(id, 1, 6)
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
    fn fixed_cost_card(id: usize, points: u32, cost: usize) -> Card {
        let mut card = Card {
            id,
            color: rand::random(),
            points,
            cost: ElementVector::default(),
        };

        for _ in 0..cost {
            card.random_inc();
        }

        debug_assert_eq!(card.cost.total(), cost as u32);
        card
    }

    fn random_inc(&mut self) {
        let color: ElementColor = rand::random();
        self.cost.add_element_ip(color, 1);
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
