module Main exposing (main)

import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Json.Decode
import Json.Encode exposing (Value)
import LiveProgram exposing (LiveProgram)


main : LiveProgram LiveState RemoteEvent
main =
    LiveProgram.static
        { view = view
        , encodeRemoteEvent = encodeRemoteEvent
        , decodeServer = decodeLiveState
        , dummyServer = dummyServer
        }


type alias LiveState =
    { energy : Int
    , fire : Int
    , plant : Int
    , water : Int
    , earth : Int
    , chaos : Int
    }


dummyServer : LiveState
dummyServer =
    { energy = 0
    , fire = 0
    , plant = 0
    , water = 0
    , earth = 0
    , chaos = 0
    }


type ElementColor
    = Fire
    | Plant
    | Water
    | Earth
    | Chaos


type RemoteEvent
    = Buy ElementColor


encodeRemoteEvent : RemoteEvent -> Value
encodeRemoteEvent e =
    case e of
        Buy color ->
            Json.Encode.object [ ( "Buy", encodeElementColor color ) ]


encodeElementColor : ElementColor -> Value
encodeElementColor e =
    case e of
        Fire ->
            Json.Encode.string "Fire"

        Plant ->
            Json.Encode.string "Plant"

        Water ->
            Json.Encode.string "Water"

        Earth ->
            Json.Encode.string "Earth"

        Chaos ->
            Json.Encode.string "Chaos"


decodeLiveState : Json.Decode.Decoder LiveState
decodeLiveState =
    Json.Decode.map6 LiveState
        (Json.Decode.at [ "data", "energy" ] Json.Decode.int)
        (Json.Decode.at [ "data", "fire" ] Json.Decode.int)
        (Json.Decode.at [ "data", "plant" ] Json.Decode.int)
        (Json.Decode.at [ "data", "water" ] Json.Decode.int)
        (Json.Decode.at [ "data", "earth" ] Json.Decode.int)
        (Json.Decode.at [ "data", "chaos" ] Json.Decode.int)


view : LiveState -> Html RemoteEvent
view model =
    div []
        [ div [] [ text ("You have " ++ String.fromInt model.energy ++ " energy.") ]
        , button [ onClick (Buy Fire) ] [ text ("Buy Fire (" ++ String.fromInt model.fire ++ ")") ]
        , button [ onClick (Buy Plant) ] [ text ("Buy Plant (" ++ String.fromInt model.plant ++ ")") ]
        , button [ onClick (Buy Water) ] [ text ("Buy Water (" ++ String.fromInt model.water ++ ")") ]
        , button [ onClick (Buy Earth) ] [ text ("Buy Earth (" ++ String.fromInt model.earth ++ ")") ]
        , button [ onClick (Buy Chaos) ] [ text ("Buy Chaos (" ++ String.fromInt model.chaos ++ ")") ]
        ]
