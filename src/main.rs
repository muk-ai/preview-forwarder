#![feature(proc_macro_hygiene, decl_macro)]
use std::process::Command;
use std::env;
use dotenv::dotenv;
use lazy_static::lazy_static;

#[macro_use] extern crate rocket;
use rocket::request::{FromRequest, Outcome, Request};

#[derive(Debug)]
struct Config {
    registry: String,
    repository: String,
    port: u16,
}
impl Config {
    fn from_env() -> Config {
        dotenv().ok();
        let registry = env::var("DOCKER_REGISTRY").expect("environment variable DOCKER_REGISTRY is not defined");
        let repository = env::var("DOCKER_REPOSITORY").expect("environment variable DOCKER_REPOSITORY is not defined");
        let port = env::var("DOCKER_PORT").expect("environment variable DOCKER_PORT is not defined");
        let port: u16 = port.parse().unwrap();
        Config {
            registry,
            repository,
            port
        }
    }
}
lazy_static! {
    static ref CONFIG: Config = {
        Config::from_env()
    };
}

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
    Command::new("docker")
        .arg("run")
        .arg("hello-world")
        .spawn()
        .expect("docker command failed");
    format!("Host is {}", host.0)
}

fn main() {
    println!("{}", CONFIG.registry);
    println!("{}", CONFIG.repository);
    println!("{}", CONFIG.port);

    rocket::ignite().mount("/", routes![index]).launch();
}
