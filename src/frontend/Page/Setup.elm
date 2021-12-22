module Page.Setup exposing (view)

import Html exposing (Html, button, div, text)
import Html.Events exposing (onClick)
import LiveData exposing (..)


view : SetupLiveState -> Html SetupRemoteEvent
view model =
    div []
        (div [] [ text ("My index is: " ++ String.fromInt model.myIndex) ]
            :: List.map viewPlayerData model.data
        )


viewPlayerData : PlayerSetupData -> Html SetupRemoteEvent
viewPlayerData data =
    div []
        [ text
            ("Player "
                ++ data.name
                ++ (if data.isReady then
                        " is ready."

                    else
                        " is not ready."
                   )
            )
        ]
