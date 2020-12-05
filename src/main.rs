#![feature(proc_macro_hygiene, decl_macro)]
use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::process::Command;
use std::{env, process::Output};

#[macro_use]
extern crate rocket;
use rocket::http::Status;
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
        let registry = env::var("DOCKER_REGISTRY")
            .expect("environment variable DOCKER_REGISTRY is not defined");
        let repository = env::var("DOCKER_REPOSITORY")
            .expect("environment variable DOCKER_REPOSITORY is not defined");
        let port =
            env::var("DOCKER_PORT").expect("environment variable DOCKER_PORT is not defined");
        let port: u16 = port.parse().unwrap();
        Config {
            registry,
            repository,
            port,
        }
    }
}
static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env());

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

fn is_commit_hash_characters(host: String) -> bool {
    host.chars()
        .all(|c| ('0' <= c && c <= '9') || ('a' <= c && c <= 'f'))
}

#[get("/")]
fn index(host: HostHeader) -> Result<String, status::Custom<String>> {
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

fn get_subdomain(host: String) -> Result<String, status::Custom<String>> {
    if cfg!(debug_assertions) {
        dbg!(&host);
    }

    let list: Vec<&str> = host.split('.').collect();
    match list.get(0) {
        Some(s) => Ok(s.to_string()),
        None => Err(status::Custom(
            Status::NotFound,
            "couldn't find subdomain".to_string(),
        )),
    }
}

fn docker_image_inspect(image: &String) -> Result<Output, status::Custom<String>> {
    let result = Command::new("docker")
        .args(&["image", "inspect", image])
        .output();
    match result {
        Ok(o) => Ok(o),
        Err(_) => Err(status::Custom(
            Status::InternalServerError,
            "docker image inspect: failed".to_string(),
        )),
    }
}

fn docker_login() -> Result<(), status::Custom<String>> {
    let result = Command::new("docker")
        .args(&[
            "run",
            "--rm",
            "amazon/aws-cli",
            "ecr",
            "get-login-password",
            "--region",
            "ap-northeast-1",
        ])
        .output();
    let output = match result {
        Ok(o) => o,
        Err(_) => {
            return Err(status::Custom(
                Status::InternalServerError,
                "aws ecr get-login-password: failed".to_string(),
            ))
        }
    };
    if !output.status.success() {
        return Err(status::Custom(
            Status::InternalServerError,
            "aws ecr get-login-password: failed".to_string(),
        ));
    }
    let password = std::str::from_utf8(&output.stdout).unwrap();
    let result = Command::new("docker")
        .args(&[
            "login",
            "--username",
            "AWS",
            "--password",
            password,
            &CONFIG.registry,
        ])
        .output();
    let output = match result {
        Ok(o) => o,
        Err(_) => {
            return Err(status::Custom(
                Status::InternalServerError,
                "docker login: failed".to_string(),
            ))
        }
    };
    if !output.status.success() {
        return Err(status::Custom(
            Status::InternalServerError,
            "docker login failed".to_string(),
        ));
    }
    Ok(())
}

fn docker_pull_image(image: &String) -> Result<String, status::Custom<String>> {
    let result = Command::new("docker").args(&["pull", &image]).output();
    let output = match result {
        Ok(o) => o,
        Err(_) => {
            return Err(status::Custom(
                Status::InternalServerError,
                "docker pull: failed".to_string(),
            ))
        }
    };
    if output.status.success() {
        return Ok("Image successfully pulled".to_string());
    } else {
        let message = format!(
            "docker pull: failed -> {}",
            std::str::from_utf8(&output.stderr).unwrap()
        );
        return Err(status::Custom(Status::InternalServerError, message));
    }
}

fn docker_run_image(
    host: String,
    tag: String,
    image: String,
) -> Result<String, status::Custom<String>> {
    let result = Command::new("docker")
        .arg("run")
        .args(&["-p", &CONFIG.port.to_string()])
        .args(&["--net", "docker.internal"])
        .args(&["--label", "traefik.enable=true"])
        .args(&[
            "--label",
            &format!("traefik.http.routers.{}.rule=Host(`{}`)", tag, host),
        ])
        .args(&[
            "--label",
            &format!("traefik.http.routers.{}.priority=50", tag),
        ])
        .arg(&image)
        .spawn();
    match result {
        Ok(_) => Ok("container launched".to_string()),
        Err(_) => {
            return Err(status::Custom(
                Status::InternalServerError,
                "docker run: failed".to_string(),
            ))
        }
    }
}

fn main() {
    if cfg!(debug_assertions) {
        dbg!(&CONFIG.registry);
        dbg!(&CONFIG.repository);
        dbg!(CONFIG.port);
    }
    rocket::ignite().mount("/", routes![index]).launch();
}
