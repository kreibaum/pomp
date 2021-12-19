module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import Json.Decode
import Json.Encode exposing (Value)
import Websocket


main : Program () Model Msg
main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }


type alias Model =
    { server : LiveState }


type alias LiveState =
    { count : Int
    }


type RemoveEvent
    = Increment
    | Decrement


send : RemoveEvent -> Cmd mgs
send e =
    case e of
        Increment ->
            Websocket.send (Json.Encode.string "Increment")

        Decrement ->
            Websocket.send (Json.Encode.string "Decrement")


decodeLiveState : Json.Decode.Decoder LiveState
decodeLiveState =
    Json.Decode.at [ "data", "count" ] Json.Decode.int
        |> Json.Decode.map LiveState


init : flags -> ( Model, Cmd Msg )
init _ =
    -- TODO: Inject the initial server state via flags.
    ( { server = { count = 0 } }, Cmd.none )


type Msg
    = JustSend RemoveEvent
    | Ping
    | Pong Value


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        JustSend event ->
            ( model, send event )

        Ping ->
            ( model, Websocket.send (Json.Encode.string "Hello, World.") )

        Pong value ->
            Json.Decode.decodeValue decodeLiveState value
                -- TODO: Error handling when the server state can't be parsed.
                |> Result.withDefault { count = -666 }
                |> (\liveState ->
                        ( { model | server = liveState }, Cmd.none )
                   )


view : Model -> Html Msg
view model =
    div []
        [ button [ onClick (JustSend Decrement) ] [ text "-" ]
        , div [] [ text (String.fromInt model.server.count) ]
        , button [ onClick (JustSend Increment) ] [ text "+" ]
        , button [ onClick Ping ] [ text "Ping" ]
        ]


subscriptions : model -> Sub Msg
subscriptions _ =
    Websocket.subscribe Pong
