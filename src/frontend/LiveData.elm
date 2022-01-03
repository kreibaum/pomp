module LiveData exposing (..)

{-| TODO: This file should be automatically generated based on Rust code.
when that is done, it also needs to move to /generated instead of /src/frontend.
-}

import Json.Decode
import Json.Encode exposing (Value)


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



-- Pomp Live State -------------------------------------------------------------
--------------------------------------------------------------------------------


type alias PompLiveState =
    { myInventory : PompMyInventory
    , others : List PompOthersInventory
    , market : List (Maybe Card)
    }


type alias ElementVector =
    { chaos : Int
    , earth : Int
    , fire : Int
    , plant : Int
    , water : Int
    }


decodeElementVector : Json.Decode.Decoder ElementVector
decodeElementVector =
    Json.Decode.map5 ElementVector
        (Json.Decode.field "chaos" Json.Decode.int)
        (Json.Decode.field "earth" Json.Decode.int)
        (Json.Decode.field "fire" Json.Decode.int)
        (Json.Decode.field "plant" Json.Decode.int)
        (Json.Decode.field "water" Json.Decode.int)


type alias Card =
    { id : Int
    , color : String
    , points : Int
    , cost : ElementVector
    }


type alias PompMyInventory =
    { name : String
    , energy : Int
    , elements : ElementVector
    }


type alias PompOthersInventory =
    { name : String
    , energy : Int
    , elements : ElementVector
    }


decodePompLiveState : Json.Decode.Decoder PompLiveState
decodePompLiveState =
    Json.Decode.map3 PompLiveState
        (Json.Decode.field "my_inventory" decodeRootMyInventory)
        (Json.Decode.field "others" <| Json.Decode.list decodeRootOthersObject)
        (Json.Decode.field "market" <| Json.Decode.list (Json.Decode.maybe decodeCardObject))


decodeCardObject : Json.Decode.Decoder Card
decodeCardObject =
    Json.Decode.map4 Card
        (Json.Decode.field "id" Json.Decode.int)
        (Json.Decode.field "color" Json.Decode.string)
        (Json.Decode.field "points" Json.Decode.int)
        (Json.Decode.field "cost" decodeElementVector)


decodeRootMyInventory : Json.Decode.Decoder PompMyInventory
decodeRootMyInventory =
    Json.Decode.map3 PompMyInventory
        (Json.Decode.field "name" Json.Decode.string)
        (Json.Decode.field "energy" Json.Decode.int)
        (Json.Decode.field "elements" decodeElementVector)


decodeRootOthersObject : Json.Decode.Decoder PompOthersInventory
decodeRootOthersObject =
    Json.Decode.map3 PompOthersInventory
        (Json.Decode.field "name" Json.Decode.string)
        (Json.Decode.field "energy" Json.Decode.int)
        (Json.Decode.field "elements" decodeElementVector)



-- Pomp Remote Event -----------------------------------------------------------
--------------------------------------------------------------------------------


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
