# Pomp

Experimenting on a real time online board game inspired by Splendor.

## Running

This still needs improvement.

    swc static/main.ts -o target/main.js
    elm-live src/frontend/Main.elm --start-page=static/index.html --hot -- --output=target/elm.js
    cargo watch -x 'run --bin pomp'

# Game Rules

If you are familiar with [Splendor](https://boardgamegeek.com/boardgame/148228/splendor)
you'll already know the core mechanics. The goal is to collect 15 _victory points_
first to win the game.

You get _victory points_ by buying _cards_ from the _market_. These cards are paid
with _elements_ from your hand. You start out without any elements but can take
those in exchange for energy. Energy slowly aquires over time for all the players
over time.

Cards also have an element color which gives you a discount on all other cards
you buy which have the same element cost.

It is usually best to buy cards from the first row first, as they are cheapest
and help you build up a discount for other cards.

## Planned features

- Chaos should become an (expensive) wildcard discount not avaliable for energy.
- Gamble will be an option to buy random elements for a discount. It should
  slowly aquire value until some player buys it.
- Special power cards like "faster energy generation", "element generation".
  These start out way overpriced and slowly become less expensive over time.

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

### UserUuid

A secret uuid for each (ephemeral) user that is connected to the system.
This uuid is stored in LocalStorage and persists across reloads.

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
