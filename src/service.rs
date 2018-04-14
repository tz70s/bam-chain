//! The service module serve the external communication for all nodes.

use gotham::router::{Router, builder::*};
use gotham::state::State;
use gotham::{self, http::response::create_response};
use hyper::{Response, StatusCode};
use mime;
use miner::MinerService;
use std::sync::Arc;

const DEFAULT_PORT: u32 = 8191;

pub fn start() {
    let addr = format!("0.0.0.0:{}", DEFAULT_PORT);
    info!("Spawn a miner server at {}", addr);
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
        })
    }
}
