//! This module holds all the general framework types that should be used in user code.

use std::{any::Any, fmt::Display, time::Duration};

use serde::Serialize;

/** Identifier for players, this way we can play without accounts. */
#[derive(Hash, Eq, PartialEq, Debug, Clone)]
pub struct UserUuid(String);

impl Display for UserUuid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UserUuid {
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
                return Some(UserUuid(uuid.as_str().to_owned()));
            }
        }

        None
    }
}

/// We don't want to expose the full actual `LiveState` or `SharedLiveState` to the
/// client. Instead both of these must be turned into a `UserView` before being
/// send to the client in a websocket message.
pub trait UserView: Serialize + Send {}

pub trait RemoteEvent: Sized {
    /// TODO: This is ugly, should use serde directly somehow.
    fn deserialize(s: &str) -> Result<Self, serde_json::Error>;
}

/// Variation of a `LiveState` that is shared between users.
pub trait SharedLiveState: Default + Unpin + Any + 'static {
    // Each Game has a type of remote event that it handles.
    type Event: RemoteEvent;
    // As well as a type of live state that it sends to the frontend.
    type View: UserView;

    // Map to the live state that is sent to the frontend.
    fn user_view(&self, player: &UserUuid) -> Self::View;

    // Handle events. After every event the current state is send to all clients
    // so there is no need to think about this in this method.
    fn process_remote_event(&mut self, event: Self::Event, sender: UserUuid) -> LiveEffect;

    /// Define how often this live state should process a tick.
    /// If you don't define it, you don't need to process ticks at all.
    fn tick_frequency(&self) -> Option<Duration> {
        None
    }

    /// Called every tick. If you don't set a tick frequency, this is never called.
    /// If you don't define this, it does nothing.
    fn process_tick(&mut self) -> LiveEffect {
        LiveEffect::None
    }

    // Add a player to the game.
    // This has a live effect, because a player may join a "preparation" page
    // a little bit too late. Then they would be redirected into the active
    // game as a spectator.
    fn join_user(&mut self, player: UserUuid) -> LiveEffect;

    // ID used to differentiate this game from others.
    // Will probably replace this by better routing later.
    fn route_id() -> &'static str;
}

/// Encapsulate possible effects that the game implementation can trigger into
/// the live state system.
#[must_use]
pub enum LiveEffect {
    None,                                          // Equivalent to Cmd.none from Elm.
    LiveRedirectInit(String, Box<dyn Any + Send>), // Not sure if "Any" can be avoided here.
    LiveRedirect(String),                          // Like LiveRedirectInit, but without setup data.
}
