use std::collections::HashSet;
use std::sync::Arc;

use actix::prelude::*;
use actix::{Actor, Handler, Message, StreamHandler, Supervised, SystemService};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

#[derive(Debug, Default, Clone)]
struct LiveState {
    count: i32,
}

#[derive(Message)]
#[rtype(result = "()")]
enum RemoteEvent {
    Increment,
    Decrement,
}

/// Define HTTP actor
#[derive(Debug)]
struct LiveActor;

impl Actor for LiveActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Register self to get updates to the game state
        GameActor::from_registry().do_send(Subscribe(ctx.address()));
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
            Ok(ws::Message::Text(text)) => self.handle_text(text, ctx),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl LiveActor {
    fn handle_text(&mut self, msg: String, ctx: &mut <LiveActor as Actor>::Context) {
        if msg == "\"Increment\"" {
            GameActor::from_registry().do_send(RemoteEvent::Increment);
        } else if msg == "\"Decrement\"" {
            GameActor::from_registry().do_send(RemoteEvent::Decrement);
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct UpdateLiveState(LiveState);

impl Handler<UpdateLiveState> for LiveActor {
    type Result = ();

    fn handle(&mut self, msg: UpdateLiveState, ctx: &mut <LiveActor as Actor>::Context) {
        ctx.text(format!("{{\"count\": {} }}", msg.0.count));
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(LiveActor {}, &req, stream);
    resp
}

// Actor that holds the shared state //
///////////////////////////////////////

#[derive(Debug, Default)]
struct GameActor {
    state: LiveState,
    subs: HashSet<Addr<LiveActor>>,
}

impl Actor for GameActor {
    type Context = actix::Context<Self>;
}

impl Supervised for GameActor {}

impl SystemService for GameActor {}

/// Technically, there should be a difference between RemoteEvents (Client -> LiveActor) and
/// Events that are send from the LiveActor to the GameActor.
impl Handler<RemoteEvent> for GameActor {
    type Result = ();

    fn handle(&mut self, e: RemoteEvent, _: &mut Self::Context) -> Self::Result {
        match e {
            RemoteEvent::Increment => self.state.count += 1,
            RemoteEvent::Decrement => self.state.count -= 1,
        }
        for sub in self.subs.iter() {
            sub.do_send(UpdateLiveState(self.state.clone()));
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Subscribe(Addr<LiveActor>);

impl Handler<Subscribe> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: Subscribe, _: &mut Self::Context) -> Self::Result {
        msg.0.do_send(UpdateLiveState(self.state.clone()));
        self.subs.insert(msg.0);
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Unsubscribe(Addr<LiveActor>);

impl Handler<Unsubscribe> for GameActor {
    type Result = ();

    fn handle(&mut self, msg: Unsubscribe, _: &mut Self::Context) -> Self::Result {
        self.subs.remove(&msg.0);
        print!("Remaining sockets: {}", self.subs.len());
    }
}

// Actually starting the server //
//////////////////////////////////

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = Arc::new(GameActor::default().start());
    // Create Actix SystemService that contains this CounterActor

    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
