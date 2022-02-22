module Page.Wedding exposing (view)

import Html exposing (Html, br, button, div, node, p, text)
import Html.Attributes exposing (class)
import Html.Events exposing (on, onClick)
import Json.Decode
import WeddingData exposing (..)


view : WeddingView -> Html WeddingEvent
view model =
    case model of
        SignUp ->
            signUpView

        Guest data ->
            guestView data

        Host data ->
            hostView data



-- Sign up ---------------------------------------------------------------------
--------------------------------------------------------------------------------


signUpView : Html WeddingEvent
signUpView =
    div [ class "p-5 text-center" ]
        [ p [ class "text-6xl py-5" ] [ text "Hochzeit", br [] [], text "Birte & Jeremias" ]
        , p [ class "text-3xl" ] [ text "Mach mit beim Hochzeitsspiel!" ]
        , br [] []
        , node "name-input" [ on "name-input" decodeNameFromCustomEvent ] []
        ]


decodeNameFromCustomEvent : Json.Decode.Decoder WeddingEvent
decodeNameFromCustomEvent =
    Json.Decode.at [ "detail", "name" ] Json.Decode.string
        |> Json.Decode.map (\name -> SetName name)



-- Guest -----------------------------------------------------------------------
--------------------------------------------------------------------------------


guestView : GuestView -> Html WeddingEvent
guestView data =
    div [ class "p-5 text-center" ]
        [ p [ class "text-3xl" ] [ text ("Hallo, " ++ data.name ++ "!") ]
        , br [] []
        , br [] []
        , p [ class "text-4xl" ] [ text data.question ]
        , br [] []
        , button [ onClick (SetGuess Bride), class "text-4xl" ] [ text "Birte" ]
        , text " - "
        , button [ onClick (SetGuess Groom), class "text-4xl" ] [ text "Jeremias" ]
        , br [] []
        , case data.answer of
            Just Bride ->
                text "Birte"

            Just Groom ->
                text "Jeremias"

            Nothing ->
                text "?"
        ]



-- Host ------------------------------------------------------------------------
--------------------------------------------------------------------------------


hostView : HostView -> Html WeddingEvent
hostView data =
    div [ class "text-lg font-medium" ]
        [ text "Hochzeit von Birte & Jeremias - Moderator"
        , br [] []
        , br [] []
        , pauseView data
        , questionView data
        ]


pauseView : HostView -> Html WeddingEvent
pauseView data =
    div []
        [ text "Pause - "
        , button [ onClick (SetQuestion Nothing) ] [ text "Pausieren" ]
        ]


questionView : HostView -> Html WeddingEvent
questionView data =
    div []
        (List.indexedMap
            (\i question ->
                oneQuestionView question (Just i == data.currentQuestion) i
            )
            data.questions
        )


oneQuestionView : Question -> Bool -> Int -> Html WeddingEvent
oneQuestionView question isActive i =
    div []
        [ text question.text
        , text " - "
        , button [ onClick (SetQuestion (Just i)) ] [ text "[Zeigen]" ]
        ]
