module Page.Pomp exposing (view)

import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import LiveData exposing (..)


view : PompLiveState -> Html PompRemoteEvent
view model =
    div []
        [ viewMyInventory model.myInventory
        , div [] [ text " --- " ]
        , viewOthers model.others
        ]


viewMyInventory : PompMyInventory -> Html PompRemoteEvent
viewMyInventory inventory =
    div []
        [ div [] [ text ("You have " ++ String.fromInt inventory.energy ++ " energy.") ]
        , button [ onClick (Buy Fire) ] [ text ("Buy Fire (" ++ String.fromInt inventory.fire ++ ")") ]
        , button [ onClick (Buy Plant) ] [ text ("Buy Plant (" ++ String.fromInt inventory.plant ++ ")") ]
        , button [ onClick (Buy Water) ] [ text ("Buy Water (" ++ String.fromInt inventory.water ++ ")") ]
        , button [ onClick (Buy Earth) ] [ text ("Buy Earth (" ++ String.fromInt inventory.earth ++ ")") ]
        , button [ onClick (Buy Chaos) ] [ text ("Buy Chaos (" ++ String.fromInt inventory.chaos ++ ")") ]
        ]


viewOthers : List PompOthersInventory -> Html a
viewOthers others =
    div []
        (List.map viewOther others)


viewOther : PompOthersInventory -> Html a
viewOther other =
    div []
        [ div [] [ text (other.name ++ ": " ++ String.fromInt other.energy ++ " energy, ?? victory points.") ]
        , text ("Fire (" ++ String.fromInt other.fire ++ ")")
        , text ("Plant (" ++ String.fromInt other.plant ++ ")")
        , text ("Water (" ++ String.fromInt other.water ++ ")")
        , text ("Earth (" ++ String.fromInt other.earth ++ ")")
        , text ("Chaos (" ++ String.fromInt other.chaos ++ ")")
        ]
