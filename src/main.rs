mod game;
mod pomp;
mod setup;

use std::any::Any;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, Handler, Message, StreamHandler, Supervised, SystemService};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use game::{GameStateTrait, LiveStateTrait, PlayerUuid, RemoteEventTrait};
use serde::Serialize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);
/// Game Loop runs at 5 fps
const GAME_LOOP_INTERVAL: Duration = Duration::from_millis(200);

/// Remote Events need to track who send them so the game can process them properly.
struct RemoteEventRaw {
    event: String,
    sender: PlayerUuid,
}

impl Message for RemoteEventRaw {
    type Result = ();
}

// PLEASE REVIEW: This construction does not feel right to me yet.
#[derive(Clone)]
enum PageActor {
    Pomp(Addr<GameActor<pomp::GameState>>),
    Setup(Addr<GameActor<setup::GameState>>),
}

impl PageActor {
    // If a message can be send to all the GameActors, the PageActor accepts it.
    fn do_send<M>(&self, m: M)
    where
        M: Message + Send + 'static,
        M::Result: Send + 'static,
        GameActor<pomp::GameState>: Handler<M>,
        GameActor<setup::GameState>: Handler<M>,
    {
        match self {
            PageActor::Pomp(addr) => addr.do_send(m),
            PageActor::Setup(addr) => addr.do_send(m),
        }
    }
}

/// The LiveActor is the actor that handles the websocket connection & the LiveState.
/// If several pages share a common state, this is a GameState instead and handled
/// by the GameActor instead.
/// To make it possible to transition between different pages, the WebsocketActor
/// takes care of the Websocket connection and nothing else.
struct WebsocketActor {
    hb: Instant,
    uuid: PlayerUuid,
    backing_actor: PageActor,
}

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register self to get updates to the game state
        self.backing_actor
            .do_send(Subscribe(ctx.address(), self.uuid.clone()));
        self.hb(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Unregister self
        self.backing_actor.do_send(Unsubscribe(ctx.address()));
    }
}

/// Delegate raw websocket messages to better places.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebsocketActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => self.handle_text(text, ctx),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl WebsocketActor {
    fn handle_text(&mut self, msg: String, _ctx: &mut <WebsocketActor as Actor>::Context) {
        // We can not decode the String into a structure directly inside the
        // WebsocketActor, because only the GameActor or LiveActor knows about
        // the right type to deserialize into.
        self.backing_actor.do_send(RemoteEventRaw {
            event: msg,
            sender: self.uuid.clone(),
        });
    }

    /// Heartbeat handler that will kill the process if the client dies.
    fn hb(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            // Are we dead yet?
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                println!("Websocket Client heartbeat failed, disconnecting!");
                ctx.stop();
            } else {
                ctx.ping(b"");
            }
        });
    }
}

/// The GameActor tells the LiveActor about the game state.
#[derive(Serialize)]
struct UpdateLiveState<T: LiveStateTrait> {
    route: &'static str,
    data: T,
}

impl<T: LiveStateTrait> Message for UpdateLiveState<T> {
    type Result = ();
}

impl<T: LiveStateTrait> Handler<UpdateLiveState<T>> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateLiveState<T>, ctx: &mut ws::WebsocketContext<WebsocketActor>) {
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

struct PerformLiveRedirect(PageActor);

impl Message for PerformLiveRedirect {
    type Result = ();
}

impl Handler<PerformLiveRedirect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: PerformLiveRedirect, ctx: &mut ws::WebsocketContext<WebsocketActor>) {
        self.backing_actor = msg.0;
    }
}

fn get_actor_reference(game_state: &str) -> PageActor {
    if pomp::GameState::route_id() == game_state {
        PageActor::Pomp(GameActor::<pomp::GameState>::from_registry())
    } else if setup::GameState::route_id() == game_state {
        PageActor::Setup(GameActor::<setup::GameState>::from_registry())
    } else {
        panic!("Unknown game state type");
    }
}

