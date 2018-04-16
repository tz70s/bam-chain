//! The servicc module serve the external communication for all nodes.

use blockchain::BlockChainAPIs;
use gotham::router::{Router, builder::*};
use gotham::state::State;
use gotham::{self, http::response::create_response};
use hyper::{Response, StatusCode};
use mime::TEXT_PLAIN;
use peers::PeerAPIs;
use std::sync::Arc;

pub fn start(port: u32) {
    let addr = format!("0.0.0.0:{}", port);
    info!("spawn a miner server at {}", addr);
    let entry_service = EntryService::new();
    let shared_entry_service = Arc::new(entry_service);
    gotham::start(addr, EntryService::router(shared_entry_service.clone()));
}

/// The root, mediate service struct.
/// Composed of helper routes, peers discovery and mining related services.
pub struct EntryService {
    block_chain_apis: BlockChainAPIs,
    peer_apis: Arc<PeerAPIs>,
}

impl EntryService {
    fn new() -> Self {
        let peer_apis = Arc::new(PeerAPIs::new());
        EntryService {
            block_chain_apis: BlockChainAPIs::new(peer_apis.clone()),
            peer_apis,
        }
    }

    /// Default route for entry_service service.
    fn default_hello(&self, state: State) -> (State, Response) {
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((
                String::from("Hello, I'm one of miner nodes!").into_bytes(),
                TEXT_PLAIN,
            )),
        );
        (state, res)
    }

    /// Router generator static method.
    fn router(entry_service: Arc<EntryService>) -> Router {
        build_simple_router(|route| {
            // ------------ External routes for ui control ------------

            // Default route, return the hello world message.
            let shared_entry_service = entry_service.clone();
            route
                .get("/")
                .to_new_handler(move || Ok(|state| shared_entry_service.default_hello(state)));

            // Listing blocks in this node.
            let shared_entry_service = entry_service.clone();
            route.get("/list").to_new_handler(move || {
                Ok(|state| {
                    shared_entry_service
                        .block_chain_apis
                        .list_block_chain(state)
                })
            });

            // Mine a block.
            let shared_entry_service = entry_service.clone();
            route.post("/mine").to_new_handler(move || {
                Ok(|state| shared_entry_service.block_chain_apis.mine_block(state))
            });

            // Add peers from carriers.
            let shared_entry_service = entry_service.clone();
            route.post("/add_peers").to_new_handler(move || {
                Ok(|state| shared_entry_service.peer_apis.add_peers(state))
            });

            // List peers of in this node.
            let shared_entry_service = entry_service.clone();
            route.get("/list_peers").to_new_handler(move || {
                Ok(|state| shared_entry_service.peer_apis.list_peers(state))
            });

            // ------------ Internal routes for miner nodes communications ------------

            // Response latest block.
            let shared_entry_service = entry_service.clone();
            route.get("/response_latest_block").to_new_handler(move || {
                Ok(|state| {
                    shared_entry_service
                        .block_chain_apis
                        .response_latest_block(state)
                })
            });

            // Response whole chain in this node.
            let shared_entry_service = entry_service.clone();
            route.get("/response_whole_chain").to_new_handler(move || {
                Ok(|state| {
                    shared_entry_service
                        .block_chain_apis
                        .response_whole_chain(state)
                })
            });

            // Publish blocks to this node.
            let shared_entry_service = entry_service.clone();
            route.post("/publish_blocks").to_new_handler(move || {
                Ok(|state| {
                    shared_entry_service
                        .block_chain_apis
                        .publish_block_handler(state)
                })
            });
        })
    }
}
