#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

mod config;
mod docker_command;
mod indexes;
mod utils;
use config::CONFIG;

fn main() {
    if cfg!(debug_assertions) {
        dbg!(&CONFIG.registry);
        dbg!(&CONFIG.repository);
        dbg!(CONFIG.port);
    }
    rocket::ignite()
        .mount(
            "/",
            routes![indexes::index, indexes::index_with_host_header],
        )
        .launch();
}
