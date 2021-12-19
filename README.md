# pomp

Experimenting on a real time online board game inspired by Splendor

## Tech Stack

Right now looking at an Rust Actix backend and an Elm Frontend that is designed
with tailwind-css.

## Running

This still needs improvement.

    swc static/main.ts -o target/main.js
    elm-live src/frontend/Main.elm --start-page=static/index.html --hot -- --output=target/elm.js
    cargo watch -x 'run --bin pomp'
