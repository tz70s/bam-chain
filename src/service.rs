//! The service module serve the external communication for all nodes.

use gotham::router::{Router, builder::*};
use gotham::state::State;
use gotham::{self, http::response::create_response};
use hyper::{Response, StatusCode};
use mime;
use miner::MinerService;
use std::sync::Arc;

pub fn start(port: u32) {
    let addr = format!("0.0.0.0:{}", port);
    info!("spawn a miner server at {}", addr);
    let middleware = MiddlewareService::new();
    let shared_middleware = Arc::new(middleware);
    gotham::start(addr, MiddlewareService::router(shared_middleware.clone()));
}

/// The root, mediate service struct.
/// Composed of helper routes, peers discovery and mining related services.
pub struct MiddlewareService {
    miner_service: MinerService,
}

impl MiddlewareService {
    fn new() -> Self {
        MiddlewareService {
            miner_service: MinerService::new(),
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

    /// Router generator static method.
    fn router(middleware: Arc<MiddlewareService>) -> Router {
        build_simple_router(|route| {
            // ------------ External routes for ui control ------------

            // Default route, return the hello world message.
            let shared_middleware = middleware.clone();
            route
                .get("/")
                .to_new_handler(move || Ok(|state| shared_middleware.default_hello(state)));

            // Listing blocks in this node.
            let shared_middleware = middleware.clone();
            route.get("/list").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.list_block_chain(state))
            });

            // Mine a block.
            let shared_middleware = middleware.clone();
            route.post("/mine").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.mine_block(state))
            });

            // Add peers from carriers.
            let shared_middleware = middleware.clone();
            route.post("/add_peers").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.add_peers(state))
            });

            // List peers of in this node.
            let shared_middleware = middleware.clone();
            route.get("/list_peers").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.list_peers(state))
            });

            // ------------ Internal routes for miner nodes communications ------------

            // Response latest block.
            let shared_middleware = middleware.clone();
            route.get("/response_latest_block").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.response_latest_block(state))
            });

            // Response whole chain in this node.
            let shared_middleware = middleware.clone();
            route.get("/response_whole_chain").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.response_whole_chain(state))
            });

            // Publish blocks to this node.
            let shared_middleware = middleware.clone();
            route.post("/publish_blocks").to_new_handler(move || {
                Ok(|state| shared_middleware.miner_service.publish_block_handler(state))
            });
        })
    }
}
