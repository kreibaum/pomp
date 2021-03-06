module Page.Pomp exposing (view)

import Html exposing (Html, button, div, text)
import Html.Attributes exposing (class)
import Html.Events exposing (onClick)
import LiveData exposing (..)
import PompData exposing (..)


view : PompPlayerView -> Html PompEvent
view model =
    div []
        [ viewWinner model.winner
        , viewMyInventory model.myInventory
        , viewMarketplace model.market
        , viewOthers model.others
        ]


viewWinner : Maybe String -> Html a
viewWinner maybeWinner =
    case maybeWinner of
        Nothing ->
            div [] []

        Just winner ->
            div [ class "text-center bg-yellow-300 p-1 sm:p-2 font-bold m-1" ]
                [ text winner
                , text " won the game!"
                ]


viewMyInventory : PlayerInventoryView -> Html PompEvent
viewMyInventory inventory =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2" ]
        [ div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt inventory.points ++ " Points") ]
            , div [ class "basis-2/4 text-center p-1 font-bold" ] [ text inventory.name ]
            , div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt inventory.energy ++ " Energy") ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ button [ onClick (Buy Fire), class "basis-1/5 text-center p-1 bg-red-200 hover:bg-red-300 active:bg-red-400 border-red-500 border-2" ]
                [ text (String.fromInt inventory.elements.fire ++ " (+" ++ String.fromInt inventory.discount.fire ++ ") Fire")
                ]
            , button [ onClick (Buy Plant), class "basis-1/5 text-center p-1 bg-green-200 hover:bg-green-300 active:bg-green-400 border-green-500 border-2" ]
                [ text (String.fromInt inventory.elements.plant ++ " (+" ++ String.fromInt inventory.discount.plant ++ ") Plant")
                ]
            , button [ onClick (Buy Water), class "basis-1/5 text-center p-1 bg-blue-200 hover:bg-blue-300 active:bg-blue-400 border-blue-500 border-2" ]
                [ text (String.fromInt inventory.elements.water ++ " (+" ++ String.fromInt inventory.discount.water ++ ") Water")
                ]
            , button [ onClick (Buy Earth), class "basis-1/5 text-center p-1 bg-amber-200 hover:bg-amber-300 active:bg-amber-400 border-amber-500 border-2" ]
                [ text (String.fromInt inventory.elements.earth ++ " (+" ++ String.fromInt inventory.discount.earth ++ ") Earth")
                ]
            , button [ onClick (Buy Chaos), class "basis-1/5 text-center p-1 bg-purple-200 hover:bg-purple-300 active:bg-purple-400 border-purple-500 border-2" ]
                [ text (String.fromInt inventory.elements.chaos ++ " (+" ++ String.fromInt inventory.discount.chaos ++ ") Chaos")
                ]
            ]
        ]


viewMarketplace : List (Maybe Card) -> Html PompEvent
viewMarketplace cards =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2 sm:space-y-1" ]
        [ div [ class "font-bold text-center" ] [ text "Marketplace" ]
        , div [ class "grid sm:gap-1 grid-cols-5 grid-rows-3" ]
            (List.map viewMaybeCard cards)
        ]


viewMaybeCard : Maybe Card -> Html PompEvent
viewMaybeCard maybeCard =
    case maybeCard of
        Just card ->
            viewCard card

        Nothing ->
            div [ class "text-center p-1 sm:p-2 border-gray-300 border-2" ] [ text "Sold" ]


viewCard : Card -> Html PompEvent
viewCard card =
    button [ class "p-1 sm:p-2 border-gray-300 border-2", onClick (BuyCard card.id) ]
        [ div [ class "flex flex-row" ]
            [ div [ class "basis-1/2" ] [ text (colorName card.color) ]
            , div [ class "basis-1/2 text-right" ] [ text (String.fromInt card.points) ]
            ]
        , viewCardCost card
        ]


colorName : ElementColor -> String
colorName color =
    case color of
        Fire ->
            "Fire"

        Plant ->
            "Plant"

        Water ->
            "Water"

        Earth ->
            "Earth"

        Chaos ->
            "Chaos"


{-| This shows how much a card costs. All elements that don't have to be paid
at all are left out from the listing. Instead they are padded in the front
with empty lines.
-}
viewCardCost : Card -> Html a
viewCardCost card =
    let
        s name cardCost =
            if cardCost == 0 then
                []

            else
                [ name ++ ": " ++ String.fromInt cardCost ]

        cost =
            List.concat
                [ s "Fire" card.cost.fire
                , s "Plant" card.cost.plant
                , s "Water" card.cost.water
                , s "Earth" card.cost.earth
                , s "Chaos" card.cost.chaos
                ]

        costDivList =
            List.map (\t -> div [ class "text-left" ] [ text t ]) cost

        -- &nbsp; is a non-breaking space
        noBreakSpace =
            String.fromChar '\u{00A0}'

        pad =
            List.repeat (5 - List.length cost) (div [] [ text noBreakSpace ])
    in
    div [] (pad ++ costDivList)


viewOthers : List PlayerInventoryView -> Html a
viewOthers others =
    div []
        (List.map viewOther others)


viewOther : PlayerInventoryView -> Html a
viewOther other =
    div [ class "m-1 bg-gray-100 p-1 sm:p-2" ]
        [ div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt other.points ++ " Points") ]
            , div [ class "basis-2/4 text-center p-1 font-bold" ] [ text other.name ]
            , div [ class "basis-1/4 text-center p-1" ] [ text (String.fromInt other.energy ++ " Energy") ]
            ]
        , div [ class "flex flex-row sm:space-x-1" ]
            [ div [ class "basis-1/5 text-center p-1 bg-red-200 border-red-500 border-2" ]
                [ text (String.fromInt other.elements.fire ++ " (+" ++ String.fromInt other.discount.fire ++ ") Fire")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-green-200 border-green-500 border-2" ]
                [ text (String.fromInt other.elements.plant ++ " (+" ++ String.fromInt other.discount.plant ++ ") Plant")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-blue-200 border-blue-500 border-2" ]
                [ text (String.fromInt other.elements.water ++ " (+" ++ String.fromInt other.discount.water ++ ") Water")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-amber-200 border-amber-500 border-2" ]
                [ text (String.fromInt other.elements.earth ++ " (+" ++ String.fromInt other.discount.earth ++ ") Earth")
                ]
            , div [ class "basis-1/5 text-center p-1 bg-purple-200 border-purple-500 border-2" ]
                [ text (String.fromInt other.elements.chaos ++ " (+" ++ String.fromInt other.discount.chaos ++ ") Chaos")
                ]
            ]
        ]
