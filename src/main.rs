#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

mod hash;
use hash::hash_key;

use std::io::Result;
use rocket::request::{self, Request, FromRequest};
use rocket::response::{self, NamedFile, Response, Responder};
use rocket::http::Status;
use rocket::Outcome;

struct APIKey(String);

impl<'a, 'r> FromRequest<'a, 'r> for APIKey {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<APIKey, ()> {
        let keys: Vec<_> = request.headers().get("Sec-WebSocket-Key").collect();
        if keys.len() != 1 {
            return Outcome::Failure((Status::BadRequest, ()));
        }

        return Outcome::Success(APIKey(hash_key(keys[0].as_bytes())));
    }
}

struct Game {
    key: String,
}

#[get("/game")]
fn game(key: APIKey) -> Game {
    Game { key: key.0 }
}

impl<'r> Responder<'r> for Game {
    fn respond(self) -> response::Result<'r> {
        Response::build()
            .status(Status::Accepted)
            .raw_header("Upgrade", "WebSocket")
            .raw_header("Connection", "Upgrade")
            .raw_header("Sec-WebSocket-Accept", self.key)
            .ok()
    }
}

#[get("/")]
fn index() -> Result<NamedFile> {
    NamedFile::open("static/index.html")
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![index, game])
}

fn main() {
    rocket().launch()
}
