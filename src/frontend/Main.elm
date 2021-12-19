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
        , dummyServer = { count = 0 }
        }


type alias LiveState =
    { count : Int
    }


type RemoteEvent
    = Increment
    | Decrement


encodeRemoteEvent : RemoteEvent -> Value
encodeRemoteEvent e =
    case e of
        Increment ->
            Json.Encode.string "Increment"

        Decrement ->
            Json.Encode.string "Decrement"


decodeLiveState : Json.Decode.Decoder LiveState
decodeLiveState =
    Json.Decode.at [ "data", "count" ] Json.Decode.int
        |> Json.Decode.map LiveState


view : LiveState -> Html RemoteEvent
view model =
    div []
        [ button [ onClick Decrement ] [ text "-" ]
        , div [] [ text (String.fromInt model.count) ]
        , button [ onClick Increment ] [ text "+" ]
        ]
