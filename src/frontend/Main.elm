module Main exposing (main)

import Browser
import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
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
    { count : Int }


init : flags -> ( Model, Cmd Msg )
init _ =
    ( { count = 0 }, Cmd.none )


type Msg
    = Increment
    | Decrement
    | Ping
    | Pong Value


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        Increment ->
            ( { model | count = model.count + 1 }, Cmd.none )

        Decrement ->
            ( { model | count = model.count - 1 }, Cmd.none )

        Ping ->
            ( model, Websocket.websocketOut (Json.Encode.string "Hello, World.") )

        Pong value ->
            Debug.log "Pong" value
                |> (\_ -> ( model, Cmd.none ))


view : Model -> Html Msg
view model =
    div []
        [ button [ onClick Decrement ] [ text "-" ]
        , div [] [ text (String.fromInt model.count) ]
        , button [ onClick Increment ] [ text "+" ]
        , button [ onClick Ping ] [ text "Ping" ]
        ]


subscriptions : model -> Sub Msg
subscriptions _ =
    Websocket.websocketIn Pong
