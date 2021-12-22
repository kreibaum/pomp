module Page.Loading exposing (view)

{-| This module contains the view that is shown if there is no data from the
Websocket yet.
-}

import Html exposing (Html, text)


view : Html a
view =
    text "Loading..."