/// Sets up a websocket connection ensuring there is a uuid.
async fn websocket_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    if let Some(uuid) = PlayerUuid::from_query_string(req.query_string()) {
        let resp = ws::start(
            WebsocketActor {
                hb: Instant::now(),
                uuid,
                //backing_actor: PageActor::Pomp(GameActor::<pomp::GameState>::from_registry()),
                backing_actor: PageActor::Setup(GameActor::<setup::GameState>::from_registry()),
            },
            &req,
            stream,
        );
        return resp;
    }
    // Return 401 Unauthorized if we can't find a UUID
    Err(actix_web::error::ErrorUnauthorized(
        "No UUID found in request",
    ))
}

/// Actor that holds the shared state
#[derive(Default)]
struct GameActor<G: GameStateTrait> {
    state: G,
    subs: HashMap<Addr<WebsocketActor>, PlayerUuid>,
}

impl<G: GameStateTrait> Actor for GameActor<G> {
    type Context = actix::Context<Self>;

    // Start game loop when actor starts
    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(GAME_LOOP_INTERVAL, |act, _ctx| {
            act.state.process_tick();
            for sub in act.subs.keys() {
                sub.do_send(UpdateLiveState {
                    data: act.state.restrict(&act.subs[sub]),
                    route: G::route_id(),
                });
            }
        });
    }
}

impl<G: GameStateTrait> Supervised for GameActor<G> {}

// TODO: There should be a broker service and then multiple games which each
// have their own game actor.
impl<G: GameStateTrait> SystemService for GameActor<G> {}

/// Technically, there should be a difference between RemoteEvents (Client -> LiveActor) and
/// Events that are send from the LiveActor to the GameActor.
impl<G: GameStateTrait> Handler<RemoteEventRaw> for GameActor<G> {
    type Result = ();

    fn handle(&mut self, e: RemoteEventRaw, _: &mut Self::Context) -> Self::Result {
        let event = match RemoteEventTrait::deserialize(&e.event) {
            Ok(event) => event,
            Err(_) => {
                println!(
                    "Could not decode message as RemoteEvent: {} from sender {}",
                    e.event, e.sender
                );
                return;
            }
        };

        let effect = self.state.process_remote_event(event, e.sender);
        match effect {
            game::LiveEffect::None => {}
            game::LiveEffect::LiveRedirect(game_state) => {
                let new_ref = get_actor_reference(&game_state);

                for sub in self.subs.iter() {
                    sub.0.do_send(PerformLiveRedirect(new_ref.clone()));

                    // TODO: Send the new state to the actor
                    if let Some(game_state) = game_state.downcast_ref::<pomp::GameState>() {
                        sub.0.do_send(UpdateLiveState {
                            data: game_state.restrict(&sub.1),
                            route: pomp::GameState::route_id(),
                        });
                    } else if let Some(game_state) = game_state.downcast_ref::<setup::GameState>() {
                        sub.0.do_send(UpdateLiveState {
                            data: game_state.restrict(&sub.1),
                            route: setup::GameState::route_id(),
                        });
                    } else {
                        panic!("Unknown game state type");
                    }
                }
            }
        }

        for sub in self.subs.iter() {
            sub.0.do_send(UpdateLiveState {
                data: self.state.restrict(&sub.1),
                route: G::route_id(),
            });
        }
    }
}

/// A LiveActor subscribes to the GameActor to get updates.
struct Subscribe(Addr<WebsocketActor>, PlayerUuid);

impl Message for Subscribe {
    type Result = ();
}

impl<G: GameStateTrait> Handler<Subscribe> for GameActor<G> {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        println!("New connection from {}", msg.1);
        self.subs.insert(msg.0.clone(), msg.1.clone());
        self.state.join_player(msg.1.clone());
        println!("Connected sockets: {}", self.subs.len());
        msg.0.do_send(UpdateLiveState {
            data: self.state.restrict(&msg.1),
            route: G::route_id(),
        });
    }
}

/// When a LiveActor disconnects, it unsubscribes from the GameActor.
struct Unsubscribe(Addr<WebsocketActor>);

impl Message for Unsubscribe {
    type Result = ();
}

impl<G: GameStateTrait> Handler<Unsubscribe> for GameActor<G> {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
        // Note that this does not tell the game implementation about the change.
        // This is because here we just take care of the disconnecting websocket
        // and the person who left may still be connected in another browser tab.
        self.subs.remove(&msg.0);
        println!("Remaining sockets: {}", self.subs.len());
    }
}

// Actually starting the server //
//////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws", web::get().to(websocket_connect)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
