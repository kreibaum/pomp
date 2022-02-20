/// Main typescript file that handles the ports and flags for elm.

/// Type Declaration to make the typescript compiler stop complaining about elm.
/// This could be more precise listing also the ports that we have for better
/// controll.
declare var Elm: any;

/// Define a web-component to get around issues with input fields without client state.
class NameInput extends HTMLElement {

    constructor() {
        super();

        let shadowRoot = this.attachShadow({ mode: 'open' });
        shadowRoot.innerHTML = `
            <input placeholder="Dein Name" /><br />
            <button>Mitspielen</button>`;
        shadowRoot.querySelector(`button`).addEventListener('click', (_event) => this.handleClick());
    }

    private handleClick() {
        // Fire a custom event with the value of the input field.
        let event = new CustomEvent('name-input', {
            detail: {
                name: this.shadowRoot.querySelector('input').value
            }
        });
        this.dispatchEvent(event);
    }
}

window.customElements.define('name-input', NameInput);

/// Init Elm app. Ports can only be set up after this.
var app = Elm.Main.init({
    node: document.getElementById("myapp"),
});

function generate_uuid() {
    return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, function (c) {
        var r = Math.random() * 16 | 0, v = c == 'x' ? r : (r & 0x3 | 0x8);
        return v.toString(16);
    });
}

/** We use UUIDs as cheap "account" alternatives. */
function getUuid(): string {
    let uuid = localStorage.getItem('uuid');
    if (!uuid) {
        uuid = generate_uuid();
        localStorage.setItem("uuid", uuid);
    }
    return uuid;
}

function connect_websocket() {
    var ws = new WebSocket(`ws://127.0.0.1:8080/ws?uuid=${getUuid()}`);
    ws.onmessage = function (message) {
        app.ports.websocketIn.send(JSON.parse(message.data));
    };
    app.ports.websocketOut.subscribe(function (msg) { ws.send(JSON.stringify(msg)); });
}

connect_websocket();