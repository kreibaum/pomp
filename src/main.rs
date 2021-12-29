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

use game::{RemoteEvent, SharedLiveState, UserUuid, UserView};
use lazy_static::lazy_static;
use log::{debug, info};
use regex::Regex;
use serde::Serialize;
use temp::BoxAddr;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// Remote Events need to track who send them so the game can process them properly.
struct RemoteEventRaw {
    event: String,
    sender: UserUuid,
}

impl Message for RemoteEventRaw {
    type Result = ();
}

/// The LiveActor is the actor that handles the websocket connection & the LiveState.
/// If several pages share a common state, this is a GameState instead and handled
/// by the GameActor instead.
/// To make it possible to transition between different pages, the WebsocketActor
/// takes care of the Websocket connection and nothing else.
struct WebsocketActor {
    hb: Instant,
    uuid: UserUuid,
    backing_actor: BoxAddr,
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
struct UpdateLiveState<T: UserView> {
    route: &'static str,
    data: T,
}

impl<T: UserView> Message for UpdateLiveState<T> {
    type Result = ();
}

impl<T: UserView> Handler<UpdateLiveState<T>> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateLiveState<T>, ctx: &mut ws::WebsocketContext<WebsocketActor>) {
        println!("Sending update to client.");
        ctx.text(serde_json::to_string(&msg).unwrap());
    }
}

struct PerformLiveRedirect(BoxAddr);

impl Message for PerformLiveRedirect {
    type Result = ();
}

impl Handler<PerformLiveRedirect> for WebsocketActor {
    type Result = ();

    fn handle(
        &mut self,
        msg: PerformLiveRedirect,
        _ctx: &mut ws::WebsocketContext<WebsocketActor>,
    ) {
        self.backing_actor = msg.0;
    }
}

/// Sets up a websocket connection ensuring there is a uuid.
async fn websocket_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    if let Some(uuid) = UserUuid::from_query_string(req.query_string()) {
        let router = LiveRouteBroker::from_registry();
        let m = RouteResolution("/pomp/1/setup".to_owned());
        let addr = router
            .send(m)
            .await
            .map_err(|_| {
                actix_web::error::ErrorInternalServerError(
                    "Internal Server Error in the actor system. (Mailbox Error)",
                )
            })?
            .ok_or(actix_web::error::ErrorNotFound("Route not found."))?;

        let resp = ws::start(
            WebsocketActor {
                hb: Instant::now(),
                uuid,
                backing_actor: addr,
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
struct SharedLiveActor<S: SharedLiveState> {
    state: S,
    subs: HashMap<Addr<WebsocketActor>, UserUuid>,
}

impl<G: SharedLiveState> Actor for SharedLiveActor<G> {
    type Context = actix::Context<Self>;

    // Start game loop when actor starts
    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(duration) = self.state.tick_frequency() {
            ctx.run_interval(duration, |act, _ctx| {
                act.state.process_tick();
                for sub in act.subs.keys() {
                    sub.do_send(UpdateLiveState {
                        data: act.state.user_view(&act.subs[sub]),
                        route: G::route_id(),
                    });
                }
            });
        }
    }
}

/// Technically, there should be a difference between RemoteEvents (Client -> LiveActor) and
/// Events that are send from the LiveActor to the GameActor.
impl<G: SharedLiveState> Handler<RemoteEventRaw> for SharedLiveActor<G> {
    type Result = ();

    fn handle(&mut self, e: RemoteEventRaw, ctx: &mut Self::Context) -> Self::Result {
        let event = match RemoteEvent::deserialize(&e.event) {
            Ok(event) => event,
            Err(_) => {
                println!(
                    "Could not decode message as RemoteEvent: {} from sender {} on route {}",
                    e.event,
                    e.sender,
                    G::route_id()
                );
                return;
            }
        };

        let effect = self.state.process_remote_event(event, e.sender);
        match effect {
            game::LiveEffect::None => {}
            game::LiveEffect::LiveRedirect(game_state) => {
                println!("Redirecting to {:?}", game_state.type_id());

                // TODO: Send the new state to the actor
                let new_ref = if let Some(game_state) = game_state.downcast_ref::<pomp::GameState>()
                {
                    println!("Redirecting to Pomp");
                    todo!() //get_actor_reference(pomp::GameState::route_id())
                } else if let Some(game_state) = game_state.downcast_ref::<setup::GameState>() {
                    println!("Redirecting to Setup");
                    todo!() //get_actor_reference(setup::GameState::route_id())
                } else {
                    panic!("Unknown game state type");
                };

                // for sub in self.subs.iter() {
                //     sub.0.do_send(PerformLiveRedirect(new_ref.clone()));

                //     // TODO: Send the new state to the actor
                //     if let Some(game_state) = game_state.downcast_ref::<pomp::GameState>() {
                //         println!("Sending Pomp state to new actor");
                //         sub.0.do_send(UpdateLiveState {
                //             data: game_state.user_view(&sub.1),
                //             route: pomp::GameState::route_id(),
                //         });
                //     } else if let Some(game_state) = game_state.downcast_ref::<setup::GameState>() {
                //         sub.0.do_send(UpdateLiveState {
                //             data: game_state.user_view(&sub.1),
                //             route: setup::GameState::route_id(),
                //         });
                //     } else {
                //         panic!("Unknown game state type");
                //     }
                // }

                // This context is no longer needed.
                self.subs.clear();
                ctx.stop();
            }
        }

        for sub in self.subs.iter() {
            sub.0.do_send(UpdateLiveState {
                data: self.state.user_view(&sub.1),
                route: G::route_id(),
            });
        }
    }
}

/// A LiveActor subscribes to the GameActor to get updates.
struct Subscribe(Addr<WebsocketActor>, UserUuid);

impl Message for Subscribe {
    type Result = ();
}

impl<G: SharedLiveState> Handler<Subscribe> for SharedLiveActor<G> {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        println!("New connection from {}", msg.1);
        self.subs.insert(msg.0.clone(), msg.1.clone());
        self.state.join_user(msg.1.clone());
        println!("Connected sockets: {}", self.subs.len());

        // A new user joining usually updates the state. Sending this to all
        // users directly.
        for sub in self.subs.iter() {
            sub.0.do_send(UpdateLiveState {
                data: self.state.user_view(&sub.1),
                route: G::route_id(),
            });
        }
    }
}

/// When a LiveActor disconnects, it unsubscribes from the GameActor.
struct Unsubscribe(Addr<WebsocketActor>);

impl Message for Unsubscribe {
    type Result = ();
}

impl<G: SharedLiveState> Handler<Unsubscribe> for SharedLiveActor<G> {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
        // Note that this does not tell the game implementation about the change.
        // This is because here we just take care of the disconnecting websocket
        // and the person who left may still be connected in another browser tab.
        self.subs.remove(&msg.0);
        println!("Remaining sockets: {}", self.subs.len());
    }
}

