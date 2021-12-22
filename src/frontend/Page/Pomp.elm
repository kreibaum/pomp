module Page.Pomp exposing (view)

import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import LiveData exposing (..)


view : PompLiveState -> Html PompRemoteEvent
view model =
    div []
        [ div [] [ text ("You have " ++ String.fromInt model.energy ++ " energy.") ]
        , button [ onClick (Buy Fire) ] [ text ("Buy Fire (" ++ String.fromInt model.fire ++ ")") ]
        , button [ onClick (Buy Plant) ] [ text ("Buy Plant (" ++ String.fromInt model.plant ++ ")") ]
        , button [ onClick (Buy Water) ] [ text ("Buy Water (" ++ String.fromInt model.water ++ ")") ]
        , button [ onClick (Buy Earth) ] [ text ("Buy Earth (" ++ String.fromInt model.earth ++ ")") ]
        , button [ onClick (Buy Chaos) ] [ text ("Buy Chaos (" ++ String.fromInt model.chaos ++ ")") ]
        ]
