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
    { liveState : Maybe LiveData.LiveState
    }


type Msg
    = RemoteEventBox LiveData.RemoteEvent
      --| ClientMsg msg
    | NewServerState LiveData.LiveState
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

        Just (LiveData.PompLiveStateWrapper liveState) ->
            Page.Pomp.view liveState |> Html.map (LiveData.PompRemoteEventWrapper >> RemoteEventBox)

        Just (LiveData.SetupLiveStateWrapper liveState) ->
            Page.Setup.view liveState |> Html.map (LiveData.SetupRemoteEventWrapper >> RemoteEventBox)


sandboxLiveStateParser : Json.Decode.Value -> Msg
sandboxLiveStateParser value =
    case Json.Decode.decodeValue LiveData.decodeLiveState value of
        Ok server ->
            NewServerState server

        Err e ->
            BrokenLiveState (Json.Decode.errorToString e)
