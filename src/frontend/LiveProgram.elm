module LiveProgram exposing (LiveProgram, static)

{-| A live programm is connected to an actor in the server which keeps its own
state. The server state is syncronized into the client whenever it changes.

This takes some ideas from Phoenix LiveView into my Rust+Elm stack.

-}

import Browser
import Html exposing (Html)
import Json.Decode
import Json.Encode
import Websocket


type alias FullState liveState client =
    { server : liveState
    , client : client
    }


type FullMsg remoteEvent msg liveState
    = RemoteEventBox remoteEvent
    | ClientMsg msg
    | NewServerState liveState
    | BrokenLiveState String


type alias LiveProgram liveState remoteEvent =
    Program () (FullState liveState ()) (FullMsg remoteEvent Never liveState)


static :
    { view : liveState -> Html remoteEvent

    -- These two should be generated automatically eventually.
    , encodeRemoteEvent : remoteEvent -> Json.Encode.Value
    , decodeServer : Json.Decode.Decoder liveState

    -- This should get eliminated by injecting the inital state into the html.
    , dummyServer : liveState
    }
    -> LiveProgram liveState remoteEvent
static data =
    Browser.element
        { init = \_ -> ( { server = data.dummyServer, client = () }, Cmd.none )
        , update = sandboxUpdate data.encodeRemoteEvent
        , view = \model -> data.view model.server |> Html.map RemoteEventBox
        , subscriptions = \_ -> Websocket.subscribe (sandboxLiveStateParser data.decodeServer)
        }


sandboxLiveStateParser :
    Json.Decode.Decoder liveState
    -> Json.Decode.Value
    -> FullMsg a Never liveState
sandboxLiveStateParser decodeServer value =
    case Json.Decode.decodeValue decodeServer value of
        Ok server ->
            NewServerState server

        Err e ->
            BrokenLiveState (Json.Decode.errorToString e)


sandboxUpdate : (remoteEvent -> Json.Encode.Value) -> FullMsg remoteEvent Never liveState -> FullState liveState () -> ( FullState liveState (), Cmd (FullMsg remoteEvent Never liveState) )
sandboxUpdate encodeRemoteEvent msg model =
    case msg of
        NewServerState server ->
            ( { model | server = server }, Cmd.none )

        ClientMsg n ->
            never n

        RemoteEventBox remoteEvent ->
            ( model, Websocket.send (encodeRemoteEvent remoteEvent) )

        BrokenLiveState errorMessage ->
            Debug.log "Got a broken liveState from the server" errorMessage
                |> (\_ -> ( model, Cmd.none ))
