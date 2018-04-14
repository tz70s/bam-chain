//! Concise and simple blockchain implementation in rust.
//! Followed guidances by naivechain https://github.com/lhartikk/naivechain

#[macro_use]
extern crate log;
extern crate chrono;
extern crate sha3;
extern crate simple_logger;
#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate serde_json;

mod block;

use block::BlockChain;

fn main() {
    simple_logger::init().unwrap();
    info!("Simple blockchain implementation in rust.");
    let block_chain = BlockChain::new();
    println!("{}", block_chain.to_string());
    let new_block = block_chain.generate_next_block("New generated block!");
    println!("{}", new_block.to_string());
}
