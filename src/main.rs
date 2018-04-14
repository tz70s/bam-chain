//! Baby blockchain implementation in rust.
//! Followed guidances by naivechain https://github.com/lhartikk/naivechain

#[macro_use]
extern crate log;
extern crate chrono;
extern crate futures;
extern crate gotham;
extern crate hyper;
extern crate mime;
extern crate sha3;
extern crate simple_logger;
#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;

mod block;
mod miner;
mod peer;
mod service;

fn main() {
    simple_logger::init().unwrap();
    info!("Simple blockchain implementation in rust.");
    service::start();
}
