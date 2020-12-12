use dotenv::dotenv;
use once_cell::sync::Lazy;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub registry: String,
    pub repository: String,
    pub port: u16,
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
pub static CONFIG: Lazy<Config> = Lazy::new(|| Config::from_env());
