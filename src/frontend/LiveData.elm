module LiveData exposing (..)

{-| TODO: This file should be automatically generated based on Rust code.
when that is done, it also needs to move to /generated instead of /src/frontend.
-}

import Json.Decode
import Json.Encode exposing (Value)


{-| Overall live state. Required for now until I figure out better typing.
-}
type LiveState
    = PompLiveStateWrapper PompLiveState
    | SetupLiveStateWrapper SetupLiveState


{-| Overall parser that looks at the "route" element first to decide which type
to decode into and then decodes the "data" accordingly.
-}
decodeLiveState : Json.Decode.Decoder LiveState
decodeLiveState =
    Json.Decode.at [ "route" ] Json.Decode.string
        |> Json.Decode.andThen
            (\x ->
                case x of
                    "pomp" ->
                        Json.Decode.at [ "data" ] decodePompLiveState
                            |> Json.Decode.map PompLiveStateWrapper

                    "setup" ->
                        Json.Decode.at [ "data" ] decodeSetupLiveState
                            |> Json.Decode.map SetupLiveStateWrapper

                    _ ->
                        Json.Decode.fail "Unknown route"
            )


{-| Overall remote event. Required for now until I figure out better typing.
-}
type RemoteEvent
    = PompRemoteEventWrapper PompRemoteEvent
    | SetupRemoteEventWrapper SetupRemoteEvent


encodeRemoteEvent : RemoteEvent -> Value
encodeRemoteEvent e =
    case e of
        -- TODO: Attach the curret route we are on so we can drop irrelevant messages on the server.
        PompRemoteEventWrapper x ->
            encodePompRemoteEvent x

        SetupRemoteEventWrapper x ->
            encodeSetupRemoteEvent x


{-| Elm version of

    struct LiveState {
        energy: u32,
        fire: u32,
        plant: u32,
        water: u32,
        earth: u32,
        chaos: u32,
    }

-}
type alias PompLiveState =
    { energy : Int
    , fire : Int
    , plant : Int
    , water : Int
    , earth : Int
    , chaos : Int
    }


type ElementColor
    = Fire
    | Plant
    | Water
    | Earth
    | Chaos


{-| Elm version of

    enum RemoteEvent {
        Buy(ElementColor),
    }

-}
type PompRemoteEvent
    = Buy ElementColor


encodePompRemoteEvent : PompRemoteEvent -> Value
encodePompRemoteEvent e =
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


decodePompLiveState : Json.Decode.Decoder PompLiveState
decodePompLiveState =
    Json.Decode.map6 PompLiveState
        (Json.Decode.at [ "energy" ] Json.Decode.int)
        (Json.Decode.at [ "fire" ] Json.Decode.int)
        (Json.Decode.at [ "plant" ] Json.Decode.int)
        (Json.Decode.at [ "water" ] Json.Decode.int)
        (Json.Decode.at [ "earth" ] Json.Decode.int)
        (Json.Decode.at [ "chaos" ] Json.Decode.int)


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
            Json.Encode.object [ ( "StartGame", Json.Encode.null ) ]
