port module Websocket exposing (..)

import Json.Encode exposing (Value)


port websocketOut : Value -> Cmd msg


port websocketIn : (Value -> msg) -> Sub msg
