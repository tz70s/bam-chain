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
#[macro_use]
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate tokio_core;

mod block;
mod miner;
mod peer;
mod service;

use peer::LISTENED_PORT;

fn main() {
    env_logger::init();
    info!("simple blockchain implementation in rust.");
    service::start(*LISTENED_PORT);
}
