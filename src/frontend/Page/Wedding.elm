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
import List.Extra as List
import WeddingData exposing (..)


view : WeddingView -> Html WeddingEvent
view model =
    case model of
        SignUp ->
            signUpView

        Guest data ->
            Element.layout [] (guestView data)

        Host data ->
            Element.layout [] (hostView data)



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


hostView : HostView -> Element WeddingEvent
hostView data =
    column [ width fill, padding 5, spacing 5 ]
        [ selectedQuestionView data
        , pauseView data
        , questionListView data
        ]


selectedQuestionView : HostView -> Element WeddingEvent
selectedQuestionView data =
    case getCurrentQuestion data of
        Nothing ->
            el [] (Element.text "Keine Frage ausgewählt")

        Just ( id, question ) ->
            activeQuestionView id question


activeQuestionView : Int -> HostQuestion -> Element WeddingEvent
activeQuestionView id question =
    column [ padding 10, spacing 5, width fill ]
        [ Element.text question.question.text
        , Element.text ("Votes: " ++ votesString question)
        , row [ spacing 5, width fill ]
            [ questionStateButton question.question.state "Abstimmung offen" id GuestsCanVote
            , questionStateButton question.question.state "Abstimmung beendet" id VotingClosed
            , questionStateButton question.question.state "Birte" id (Answered Bride)
            , questionStateButton question.question.state "Jeremias" id (Answered Groom)
            ]
        ]


questionStateButton : QuestionState -> String -> Int -> QuestionState -> Element WeddingEvent
questionStateButton currentState caption id targetState =
    if currentState == targetState then
        Input.button [ width fill, padding 5, Border.width 1, Background.color (rgb 0.6 0.6 0.9) ]
            { onPress = Nothing
            , label = Element.text caption
            }

    else
        Input.button [ width fill, padding 5, Border.width 1 ]
            { onPress = Just (SetQuestionState id targetState)
            , label = Element.text caption
            }


getCurrentQuestion : HostView -> Maybe ( Int, HostQuestion )
getCurrentQuestion data =
    case data.currentQuestion of
        Nothing ->
            Nothing

        Just id ->
            List.getAt id data.questions
                |> Maybe.map (\question -> ( id, question ))


pauseView : HostView -> Element WeddingEvent
pauseView data =
    row [ width fill, spacing 5 ]
        [ Element.text "Pause - "
        , Input.button []
            { onPress = Just (SetQuestion Nothing)
            , label = Element.text "Pause"
            }
        ]


questionListView : HostView -> Element WeddingEvent
questionListView data =
    column [ spacing 5 ]
        (List.indexedMap
            (\i question ->
                oneQuestionView question (Just i == data.currentQuestion) i
            )
            data.questions
        )


oneQuestionView : HostQuestion -> Bool -> Int -> Element WeddingEvent
oneQuestionView question isActive i =
    row []
        [ Element.text question.question.text
        , Element.text " - "
        , Element.text (votesString question)
        , Element.text " - "
        , Input.button []
            { onPress = Just (SetQuestion (Just i))
            , label = Element.text "[Zeigen]"
            }
        ]


votesString : HostQuestion -> String
votesString question =
    "Bi = " ++ String.fromInt question.brideGuesses ++ ", Je = " ++ String.fromInt question.groomGuesses
