//! Baby blockchain implementation in rust,
//! inspired by [naivechain](https://github.com/lhartikk/naivechain).

#[macro_use]
extern crate log;
extern crate chrono;
extern crate env_logger;
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate sha3;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

mod block;
mod miner;
mod peer;
mod service;

use std::env;

const DEFAULT_PORT: u32 = 8191;

fn main() {
    env_logger::init();
    info!("Simple blockchain implementation in rust.");
    let arg: Vec<_> = env::args().collect();
    if arg.len() > 1 {
        service::start(arg[1].parse().unwrap_or(DEFAULT_PORT));
    } else {
        service::start(DEFAULT_PORT);
    }
}
