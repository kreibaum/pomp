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
        , dummyServer = { count = 0, privateCount = 0, timeElapsed = 0 }
        }


type alias LiveState =
    { count : Int
    , privateCount : Int
    , timeElapsed : Int
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
    Json.Decode.map3 LiveState
        (Json.Decode.at [ "data", "count" ] Json.Decode.int)
        (Json.Decode.at [ "data", "private_count" ] Json.Decode.int)
        (Json.Decode.at [ "data", "time_elapsed" ] Json.Decode.int)


view : LiveState -> Html RemoteEvent
view model =
    div []
        [ button [ onClick Decrement ] [ text "-" ]
        , div [] [ text (String.fromInt model.count) ]
        , div [] [ text "," ]
        , div [] [ text (String.fromInt model.privateCount) ]
        , button [ onClick Increment ] [ text "+" ]
        , div [] [ text <| String.fromInt model.timeElapsed ]
        ]
