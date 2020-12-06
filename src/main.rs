#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;

mod config;
use config::CONFIG;
mod docker_command;
use docker_command::{docker_image_inspect, docker_login, docker_pull_image, docker_run_image};
mod utils;
use utils::{get_subdomain, is_commit_hash_characters};

struct HostHeader(pub String);
impl<'a, 'r> FromRequest<'a, 'r> for HostHeader {
    type Error = ();

    fn from_request(request: &'a Request) -> Outcome<Self, Self::Error> {
        match request.headers().get_one("Host") {
            Some(host) => match get_subdomain(host.to_string()) {
                Ok(subdomain) => {
                    if is_commit_hash_characters(subdomain) {
                        Outcome::Success(HostHeader(host.to_string()))
                    } else {
                        Outcome::Forward(())
                    }
                }
                Err(_) => Outcome::Forward(()),
            },
            None => Outcome::Forward(()),
        }
    }
}

#[get("/")]
fn index_with_host_header(host: HostHeader) -> Result<String, status::Custom<String>> {
    let tag = get_subdomain(host.0.clone())?;
    let image = format!("{}/{}:{}", CONFIG.registry, CONFIG.repository, tag);
    if cfg!(debug_assertions) {
        dbg!(&image);
    }

    let inspect = docker_image_inspect(&image)?;
    if !inspect.status.success() {
        docker_login()?;
        return docker_pull_image(&image);
    }

    return docker_run_image(host.0, tag, image);
}

#[get("/", rank = 2)]
fn index() -> &'static str {
    "Could you please specify a docker tag."
}

fn main() {
    if cfg!(debug_assertions) {
        dbg!(&CONFIG.registry);
        dbg!(&CONFIG.repository);
        dbg!(CONFIG.port);
    }
    rocket::ignite()
        .mount("/", routes![index, index_with_host_header])
        .launch();
}
