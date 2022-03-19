module Page.Wedding exposing (view)

import Element exposing (Element, centerX, centerY, column, el, fill, height, padding, paragraph, px, rgb, row, spacing, width)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Html exposing (Html, br, button, div, node, p, text)
import Html.Attributes exposing (class)
import Html.Events exposing (on, onClick)
import Json.Decode
import List.Extra as List
import Svg exposing (Svg)
import Svg.Attributes as SvgA
import Thunderstorm exposing (thunderstorm)
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

        Projector data ->
            Element.layout [] (projectorView data)



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
            , questionStateButton question.question.state "Konflikt" id ConflictAnswer
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



-- Projector -------------------------------------------------------------------
--------------------------------------------------------------------------------


projectorView : ProjectorView -> Element a
projectorView data =
    column [ width fill, padding 10, spacing 5 ]
        [ el [ centerX, Font.size 50, Font.color (Element.rgb 0.4 0.4 0.4) ] (Element.text "Hochzeit von Birte & Jeremias")
        , case data.question of
            Just hostQuestion ->
                questionView hostQuestion

            Nothing ->
                Element.text "Gleich geht es los!"
        , Element.text ("Es spielen " ++ String.fromInt (List.length data.connectedUsers) ++ " Gäste:")
        , paragraph [ width fill ]
            (List.map (\name -> Element.text (name ++ ", ")) data.connectedUsers)
        ]


questionView : HostQuestion -> Element a
questionView hostQuestion =
    column [ width fill, spacing 5 ]
        [ el [ centerX, Font.size 40 ] (Element.text hostQuestion.question.text)
        , row [ width fill ]
            [ el [ width fill ] (Element.html (graph hostQuestion))
            , el [ width fill ] (Element.text "Question Scores")
            , el [ width fill ] (Element.text "Total Scores")
            ]
        ]


graph : HostQuestion -> Html a
graph hostQuestion =
    let
        maxGuesses =
            max hostQuestion.brideGuesses hostQuestion.groomGuesses

        hBride =
            if maxGuesses == 0 then
                0

            else
                (toFloat hostQuestion.brideGuesses / toFloat maxGuesses) * 50

        hGroom =
            if maxGuesses == 0 then
                0

            else
                (toFloat hostQuestion.groomGuesses / toFloat maxGuesses) * 50

        colorBride =
            if hostQuestion.question.state == Answered Bride then
                "#ff0000"

            else
                "#aa7777"

        colorGroom =
            if hostQuestion.question.state == Answered Groom then
                "#0000ff"

            else
                "#7777aa"

        thunderOverlay =
            if hostQuestion.question.state == ConflictAnswer then
                thunderstorm

            else
                []
    in
    Svg.svg [ SvgA.width "70mm", SvgA.height "70mm", SvgA.viewBox "0 0 70 70" ]
        ([ Svg.g []
            [ Svg.text_ [ svgTextStyle, SvgA.x "13.149777", SvgA.y "66.716469" ]
                [ Svg.text "Birte" ]
            , Svg.text_ [ svgTextStyle, SvgA.x "38.147503", SvgA.y "66.716469" ]
                [ Svg.text "Jeremias" ]
            , Svg.text_ [ svgTextStyle, SvgA.x "19.951769", SvgA.y "7.4098768" ]
                [ Svg.tspan [ SvgA.style "text-align:center;text-anchor:middle" ] [ Svg.text (String.fromInt hostQuestion.brideGuesses) ] ]
            , Svg.text_ [ svgTextStyle, SvgA.x "49.951767", SvgA.y "7.4642406" ]
                [ Svg.tspan [ SvgA.style "text-align:center;text-anchor:middle" ] [ Svg.text (String.fromInt hostQuestion.groomGuesses) ] ]
            , Svg.rect
                [ SvgA.style ("fill:" ++ colorBride ++ ";stroke-width:8.94427;stroke-linecap:round;stroke-linejoin:round")
                , SvgA.width "20"
                , SvgA.height (String.fromFloat hBride)
                , SvgA.x "10"
                , SvgA.y (String.fromFloat (60 - hBride))
                , SvgA.rx "3"
                , SvgA.ry "3"
                ]
                []
            , Svg.rect
                [ SvgA.style ("fill:" ++ colorGroom ++ ";stroke-width:8.94427;stroke-linecap:round;stroke-linejoin:round")
                , SvgA.width "20"
                , SvgA.height (String.fromFloat hGroom)
                , SvgA.x "40"
                , SvgA.y (String.fromFloat (60 - hGroom))
                , SvgA.rx "3"
                , SvgA.ry "3"
                ]
                []
            ]
         ]
            ++ thunderOverlay
        )


svgTextStyle : Svg.Attribute a
svgTextStyle =
    SvgA.style "font-size:5.64444px;line-height:1.25;font-family:sans-serif;stroke-width:0.264583"
