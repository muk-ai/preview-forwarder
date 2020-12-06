use rocket::http::Status;
use rocket::response::status;
use std::process::Command;
use std::process::Output;

use crate::config::CONFIG;

pub fn docker_image_inspect(image: &String) -> Result<Output, status::Custom<String>> {
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

pub fn docker_login() -> Result<(), status::Custom<String>> {
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

pub fn docker_pull_image(image: &String) -> Result<String, status::Custom<String>> {
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

pub fn docker_run_image(
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