// Live Route Broker that knows all the actors for live routes and can set up //
// new ones.                                                                  //
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct LiveRouteBroker {
    setup: Option<BoxAddr>,
    pomp: Option<BoxAddr>,
}

impl Supervised for LiveRouteBroker {}

impl SystemService for LiveRouteBroker {}

impl Actor for LiveRouteBroker {
    type Context = Context<Self>;
}

/// Send a route resolution message to the live route broker to find the LiveActor
/// responsible for handling the route.
struct RouteResolution(String);

impl Message for RouteResolution {
    type Result = Option<BoxAddr>;
}

impl Handler<RouteResolution> for LiveRouteBroker {
    type Result = Option<BoxAddr>;

    fn handle(&mut self, msg: RouteResolution, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Resolving route {}", msg.0);

        // TODO: This needs to be replaced by some propper router eventually.
        lazy_static! {
            static ref POMP_ROUTE: Regex = Regex::new(r"^/pomp/(\d+)$").unwrap();
            static ref SETUP_ROUTE: Regex = Regex::new(r"^/pomp/(\d+)/setup$").unwrap();
        }

        // Resolve "/" to the index live actor.
        // todo. For now we just send you to /setup/1.

        // Resolve "/pomp/{game_id}" to the pomp live actor.
        if POMP_ROUTE.is_match(&msg.0) {
            // For now there is only a single game that can be played.
            // This should not be set up automatically, because it needs to be
            // set up via the /pomp/{game_id}/setup route.
            return self.pomp.clone();
        }

        // Resolve "/pomp/{game_id}/setup" to the setup live actor.
        if SETUP_ROUTE.is_match(&msg.0) {
            // for now there is only a single game that can be played.
            if self.setup.is_none() {
                info!("Spawning new setup actor");
                let actor: SharedLiveActor<setup::GameState> = SharedLiveActor::default();
                let addr = actor.start();
                self.setup = Some(BoxAddr::Setup(addr));
            }
            return Some(self.setup.clone().unwrap());
        }

        None
    }
}

mod temp {
    //! Module that holds app-specific boilerplate code that should be eliminated
    //! or automatically generated.
    use super::*;

    #[derive(Clone)]
    pub(crate) enum BoxAddr {
        Setup(Addr<SharedLiveActor<setup::GameState>>),
        Pomp(Addr<SharedLiveActor<pomp::GameState>>),
    }

    impl BoxAddr {
        // If a message can be send to all the LiveActors, the PageActor accepts it.
        pub(crate) fn do_send<M>(&self, m: M)
        where
            M: Message + Send + 'static,
            M::Result: Send + 'static,
            SharedLiveActor<pomp::GameState>: Handler<M>,
            SharedLiveActor<setup::GameState>: Handler<M>,
        {
            match self {
                BoxAddr::Pomp(addr) => addr.do_send(m),
                BoxAddr::Setup(addr) => addr.do_send(m),
            }
        }
    }
}

// Set up logging //
////////////////////

fn init_logger() {
    use simplelog::*;

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Debug,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        // WriteLogger::new(
        //     LevelFilter::Info,
        //     Config::default(),
        //     std::fs::File::create("server.log").unwrap(),
        // ),
    ])
    .unwrap();

    debug!("Logger successfully initialized");
}

// Actually starting the server //
//////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    HttpServer::new(|| App::new().route("/ws", web::get().to(websocket_connect)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
