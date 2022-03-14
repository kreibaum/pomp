module Page.Wedding exposing (view)

import Element exposing (Element, centerX, centerY, column, el, fill, height, padding, px, rgb, row, spacing, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
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
            Element.layout [] (guestView data)

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
-- TODO: Namep [ class "text-3xl" ] [ text ("Hallo, " ++ data.name ++ "!") ]
-- TODO: Mark selection


fontL : Element.Attr () a
fontL =
    Font.size 50


fontM : Element.Attr () a
fontM =
    Font.size 40


gap : Int
gap =
    20


guestView : GuestView -> Element WeddingEvent
guestView data =
    column [ padding gap, spacing gap, width fill ]
        [ el [ centerX, fontL ] (Element.text data.question)
        , row [ spacing gap, width fill, height (px 200) ]
            [ Input.button [ width fill, height fill, Border.width 5, guessButtonBg data.guess Bride ]
                { onPress = Just (SetGuess Bride)
                , label = el [ centerX, centerY, fontL ] (Element.text "Birte")
                }
            , Input.button [ width fill, height fill, Border.width 5, guessButtonBg data.guess Groom ]
                { onPress = Just (SetGuess Groom)
                , label = el [ centerX, centerY, fontL ] (Element.text "Jeremias")
                }
            ]
        , el [ fontM ] (Element.text ("Du spielst als " ++ data.name ++ "."))
        , el [ fontM ] (Element.text ("Du hast " ++ String.fromInt 0 ++ " Punkte."))
        ]


guessButtonBg : Maybe Espoused -> Espoused -> Element.Attr () a
guessButtonBg enteredGuess buttonMeaning =
    if enteredGuess == Just buttonMeaning then
        Background.color (rgb 0.6 0.6 0.9)

    else
        Background.color (rgb 1.0 1.0 1.0)



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


oneQuestionView : HostQuestion -> Bool -> Int -> Html WeddingEvent
oneQuestionView question isActive i =
    div []
        [ text question.question.text
        , text " - "
        , text (" Bi = " ++ String.fromInt question.brideGuesses)
        , text (", Je = " ++ String.fromInt question.groomGuesses ++ " - ")
        , button [ onClick (SetQuestion (Just i)) ] [ text "[Zeigen]" ]
        ]
