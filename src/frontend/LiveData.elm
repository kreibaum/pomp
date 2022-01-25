module LiveData exposing (..)

{-| TODO: This file should be automatically generated based on Rust code.
when that is done, it also needs to move to /generated instead of /src/frontend.
-}

import Json.Decode
import Json.Encode exposing (Value)
import PompData exposing (PompEvent, encodePompEvent)


decodeLiveStateOneRouteOnly : String -> Json.Decode.Decoder page -> (page -> wrapped) -> Json.Decode.Decoder wrapped
decodeLiveStateOneRouteOnly routeId decoder wrapper =
    Json.Decode.at [ "route" ] Json.Decode.string
        |> Json.Decode.andThen
            (\x ->
                if x == routeId then
                    Json.Decode.at [ "data" ] decoder
                        |> Json.Decode.map wrapper

                else
                    Json.Decode.fail ("Not annotated as 'route':" ++ routeId)
            )


{-| Overall remote event. Required for now until I figure out better typing.
-}
type RemoteEvent
    = PompRemoteEventWrapper PompEvent
    | SetupRemoteEventWrapper SetupRemoteEvent


encodeRemoteEvent : RemoteEvent -> Value
encodeRemoteEvent e =
    case e of
        -- TODO: Attach the curret route we are on so we can drop irrelevant messages on the server.
        PompRemoteEventWrapper x ->
            encodePompEvent x

        SetupRemoteEventWrapper x ->
            encodeSetupRemoteEvent x


{-| Elm version of

    struct PlayerSetupData {
        is_ready: bool,
        name: String,
    }

    struct LiveState {
        data: Vec<PlayerSetupData>,
        my_index: isize,
    }

-}
type alias SetupLiveState =
    { data : List PlayerSetupData
    , myIndex : Int
    }


decodeSetupLiveState : Json.Decode.Decoder SetupLiveState
decodeSetupLiveState =
    Json.Decode.map2 SetupLiveState
        (Json.Decode.at [ "data" ] (Json.Decode.list decodePlayerSetupData))
        (Json.Decode.at [ "my_index" ] Json.Decode.int)


type alias PlayerSetupData =
    { isReady : Bool
    , name : String
    }


decodePlayerSetupData : Json.Decode.Decoder PlayerSetupData
decodePlayerSetupData =
    Json.Decode.map2 PlayerSetupData
        (Json.Decode.at [ "is_ready" ] Json.Decode.bool)
        (Json.Decode.at [ "name" ] Json.Decode.string)


{-| Elm version of

    enum RemoteEvent {
        SetName(String),
        SetReady(bool),
        StartGame,
    }

-}
type SetupRemoteEvent
    = SetName String
    | SetReady Bool
    | StartGame


encodeSetupRemoteEvent : SetupRemoteEvent -> Value
encodeSetupRemoteEvent e =
    case e of
        SetName name ->
            Json.Encode.object [ ( "SetName", Json.Encode.string name ) ]

        SetReady ready ->
            Json.Encode.object [ ( "SetReady", Json.Encode.bool ready ) ]

        StartGame ->
            Json.Encode.string "StartGame"
