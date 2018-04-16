//! This module builds mining relative external routes for user interactions.

use super::blockchain::BlockChain;
use super::blockchain_sync::SyncBlockMessage;
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use mime::{APPLICATION_JSON, TEXT_PLAIN};
use peers::PeerAPIs;
use std::io::{self, ErrorKind};
use std::sync::{Arc, RwLock};
use tokio_core::reactor::Handle;

pub struct BlockChainExternalAPIs {
    block_chain: Arc<RwLock<BlockChain>>,
    peer_apis: Arc<PeerAPIs>,
}

impl BlockChainExternalAPIs {
    pub fn new(block_chain: Arc<RwLock<BlockChain>>, peer_apis: Arc<PeerAPIs>) -> Self {
        BlockChainExternalAPIs {
            block_chain,
            peer_apis,
        }
    }

    /// Listing the block chain.
    pub fn list_block_chain(&self, state: State) -> (State, Response) {
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((self.block_chain.read().unwrap().to_vec(), APPLICATION_JSON)),
        );
        (state, res)
    }

    /// Use the request data to generate new block.
    /// Currently, the workflow will be:
    /// 1. Parse the requests body, whether the format is.
    /// 2. Generate a new block from the request data.
    /// 3. Checkout if the request new block is validate.
    /// 4. Add block to chain if validate.
    /// 5. Generate broadcast requests as concatenate futures.
    /// 6. Response the updated block chain.
    ///
    /// The following minining is currently without POW, POS works.
    /// Will investigate after whole communication go done.
    pub fn mine_block(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_chain_parse = self.block_chain.clone();
        let parse_future = Body::take_from(&mut state).concat2().and_then(move |body| {
            let content = String::from_utf8(body.to_vec()).unwrap();
            let new_block = cloned_chain_parse
                .read()
                .unwrap()
                .generate_next_block(content);
            let valid = cloned_chain_parse
                .write()
                .unwrap()
                .add_new_block(new_block.clone());
            if !valid {
                return future::err(io::Error::new(ErrorKind::Other, "invalid block.").into());
            }
            future::ok(new_block)
        });

        let handle = Handle::borrow_from(&mut state).clone();
        let cloned_chain_notify = self.block_chain.clone();
        let cloned_peer_apis = self.peer_apis.clone();
        let notify_future = parse_future.and_then(move |b| {
            trace!("start broadcasting after parsing block...");
            let own_chain = cloned_chain_notify.read().unwrap();
            let broadcast_future = cloned_peer_apis.broadcast(
                handle,
                SyncBlockMessage::PublishLatestBlock(own_chain.to_vec()),
            );
            broadcast_future.and_then(|_| Ok(()))
        });

        let cloned_chain_final = self.block_chain.clone();
        Box::new(notify_future.then(move |result| match result {
            Ok(_) => {
                let res = create_response(
                    &state,
                    StatusCode::Ok,
                    // Response the content of whole chain.
                    Some((cloned_chain_final.read().unwrap().to_vec(), TEXT_PLAIN)),
                );
                Ok((state, res))
            }
            Err(err) => Err((state, err.into_handler_error())),
        }))
    }
}
