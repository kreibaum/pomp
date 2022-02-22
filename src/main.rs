mod game;
mod pomp;
mod setup;
mod wedding;
mod wedding_types;

use std::any::Any;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, Handler, Message, StreamHandler, Supervised, SystemService};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

use game::{RemoteEvent, SharedLiveState, UserUuid, UserView};
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use regex::Regex;
use serde::Serialize;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// The `WebsocketActor` takes care of the websocket connection. It forwards the
/// current `UserView` (LiveState) to the client. It also tracks which
/// `LiveActor` it is connected to and will send `RemoteEvent`s to the correct
/// `LiveActor`.
///
/// The `WebsocketActor` keeps the websocket connection open, even while live
/// navigation is happening.
///
/// It also handles the heartbeat pings.
struct WebsocketActor {
    hb: Instant,
    uuid: UserUuid,
    backing_actor: Recipient<SharedLiveActorMessage>,
    last_send: String,
}

impl Actor for WebsocketActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register self to get updates to the game state
        let result = self
            .backing_actor
            .do_send(SharedLiveActorMessage::Subscribe(
                ctx.address(),
                self.uuid.clone(),
            ));
        if result.is_err() {
            error!("Could not subscribe to backing actor.");
        }
        self.hb(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Unregister self
        let result = self
            .backing_actor
            .do_send(SharedLiveActorMessage::Unsubscribe(ctx.address()));
        if result.is_err() {
            error!("Could not unsubscribe from backing actor.");
        }
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
            Ok(ws::Message::Binary(_)) => {
                debug!("Received binary message which is not expected.");
            }
            _ => (),
        }
    }
}

impl WebsocketActor {
    fn handle_text(&mut self, msg: String, _ctx: &mut <WebsocketActor as Actor>::Context) {
        // We can not decode the String into a structure directly inside the
        // WebsocketActor, because only the GameActor or LiveActor knows about
        // the right type to deserialize into.
        let result = self
            .backing_actor
            .do_send(SharedLiveActorMessage::ClientSideEvent {
                event: msg,
                sender: self.uuid.clone(),
            });
        if result.is_err() {
            error!("Could not send client side event to backing actor.");
        }
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

/// The Live Actor tells the Websocket about the game state.
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
        trace!("Sending update to client.");

        let text = serde_json::to_string(&msg).expect("Serde error while encoding to json.");
        // Check if there were any changes
        if self.last_send == text {
            // Nothing to do, skip this update.
            return;
        }

        // TODO: Ideally we would only send the changes to reduce trafic.
        self.last_send = text.clone();
        ctx.text(text);
    }
}

struct PerformLiveRedirect(Recipient<SharedLiveActorMessage>);

impl Message for PerformLiveRedirect {
    type Result = ();
}

impl Handler<PerformLiveRedirect> for WebsocketActor {
    type Result = ();

    fn handle(&mut self, msg: PerformLiveRedirect, ctx: &mut ws::WebsocketContext<WebsocketActor>) {
        self.backing_actor = msg.0;
        let result = self
            .backing_actor
            .do_send(SharedLiveActorMessage::Subscribe(
                ctx.address(),
                self.uuid.clone(),
            ));
        if result.is_err() {
            error!("Could not subscribe to backing actor. (Called for redirect)");
        }
    }
}

