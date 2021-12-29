# Pomp

Experimenting on a real time online board game inspired by Splendor.

## Running

This still needs improvement.

    swc static/main.ts -o target/main.js
    elm-live src/frontend/Main.elm --start-page=static/index.html --hot -- --output=target/elm.js
    cargo watch -x 'run --bin pomp'

# Tech Stack

Right now looking at an Rust Actix backend and an Elm Frontend that is designed
with tailwind-css.

# Architecture

I am aiming for something that is maybe best called _LiveState_. I am taking
some ideas from [Phoenix LiveView](https://www.phoenixframework.org/) but still
retain a distinct frontend written in [Elm](elm-lang.org/) which does the rendering.

## Data

### LiveState

State shared between the server and the client. The rust code is the single
source of truth for the shared types and the routing. Elm code is generated
from this.

### ClientState

In addition to the _LiveState_, the client also holds a (per page) _ClientState_
that is not known to the server. In here we would for example handle text input
fields where we don't want to send each Keystroke to the server directly.

In simple cases there is no _ClientState_ which saves us a bunch of boilerplate
on the client.

### PlayerUuid

A secret uuid for each (ephemeral) user that is connected to the system.
This uuid is stored in LocalStorage and persists across reloads.

Needs to be renamed to something less game centric.

## Actors

### WebsocketActor

Each connection to the page spins up a websocket connetion through a websocket
actor. This actor persists while the user is navigating across multiple pages.
The _WebsocketActor_ is really a framework part so it should not be seen directly
when implementing routes.

### LiveActor

A live actor owns the _LiveState_ for a page that is currently open in the client.
Changes to the _LiveState_ must happen in this actor. Usually by reacting to
a _RemoteEvent_.

### SharedLiveActor

Also called _GameActor_ this is basically a _LiveActor_ that has a common state.
There may be secrets that can not be shown to all the clients, this common state
is restricted for each player.
