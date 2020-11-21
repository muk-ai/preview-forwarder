#![feature(proc_macro_hygiene, decl_macro)]
use std::process::Command;
use std::env;
use dotenv::dotenv;
use lazy_static::lazy_static;

#[macro_use] extern crate rocket;
use rocket::request::{FromRequest, Outcome, Request};
use rocket::response::status;

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
fn index(host: HostHeader) -> Result<String, status::NotFound<&'static str>> {
    let tag = get_subdomain(&host)?;
    let image = format!("{}/{}:{}", CONFIG.registry, CONFIG.repository, tag);

    let output = Command::new("docker")
        .args(&["image", "inspect", &image])
        .output()
        .expect("error1");
    if !output.status.success() {
        let output = Command::new("docker")
            .args(&["run", "--rm", "amazon/aws-cli", "ecr", "get-login-password", "--region", "ap-northeast-1"])
            .output()
            .expect("error1");
        if !output.status.success() {
            return Ok("aws ecr get-login-password failed".to_string());
        }
        let password = std::str::from_utf8(&output.stdout).unwrap();
        let output = Command::new("docker")
            .args(&["login", "--username", "AWS", "--password", password, &CONFIG.registry])
            .output()
            .expect("error1");
        if !output.status.success() {
            return Ok("docker login failed".to_string());
        }
        let output = Command::new("docker")
            .args(&["pull", &image])
            .output()
            .expect("error1");
        if output.status.success() {
            return Ok("Image successfully pulled".to_string());
        } else {
            return Ok(std::str::from_utf8(&output.stderr).unwrap().to_string());
        }
    }

    let _output = Command::new("docker")
        .arg("run")
        .args(&["-p", &CONFIG.port.to_string()])
        .args(&["--net", "docker.internal"])
        .args(&["--label", "traefik.enable=true"])
        .args(&["--label", &format!("traefik.http.routers.{}.rule=Host(`{}`)", tag, host.0)])
        .args(&["--label", &format!("traefik.http.routers.{}.priority=50", tag)])
        .arg(&image)
        .spawn()
        .expect("error1");
    Ok("container launched".to_string())
}

fn get_subdomain(host: &HostHeader) -> Result<String, status::NotFound<&'static str>> {
    let list: Vec<&str> = host.0.split('.').collect();
    match list.get(0) {
        Some(s) => Ok(s.to_string()),
        None => Err(status::NotFound("couldn't find subdomain"))
    }
}

fn main() {
    rocket::ignite().mount("/", routes![index]).launch();
}