/// Sets up a websocket connection ensuring there is a uuid.
async fn websocket_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    if let Some(uuid) = UserUuid::from_query_string(req.query_string()) {
        let router = LiveRouteBroker::from_registry();
        let m = RouteResolution("/wedding".to_owned());
        let addr = router
            .send(m)
            .await
            .map_err(|_| {
                actix_web::error::ErrorInternalServerError(
                    "Internal Server Error in the actor system. (Mailbox Error)",
                )
            })?
            .ok_or_else(|| actix_web::error::ErrorNotFound("Route not found."))?;

        let resp = ws::start(
            WebsocketActor {
                hb: Instant::now(),
                uuid,
                backing_actor: addr,
                last_send: "".to_owned(),
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

impl<S: SharedLiveState> SharedLiveActor<S> {
    fn new(state: S) -> Self {
        Self {
            state,
            subs: HashMap::new(),
        }
    }
}

impl<G: SharedLiveState> Actor for SharedLiveActor<G> {
    type Context = actix::Context<Self>;

    // Start game loop when actor starts
    fn started(&mut self, ctx: &mut Self::Context) {
        if let Some(duration) = self.state.tick_frequency() {
            ctx.run_interval(duration, |act, _ctx| {
                act.state.process_tick(); // TODO: Figure out how we can handle effects here.
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

impl<G: SharedLiveState> SharedLiveActor<G> {
    fn handle_live_effect(
        &mut self,
        effect: game::LiveEffect,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        match effect {
            game::LiveEffect::None => {
                for sub in self.subs.iter() {
                    sub.0.do_send(UpdateLiveState {
                        data: self.state.user_view(sub.1),
                        route: G::route_id(),
                    });
                }

                Box::pin(async move {})
            }
            game::LiveEffect::LiveRedirectInit(route, game_state) => {
                // When processign a LiveRedirect with a game state, we need to
                // ask the broker to ensure it exist and init it with the game state
                // if it doesn't.
                debug!("LiveRedirect to {} with setup", route);
                let router = LiveRouteBroker::from_registry();
                let m = RouteResolutionWithSetup(route, game_state);
                let new_ref_req = router.send(m);

                let all_subs: Vec<_> = self.subs.keys().cloned().collect();
                self.subs.clear();

                Box::pin(async move {
                    let new_ref = new_ref_req
                        .await
                        .expect("Could not resolve route")
                        .expect("Got 404 for route");

                    // Redirect all subscribers to the new route
                    for sub in all_subs {
                        sub.do_send(PerformLiveRedirect(new_ref.clone()));
                    }
                })
            }
            game::LiveEffect::LiveRedirect(route) => {
                debug!("LiveRedirect to {} without setup", route);
                let router = LiveRouteBroker::from_registry();
                let m = RouteResolution(route);
                let new_ref_req = router.send(m);

                let all_subs: Vec<_> = self.subs.keys().cloned().collect();
                self.subs.clear();

                Box::pin(async move {
                    let new_ref = new_ref_req
                        .await
                        .expect("Could not resolve route")
                        .expect("Got 404 for route");

                    // Redirect all subscribers to the new route
                    for sub in all_subs {
                        sub.do_send(PerformLiveRedirect(new_ref.clone()));
                    }
                })
            }
        }
    }
}

// Message send by the Websocket actor to the backing Live State Actor.
enum SharedLiveActorMessage {
    Subscribe(Addr<WebsocketActor>, UserUuid),
    Unsubscribe(Addr<WebsocketActor>),
    ClientSideEvent { event: String, sender: UserUuid },
}

impl Message for SharedLiveActorMessage {
    type Result = ();
}

impl<G: SharedLiveState> Handler<SharedLiveActorMessage> for SharedLiveActor<G> {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: SharedLiveActorMessage, _ctx: &mut Self::Context) -> Self::Result {
        match msg {
            SharedLiveActorMessage::Subscribe(sub, uuid) => {
                println!("New connection from {}", uuid);
                self.subs.insert(sub, uuid.clone());
                let effect = self.state.join_user(uuid);
                println!("Connected sockets: {}", self.subs.len());
                self.handle_live_effect(effect)
            }
            SharedLiveActorMessage::Unsubscribe(sub) => {
                // Note that this does not tell the game implementation about the change.
                // This is because here we just take care of the disconnecting websocket
                // and the person who left may still be connected in another browser tab.
                self.subs.remove(&sub);
                println!("Remaining sockets: {}", self.subs.len());
                Box::pin(async move {})
            }
            SharedLiveActorMessage::ClientSideEvent { event, sender } => {
                let event = match RemoteEvent::deserialize(&event) {
                    Ok(event) => event,
                    Err(_) => {
                        println!(
                            "Could not decode message as RemoteEvent: {} from sender {} on route {}",
                            event,
                            sender,
                            G::route_id()
                        );
                        return Box::pin(async move {});
                    }
                };

                let effect = self.state.process_remote_event(event, sender);

                self.handle_live_effect(effect)
            }
        }
    }
}
// Live Route Broker that knows all the actors for live routes and can set up //
// new ones.                                                                  //
////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
struct LiveRouteBroker {
    setup: Option<Recipient<SharedLiveActorMessage>>,
    pomp: Option<Recipient<SharedLiveActorMessage>>,
    wedding: Option<Recipient<SharedLiveActorMessage>>,
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
    type Result = Option<Recipient<SharedLiveActorMessage>>;
}

lazy_static! {
    static ref POMP_ROUTE: Regex = Regex::new(r"^/pomp/(\d+)$").unwrap();
    static ref SETUP_ROUTE: Regex = Regex::new(r"^/pomp/(\d+)/setup$").unwrap();
}

impl Handler<RouteResolution> for LiveRouteBroker {
    type Result = Option<Recipient<SharedLiveActorMessage>>;

    fn handle(&mut self, msg: RouteResolution, _ctx: &mut Self::Context) -> Self::Result {
        if msg.0 == "/wedding" {
            if self.wedding.is_none() {
                let actor: SharedLiveActor<wedding::WeddingData> =
                    SharedLiveActor::new(wedding::WeddingData::default());
                let addr = actor.start();
                self.wedding = Some(addr.recipient());
            }
            return Some(self.wedding.clone().unwrap());
        }

        debug!("Resolving route {}", msg.0);

        // TODO: This needs to be replaced by some propper router eventually.

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
                self.setup = Some(addr.recipient());
            }
            return Some(self.setup.clone().unwrap());
        }

        None
    }
}

struct RouteResolutionWithSetup(String, Box<dyn Any + Send>);

impl Message for RouteResolutionWithSetup {
    // TODO: Right now None is "not found" and "illegal setup". I should introduce
    // a custom error type here. (With an Into<Error> impl for actix.)
    type Result = Option<Recipient<SharedLiveActorMessage>>;
}

impl Handler<RouteResolutionWithSetup> for LiveRouteBroker {
    type Result = Option<Recipient<SharedLiveActorMessage>>;

    fn handle(&mut self, msg: RouteResolutionWithSetup, _ctx: &mut Self::Context) -> Self::Result {
        debug!("Resolving route {} (with setup data)", msg.0);

        if POMP_ROUTE.is_match(&msg.0) {
            if self.pomp.is_none() {
                info!("Spawning new pomp actor");
                let game: pomp::GameState = *msg
                    .1
                    .downcast::<pomp::GameState>()
                    .expect("Setup data is not a pomp::GameState");

                let actor = SharedLiveActor::new(game);
                let addr = actor.start();
                self.pomp = Some(addr.recipient());
            }
            return self.pomp.clone();
        }

        todo!()
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
        .bind("0.0.0.0:8080")?
        .run()
        .await
}
