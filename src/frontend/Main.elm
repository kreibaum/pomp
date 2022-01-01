module Main exposing (main)

import Browser
import FontAwesome.Styles
import Html exposing (Html)
import Json.Decode
import LiveData
import Page.Loading
import Page.Pomp
import Page.Setup
import Websocket


{-| For now we don't have flags, but eventually that is going to pass in the
inital remote data for us.
-}
type alias Flags =
    ()


type alias Model =
    { liveState : Maybe LiveState
    }


type Msg
    = RemoteEventBox LiveData.RemoteEvent
      --| ClientMsg msg
    | NewServerState LiveState
    | BrokenLiveState String


main : Program Flags Model Msg
main =
    Browser.element
        { init = \_ -> ( { liveState = Nothing }, Cmd.none )
        , update = update
        , view = \model -> Html.div [] [ FontAwesome.Styles.css, view model ]
        , subscriptions = \_ -> Websocket.subscribe sandboxLiveStateParser
        }


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        RemoteEventBox remoteEvent ->
            ( model, Websocket.send (LiveData.encodeRemoteEvent remoteEvent) )

        NewServerState liveState ->
            ( { liveState = Just liveState
              }
            , Cmd.none
            )

        BrokenLiveState errorMessage ->
            Debug.log "Got a broken liveState from the server" errorMessage
                |> (\_ -> ( model, Cmd.none ))


view : Model -> Html Msg
view model =
    case model.liveState of
        Nothing ->
            Page.Loading.view

        Just (PompLiveStateWrapper liveState) ->
            Page.Pomp.view liveState |> Html.map (LiveData.PompRemoteEventWrapper >> RemoteEventBox)

        Just (SetupLiveStateWrapper liveState) ->
            Page.Setup.view liveState |> Html.map (LiveData.SetupRemoteEventWrapper >> RemoteEventBox)


sandboxLiveStateParser : Json.Decode.Value -> Msg
sandboxLiveStateParser value =
    case Json.Decode.decodeValue decodeLiveState value of
        Ok server ->
            NewServerState server

        Err e ->
            BrokenLiveState (Json.Decode.errorToString e)


{-| Overall live state. Should be automatically generated eventually.
-}
type LiveState
    = PompLiveStateWrapper LiveData.PompLiveState
    | SetupLiveStateWrapper LiveData.SetupLiveState


{-| Overall parser that looks at the "route" element first to decide which type
to decode into and then decodes the "data" accordingly.
-}
decodeLiveState : Json.Decode.Decoder LiveState
decodeLiveState =
    Json.Decode.oneOf
        [ LiveData.decodeLiveStateOneRouteOnly "pomp" LiveData.decodePompLiveState PompLiveStateWrapper
        , LiveData.decodeLiveStateOneRouteOnly "setup" LiveData.decodeSetupLiveState SetupLiveStateWrapper
        ]
