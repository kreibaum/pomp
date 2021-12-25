use std::{any::Any, fmt::Display};

use serde::Serialize;

/** Identifier for players, this way we can play without accounts. */
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct PlayerUuid(String);

impl Display for PlayerUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PlayerUuid {
    /** Login happens via /ws?uuid=... this parses the "uuid=..." part for you. */
    pub fn from_query_string(query_string: &str) -> Option<Self> {
        use lazy_static::lazy_static;
        use regex::Regex;
        lazy_static! {
            static ref RE: Regex = Regex::new(
                "UUID=([0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12})"
            )
            .unwrap();
        }

        if let Some(cap) = RE.captures_iter(&query_string.to_uppercase()).next() {
            if let Some(uuid) = cap.get(1) {
                return Some(PlayerUuid(uuid.as_str().to_owned()));
            }
        }

        None
    }
}

pub trait LiveStateTrait: Serialize + Send {}

pub trait RemoteEventTrait: Sized {
    /// TODO: This is ugly, should use serde directly somehow.
    fn deserialize(s: &str) -> Result<Self, serde_json::Error>;
}

pub trait GameStateTrait: Default + Unpin + Any + 'static {
    // Each Game has a type of remote event that it handles.
    type R: RemoteEventTrait;
    // As well as a type of live state that it sends to the frontend.
    type L: LiveStateTrait;

    // Handle events. After every event the current state is send to all clients
    // so there is no need to think about this in this method.
    fn process_remote_event(&mut self, event: Self::R, sender: PlayerUuid) -> LiveEffect;

    // Map to the live state that is sent to the frontend.
    fn restrict(&self, player: &PlayerUuid) -> Self::L;

    // Add a player to the game.
    // This has a live effect, because a player may join a "preparation" page
    // a little bit too late. Then they would be redirected into the active
    // game as a spectator.
    fn join_player(&mut self, player: PlayerUuid) -> LiveEffect;

    // Called every tick.
    fn process_tick(&mut self) -> LiveEffect;

    // ID used to differentiate this game from others.
    // Will probably replace this by better routing later.
    fn route_id() -> &'static str;
}

/// Encapsulate possible effects that the game implementation can trigger into
/// the live state system.
pub enum LiveEffect {
    None,                 // Equivalent to Cmd.none from Elm.
    LiveRedirect(String), // Not sure if "Any" can be avoided here.
}
