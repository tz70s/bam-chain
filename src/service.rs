//! The service module serve the external communication for all nodes.

use block::BlockChain;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::router::{Router, builder::*};
use gotham::state::{FromState, State};
use gotham::{self, http::response::create_response};
use hyper::{Body, Response, StatusCode};
use mime;
use peer::Peers;
use serde_json;
use std::sync::{Arc, RwLock};

const DEFAULT_PORT: u32 = 8191;

pub fn start() {
    let addr = format!("0.0.0.0:{}", DEFAULT_PORT);
    info!("Spawn a miner server at {}", addr);
    let miner_service = MinerService::new();
    let shared_service = Arc::new(miner_service);
    gotham::start(addr, router(shared_service.clone()));
}

fn router(miner_service: Arc<MinerService>) -> Router {
    build_simple_router(|route| {
        // For the path "/" invoke the handler "say_hello"
        let cloned_service = miner_service.clone();
        route
            .get("/")
            .to_new_handler(move || Ok(|state| cloned_service.default_hello(state)));
        let cloned_service = miner_service.clone();
        route
            .get("/list")
            .to_new_handler(move || Ok(|state| cloned_service.list_block_chain(state)));
        let cloned_service = miner_service.clone();
        route
            .post("/mine")
            .to_new_handler(move || Ok(|state| cloned_service.mine_block(state)));

        let cloned_service = miner_service.clone();
        route
            .post("/add_peers")
            .to_new_handler(move || Ok(|state| cloned_service.add_peers(state)));
    })
}

pub struct MinerService {
    peers: Arc<RwLock<Peers>>,
    block_chain: Arc<RwLock<BlockChain>>,
}

impl MinerService {
    fn new() -> Self {
        MinerService {
            peers: Arc::new(RwLock::new(Peers::new())),
            block_chain: Arc::new(RwLock::new(BlockChain::new())),
        }
    }

    /// Default route for miner service.
    fn default_hello(&self, state: State) -> (State, Response) {
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((
                String::from("Hello, I'm one of miner nodes!").into_bytes(),
                mime::TEXT_PLAIN,
            )),
        );
        (state, res)
    }

    /// Listing the block chain.
    fn list_block_chain(&self, state: State) -> (State, Response) {
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((
                self.block_chain.read().unwrap().to_vec(),
                mime::APPLICATION_JSON,
            )),
        );
        (state, res)
    }

    /// Use the request data to generate new block.
    /// A.k.a mine block.
    fn mine_block(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_chain = self.block_chain.clone();
        let parse_future = Body::take_from(&mut state)
            .concat2()
            .then(move |full_body| match full_body {
                Ok(valid_body) => {
                    let content = String::from_utf8(valid_body.to_vec()).unwrap();
                    println!("{}", content);

                    let new_block = cloned_chain.read().unwrap().generate_next_block(content);
                    // TODO: somewhat check and add into chain.
                    cloned_chain.write().unwrap().add_new_block(new_block);
                    // TODO: broadcast to other nodes.
                    let res = create_response(
                        &state,
                        StatusCode::Ok,
                        Some((
                            cloned_chain.read().unwrap().to_vec(),
                            mime::APPLICATION_JSON,
                        )),
                    );
                    future::ok((state, res))
                }
                Err(e) => return future::err((state, e.into_handler_error())),
            });
        Box::new(parse_future)
    }

    fn add_peers(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_peers = self.peers.clone();
        let parse_future = Body::take_from(&mut state)
            .concat2()
            .then(move |full_body| match full_body {
                Ok(valid_body) => {
                    let other_peers: Peers = serde_json::from_slice(&valid_body.to_vec()).unwrap();
                    Peers::compare_and_update(cloned_peers, other_peers);

                    let res = create_response(&state, StatusCode::Ok, None);
                    future::ok((state, res))
                }
                Err(e) => return future::err((state, e.into_handler_error())),
            });
        Box::new(parse_future)
    }
}
