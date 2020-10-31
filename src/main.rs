#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
use rocket::request::{FromRequest, Outcome, Request};

struct HostHeader(pub String);
impl<'a, 'r> FromRequest<'a, 'r> for HostHeader {
    type Error = ();

    fn from_request(request: &'a Request) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Host") {
            Some(host) => Outcome::Success(HostHeader(host.to_string())),
            None => Outcome::Forward(()),
        }
    }
}

#[get("/")]
fn index(host: HostHeader) -> String {
    format!("Host is {}", host.0)
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
