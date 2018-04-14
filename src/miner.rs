//! The miner module builds mining relative routes for miner.

use block::BlockChain;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use mime;
use peer::PeerService;
use std::sync::{Arc, RwLock};

pub struct MinerService {
    block_chain: Arc<RwLock<BlockChain>>,
    peer_service: PeerService,
}

impl MinerService {
    pub fn new() -> Self {
        MinerService {
            block_chain: Arc::new(RwLock::new(BlockChain::new())),
            peer_service: PeerService::new(),
        }
    }

    /// Listing the block chain.
    pub fn list_block_chain(&self, state: State) -> (State, Response) {
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
    pub fn mine_block(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_chain = self.block_chain.clone();
        let parse_future = Body::take_from(&mut state)
            .concat2()
            .then(move |full_body| match full_body {
                Ok(valid_body) => {
                    let content = String::from_utf8(valid_body.to_vec()).unwrap();
                    let new_block = cloned_chain.read().unwrap().generate_next_block(content);
                    // TODO: somewhat check and then add into chain on success.
                    cloned_chain.write().unwrap().add_new_block(new_block);
                    // TODO: broadcast to other nodes for the update.

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

    /// Re-export peer service add peers method.
    pub fn add_peers(&self, state: State) -> Box<HandlerFuture> {
        self.peer_service.add_peers(state)
    }
}
