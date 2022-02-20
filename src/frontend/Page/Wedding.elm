module Page.Wedding exposing (view)

import Html exposing (Html)
import WeddingData exposing (..)


view : WeddingView -> Html WeddingEvent
view model =
    Html.text "Wedding"
