use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use actix::prelude::*;
use actix::{Actor, Handler, Message, StreamHandler, Supervised, SystemService};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

/// How often heartbeat pings are sent
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// How long before lack of client response causes a timeout
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Default, Clone)]
struct LiveState {
    count: i32,
    private_count: i32,
}

#[derive(Debug, Default)]
struct GameState {
    players: HashSet<String>,
    count: i32,
    player_private_count: HashMap<String, i32>,
}

impl GameState {
    fn restrict(&self, player: &str) -> LiveState {
        LiveState {
            count: self.count,
            private_count: self.player_private_count.get(player).unwrap_or(&0).clone(),
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct RemoteEventWrapper {
    event: RemoteEvent,
    sender: String,
}

enum RemoteEvent {
    Increment,
    Decrement,
}

/// Define HTTP actor
#[derive(Debug)]
struct LiveActor {
    hb: Instant,
    uuid: String,
}

impl Actor for LiveActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register self to get updates to the game state
        GameActor::from_registry().do_send(Subscribe(ctx.address(), self.uuid.clone()));
        self.hb(ctx);
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Unregister self
        GameActor::from_registry().do_send(Unsubscribe(ctx.address()));
    }
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for LiveActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
                // println!("Pong")
            }
            Ok(ws::Message::Text(text)) => self.handle_text(text, ctx),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl LiveActor {
    fn handle_text(&mut self, msg: String, _ctx: &mut <LiveActor as Actor>::Context) {
        if msg == "\"Increment\"" {
            GameActor::from_registry().do_send(RemoteEventWrapper {
                event: RemoteEvent::Increment,
                sender: self.uuid.clone(),
            });
        } else if msg == "\"Decrement\"" {
            GameActor::from_registry().do_send(RemoteEventWrapper {
                event: RemoteEvent::Decrement,
                sender: self.uuid.clone(),
            });
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

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateLiveState(LiveState);

impl Handler<UpdateLiveState> for LiveActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateLiveState, ctx: &mut <LiveActor as Actor>::Context) {
        ctx.text(format!(
            "{{\"count\": {}, \"private_count\":{} }}",
            msg.0.count, msg.0.private_count
        ));
    }
}

async fn websocket_connect(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    use lazy_static::lazy_static;
    use regex::Regex;
    lazy_static! {
        static ref RE: Regex = Regex::new(
            "UUID=([0-9A-F]{8}-[0-9A-F]{4}-4[0-9A-F]{3}-[89AB][0-9A-F]{3}-[0-9A-F]{12})"
        )
        .unwrap();
    }
    if let Some(cap) = RE.captures_iter(&req.query_string().to_uppercase()).next() {
        if let Some(uuid) = cap.get(1) {
            let uuid = uuid.as_str().to_owned();
            let resp = ws::start(
                LiveActor {
                    hb: Instant::now(),
                    uuid,
                },
                &req,
                stream,
            );
            return resp;
        }
    }
    // Return 401 Unauthorized if we can't find a UUID
    Err(actix_web::error::ErrorUnauthorized(
        "No UUID found in request",
    ))
}

// Actor that holds the shared state //
///////////////////////////////////////

#[derive(Debug, Default)]
struct GameActor {
    state: GameState,
    subs: HashMap<Addr<LiveActor>, String>,
}

// impl Default for GameActor {
//     fn default() -> Self {
//         GameActor {
//             state: GameState::default(),
//             subs: HashMap::new(),
//         }
//     }
// }

impl Actor for GameActor {
    type Context = actix::Context<Self>;
}

impl Supervised for GameActor {}

impl SystemService for GameActor {}

/// Technically, there should be a difference between RemoteEvents (Client -> LiveActor) and
/// Events that are send from the LiveActor to the GameActor.
impl Handler<RemoteEventWrapper> for GameActor {
    type Result = ();

    fn handle(&mut self, e: RemoteEventWrapper, _: &mut Self::Context) -> Self::Result {
        match e.event {
            RemoteEvent::Increment => {
                self.state.count += 1;
                // Increase private count of the player that sent the event
                self.state
                    .player_private_count
                    .entry(e.sender.clone())
                    .and_modify(|v| *v += 1)
                    .or_insert(1);
            }
            RemoteEvent::Decrement => {
                self.state.count -= 1;
                // Decrease private count of the player that sent the event
                self.state
                    .player_private_count
                    .entry(e.sender.clone())
                    .and_modify(|v| *v -= 1)
                    .or_insert(-1);
            }
        }
        for sub in self.subs.iter() {
            sub.0.do_send(UpdateLiveState(self.state.restrict(&sub.1)));
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe(Addr<LiveActor>, String);

impl Handler<Subscribe> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        msg.0.do_send(UpdateLiveState(self.state.restrict(&msg.1)));
        println!("New connection from {}", msg.1);
        self.subs.insert(msg.0, msg.1.clone());
        self.state.players.insert(msg.1);
        println!("Connected sockets: {}", self.subs.len());
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Unsubscribe(Addr<LiveActor>);

impl Handler<Unsubscribe> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
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
