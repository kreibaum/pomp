port module Websocket exposing (send, subscribe)

import Json.Encode exposing (Value)


port websocketOut : Value -> Cmd msg


send : Value -> Cmd msg
send =
    websocketOut


port websocketIn : (Value -> msg) -> Sub msg


subscribe : (Value -> msg) -> Sub msg
subscribe =
    websocketIn
