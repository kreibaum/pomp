use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;

#[derive(Debug, Default)]
struct LiveState {
    count: i32,
}

/// Define HTTP actor
struct MyWs {
    state: LiveState,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

/// Handler for ws::Message message
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => self.handle_text(text, ctx),
            Ok(ws::Message::Binary(bin)) => ctx.binary(bin),
            _ => (),
        }
    }
}

impl MyWs {
    fn handle_text(&mut self, msg: String, ctx: &mut <MyWs as Actor>::Context) {
        if msg == "\"Increment\"" {
            println!("Incrementing");
            self.state.count += 1;
        } else if msg == "\"Decrement\"" {
            println!("Decrementing");
            self.state.count -= 1;
        }
        ctx.text(format!("{{\"count\": {} }}", self.state.count));
    }
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(
        MyWs {
            state: Default::default(),
        },
        &req,
        stream,
    );
    println!("{:?}", resp);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
