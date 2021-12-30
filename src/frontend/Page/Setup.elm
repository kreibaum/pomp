module Page.Setup exposing (view)

import FontAwesome.Icon exposing (viewIcon)
import FontAwesome.Solid as FA
import Html exposing (Html, button, div, h1, p, text)
import Html.Attributes exposing (class, disabled)
import Html.Events exposing (onClick)
import LiveData exposing (..)


view : SetupLiveState -> Html SetupRemoteEvent
view model =
    div [ class "p-4" ]
        [ h1 [ class "text-xl pb-2" ] [ text "Set up a Pomp Game" ]
        , playerTable model
        , if model.myIndex == 0 then
            startGameSection model

          else
            div [ class "pt-2" ]
                [ text "Please wait for the game to start"
                ]
        ]



-- Player Table ----------------------------------------------------------------
--------------------------------------------------------------------------------


playerTable : SetupLiveState -> Html SetupRemoteEvent
playerTable model =
    div [] (List.indexedMap (viewPlayerData model.myIndex) model.data)


viewPlayerData : Int -> Int -> PlayerSetupData -> Html SetupRemoteEvent
viewPlayerData myIndex dataIndex data =
    if myIndex == dataIndex then
        myPlayerData data

    else
        otherPlayerData data


myPlayerData : PlayerSetupData -> Html SetupRemoteEvent
myPlayerData data =
    div [ class "flex flex-row" ]
        [ div [ class "basis-3/4 space-x-2" ]
            [ text data.name
            , viewIcon FA.user
            ]
        , div [ class "basis-1/4" ]
            [ readyButton data.isReady ]
        ]


otherPlayerData : PlayerSetupData -> Html SetupRemoteEvent
otherPlayerData data =
    div [ class "flex flex-row" ]
        [ div [ class "basis-3/4" ]
            [ text data.name
            ]
        , div [ class "basis-1/4" ]
            [ readyLabel data.isReady ]
        ]


readyLabel : Bool -> Html SetupRemoteEvent
readyLabel isReady =
    p [ class "px-1" ] [ readyLabelContent isReady ]


readyButton : Bool -> Html SetupRemoteEvent
readyButton isReady =
    button [ class "px-1 bg-gray-300 hover:bg-gray-600 rounded", onClick (SetReady (not isReady)) ] [ readyLabelContent isReady ]


readyLabelContent : Bool -> Html SetupRemoteEvent
readyLabelContent isReady =
    div [ class "space-x-2" ]
        (if isReady then
            [ viewIcon FA.check
            , text "Ready"
            ]

         else
            [ viewIcon FA.times
            , text "Not Ready"
            ]
        )



-- Ready Button and Hints ------------------------------------------------------
--------------------------------------------------------------------------------


startGameSection : SetupLiveState -> Html SetupRemoteEvent
startGameSection model =
    startGameButton model


startGameButton : SetupLiveState -> Html SetupRemoteEvent
startGameButton model =
    div [ class "pt-2" ]
        [ if allPlayersReady model then
            button [ class "p-1 bg-indigo-300 hover:bg-indigo-600 rounded", onClick StartGame ] [ text "Start Game" ]

          else
            button [ class "p-1 bg-gray-300 text-gray-600 rounded", disabled True ] [ text "Start Game" ]
        ]


allPlayersReady : SetupLiveState -> Bool
allPlayersReady model =
    List.all .isReady model.data
