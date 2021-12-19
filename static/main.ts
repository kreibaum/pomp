/// Main typescript file that handles the ports and flags for elm.

/// Type Declaration to make the typescript compiler stop complaining about elm.
/// This could be more precise listing also the ports that we have for better
/// controll.
declare var Elm: any;

/// Init Elm app. Ports can only be set up after this.
var app = Elm.Main.init({
    node: document.getElementById("myapp"),
});

var ws = new WebSocket("ws://127.0.0.1:8080/ws/");
ws.onmessage = function (message) {
    console.log(message);
    app.ports.websocketIn.send({ data: JSON.parse(message.data), timeStamp: message.timeStamp });
};
app.ports.websocketOut.subscribe(function (msg) { ws.send(JSON.stringify(msg)); });
