# Pomp

Experimenting on a real time online board game inspired by Splendor.

## Tech Stack

Right now looking at an Rust Actix backend and an Elm Frontend that is designed
with tailwind-css.

## Architecture

### LiveState

I am aiming for something that is maybe best called _LiveState_. I am taking
some ideas from [Phoenix LiveView](https://www.phoenixframework.org/) but still
retain a distinct frontend written in Elm which does the rendering.

Elm renders the frontend based on the _LiveState_ which is owned by the server
and the _ClientState_ which is client only. Event can be handled in the client
to update the _ClientState_, or they can be send to the server as a _RemoteEvent_.

Each open browser tab has a websocket connection open.
A _LiveActor_ manages this connection and holds the _LiveState_ of the page.
Here we have another update method which can update the _LiveState_ based on.

Because we are in the server, it is easier to update the _LiveState_ of other
_LiveActors_ as well. These changes will directly be pushed their respective
frontends.

In simple cases there is no _ClientState_ which saves us a bunch of boilerplate
on the client.

### LiveGame

Since this repo implements a game, the really interesting state is the full
_GameState_. This is managed by a _GameActor_ for each game in progress.
The _LiveState_ still applies -- but in this case it is just a restricted view
into the _GameState_ that can be shown to the player. The _LiveActor_ does not
actually do all that much interesting stuff here.

## Running

This still needs improvement.

    swc static/main.ts -o target/main.js
    elm-live src/frontend/Main.elm --start-page=static/index.html --hot -- --output=target/elm.js
    cargo watch -x 'run --bin pomp'
