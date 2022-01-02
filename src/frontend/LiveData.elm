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


type alias Card =
    { id : Int
    , color : String
    , points : Int
    , fireCost : Int
    , plantCost : Int
    , waterCost : Int
    , earthCost : Int
    , chaosCost : Int
    }


type alias PompMyInventory =
    { chaos : Int
    , earth : Int
    , energy : Int
    , fire : Int
    , name : String
    , plant : Int
    , water : Int
    }


type alias PompOthersInventory =
    { chaos : Int
    , earth : Int
    , energy : Int
    , fire : Int
    , name : String
    , plant : Int
    , water : Int
    }


decodePompLiveState : Json.Decode.Decoder PompLiveState
decodePompLiveState =
    Json.Decode.map3 PompLiveState
        (Json.Decode.field "my_inventory" decodeRootMyInventory)
        (Json.Decode.field "others" <| Json.Decode.list decodeRootOthersObject)
        (Json.Decode.field "market" <| Json.Decode.list (Json.Decode.maybe decodeCardObject))


decodeCardObject : Json.Decode.Decoder Card
decodeCardObject =
    Json.Decode.map8 Card
        (Json.Decode.field "id" Json.Decode.int)
        (Json.Decode.field "color" Json.Decode.string)
        (Json.Decode.field "points" Json.Decode.int)
        (Json.Decode.field "fire_cost" Json.Decode.int)
        (Json.Decode.field "plant_cost" Json.Decode.int)
        (Json.Decode.field "water_cost" Json.Decode.int)
        (Json.Decode.field "earth_cost" Json.Decode.int)
        (Json.Decode.field "chaos_cost" Json.Decode.int)


decodeRootMyInventory : Json.Decode.Decoder PompMyInventory
decodeRootMyInventory =
    Json.Decode.map7 PompMyInventory
        (Json.Decode.field "chaos" Json.Decode.int)
        (Json.Decode.field "earth" Json.Decode.int)
        (Json.Decode.field "energy" Json.Decode.int)
        (Json.Decode.field "fire" Json.Decode.int)
        (Json.Decode.field "name" Json.Decode.string)
        (Json.Decode.field "plant" Json.Decode.int)
        (Json.Decode.field "water" Json.Decode.int)


decodeRootOthersObject : Json.Decode.Decoder PompOthersInventory
decodeRootOthersObject =
    Json.Decode.map7 PompOthersInventory
        (Json.Decode.field "chaos" Json.Decode.int)
        (Json.Decode.field "earth" Json.Decode.int)
        (Json.Decode.field "energy" Json.Decode.int)
        (Json.Decode.field "fire" Json.Decode.int)
        (Json.Decode.field "name" Json.Decode.string)
        (Json.Decode.field "plant" Json.Decode.int)
        (Json.Decode.field "water" Json.Decode.int)


encodeRoot : PompLiveState -> Json.Encode.Value
encodeRoot root =
    Json.Encode.object
        [ ( "my_inventory", encodeRootMyInventory root.myInventory )
        , ( "others", Json.Encode.list encodeRootOthersObject root.others )
        ]


encodeRootMyInventory : PompMyInventory -> Json.Encode.Value
encodeRootMyInventory rootMyInventory =
    Json.Encode.object
        [ ( "chaos", Json.Encode.int rootMyInventory.chaos )
        , ( "earth", Json.Encode.int rootMyInventory.earth )
        , ( "energy", Json.Encode.int rootMyInventory.energy )
        , ( "fire", Json.Encode.int rootMyInventory.fire )
        , ( "name", Json.Encode.string rootMyInventory.name )
        , ( "plant", Json.Encode.int rootMyInventory.plant )
        , ( "water", Json.Encode.int rootMyInventory.water )
        ]


encodeRootOthersObject : PompOthersInventory -> Json.Encode.Value
encodeRootOthersObject rootOthersObject =
    Json.Encode.object
        [ ( "chaos", Json.Encode.int rootOthersObject.chaos )
        , ( "earth", Json.Encode.int rootOthersObject.earth )
        , ( "energy", Json.Encode.int rootOthersObject.energy )
        , ( "fire", Json.Encode.int rootOthersObject.fire )
        , ( "name", Json.Encode.string rootOthersObject.name )
        , ( "plant", Json.Encode.int rootOthersObject.plant )
        , ( "water", Json.Encode.int rootOthersObject.water )
        ]



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
