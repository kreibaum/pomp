module Page.Pomp exposing (view)

import Html exposing (Html, button, div, text)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import LiveData exposing (..)


view : PompLiveState -> Html PompRemoteEvent
view model =
    div []
        [ viewMyInventory model.myInventory
        , viewMarketplace
        , viewOthers model.others
        ]


viewMyInventory : PompMyInventory -> Html PompRemoteEvent
viewMyInventory inventory =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2" ]
        [ div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/4 text-center p-1" ] [ text "0 Points" ]
            , div [ class "basis-2/4 text-center p-1 font-bold" ] [ text inventory.name ]
            , div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt inventory.energy ++ " Energy") ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ button [ onClick (Buy Fire), class "basis-1/5 text-center p-1 bg-red-200 hover:bg-red-300 active:bg-red-400 border-red-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt inventory.fire ++ ") Fire")
                ]
            , button [ onClick (Buy Plant), class "basis-1/5 text-center p-1 bg-green-200 hover:bg-green-300 active:bg-green-400 border-green-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt inventory.plant ++ ") Plant")
                ]
            , button [ onClick (Buy Water), class "basis-1/5 text-center p-1 bg-blue-200 hover:bg-blue-300 active:bg-blue-400 border-blue-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt inventory.water ++ ") Water")
                ]
            , button [ onClick (Buy Earth), class "basis-1/5 text-center p-1 bg-amber-200 hover:bg-amber-300 active:bg-amber-400 border-amber-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt inventory.earth ++ ") Earth")
                ]
            , button [ onClick (Buy Chaos), class "basis-1/5 text-center p-1 bg-purple-200 hover:bg-purple-300 active:bg-purple-400 border-purple-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt inventory.chaos ++ ") Chaos")
                ]
            ]
        ]


viewMarketplace : Html a
viewMarketplace =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2 sm:space-y-1" ]
        [ div [ class "font-bold text-center" ] [ text "Marketplace" ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Lorem" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Ipsum" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Dolor" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "sit" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "amet" ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Lorem" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Ipsum" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Dolor" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "sit" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "amet" ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Lorem" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Ipsum" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "Dolor" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "sit" ]
            , div [ class "basis-1/5 text-center p-1 border-gray-300 border-2" ] [ text "amet" ]
            ]
        ]


viewOthers : List PompOthersInventory -> Html a
viewOthers others =
    div []
        (List.map viewOther others)


viewOther : PompOthersInventory -> Html a
viewOther other =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2" ]
        [ div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/4 text-center p-1" ] [ text "0 Points" ]
            , div [ class "basis-2/4 text-center p-1 font-bold" ] [ text other.name ]
            , div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt other.energy ++ " Energy") ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/5 text-center p-1 bg-red-200 border-red-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt other.fire ++ ") Fire")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-green-200 border-green-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt other.plant ++ ") Plant")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-blue-200 border-blue-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt other.water ++ ") Water")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-amber-200 border-amber-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt other.earth ++ ") Earth")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-purple-200 border-purple-500 border-2" ]
                [ text ("0 (+" ++ String.fromInt other.chaos ++ ") Chaos")
                ]
            ]
        ]
