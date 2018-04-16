//! This module builds mining relative internal routes for nodes handshakes.

use super::blockchain::{replace_to_new_chain, BlockChain};
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use mime::APPLICATION_JSON;
use peers::PeerAPIs;
use serde_json;
use std::io::{self, ErrorKind};
use std::mem;
use std::sync::{Arc, RwLock};
use tokio_core::reactor::Handle;

#[derive(Serialize, Deserialize)]
pub enum SyncBlockMessage {
    QueryLatest,
    QueryAll,
    PublishLatestBlock(Vec<u8>),
}

pub struct BlockChainSyncAPIs {
    block_chain: Arc<RwLock<BlockChain>>,
    peer_apis: Arc<PeerAPIs>,
}

impl BlockChainSyncAPIs {
    pub fn new(block_chain: Arc<RwLock<BlockChain>>, peer_apis: Arc<PeerAPIs>) -> Self {
        BlockChainSyncAPIs {
            block_chain,
            peer_apis,
        }
    }

    /// Response the latest block, in json format.
    /// While if there's no latest block, return NotFound as status code.
    pub fn response_latest_block(&self, state: State) -> (State, Response) {
        let block = self.block_chain
            .read()
            .unwrap()
            .get_latest()
            .and_then(|b| Some(b.to_vec()));

        let res = match block {
            Some(b) => create_response(&state, StatusCode::Ok, Some((b, APPLICATION_JSON))),
            None => create_response(&state, StatusCode::NotFound, None),
        };

        (state, res)
    }

    pub fn publish_block_handler(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_chain_parse = self.block_chain.clone();
        let parse_future = Body::take_from(&mut state).concat2().and_then(move |body| {
            // FIXME: currently, we assumed that the body content will be in a listing style of blockchain.
            trace!(
                "accepting content : {} ",
                String::from_utf8(body.to_vec()).unwrap()
            );
            let mut blocks: BlockChain = if let Ok(bs) = serde_json::from_slice(&body.to_vec()) {
                bs
            } else {
                trace!("parse the requested block chain failed.");
                return future::err(
                    io::Error::new(ErrorKind::InvalidData, "parsing block data error.").into(),
                );
            };

            // TODO: sort the blocks by index.

            let concact;
            {
                // TODO: remove unwrap.
                trace!("start to parse latest block...");
                let latest_block = blocks.get_latest().unwrap();
                let own_chain = cloned_chain_parse.read().unwrap();
                let own_latest_block = own_chain.get_latest().unwrap();
                if latest_block.index < own_latest_block.index {
                    trace!("the requested block is lower than latest block...");
                    return future::ok(None);
                }
                concact = own_latest_block.hash == latest_block.pre_hash;
            }

            if concact {
                trace!("concatenate hash value, add to this chain ...");
                // The received block can be concatenated after own latest block.
                // Add it into self chain.
                cloned_chain_parse
                    .write()
                    .unwrap()
                    .add_new_block(blocks.pop_latest().unwrap());
                // TODO: strictly consider exchange message design.
                let self_chain = cloned_chain_parse.read().unwrap();
                let latest_block = self_chain.get_latest().unwrap();
                return future::ok(Some(SyncBlockMessage::PublishLatestBlock(
                    latest_block.to_vec(),
                )));
            } else {
                let new_chain;
                {
                    let own_chain = cloned_chain_parse.write().unwrap();
                    new_chain = replace_to_new_chain(&own_chain, blocks);
                }
                if let Some(nc) = new_chain {
                    let mut own_chain = cloned_chain_parse.write().unwrap();
                    mem::replace(&mut *own_chain, nc);
                } else {
                    // Validation failed on replacing new chain.
                    return future::err(
                        io::Error::new(ErrorKind::InvalidData, "invalid chain.").into(),
                    );
                }
            }
            future::ok(None)
        });

        let handle = Handle::borrow_from(&mut state).clone();
        let cloned_peer_apis = self.peer_apis.clone();
        let notify_future = parse_future
            .and_then(move |opt| match opt {
                Some(msg) => {
                    let broadcast_future = cloned_peer_apis.broadcast(handle, msg);
                    broadcast_future
                }
                None => Box::new(future::ok(None)),
            })
            .and_then(|_| Ok(()));

        Box::new(notify_future.then(move |result| match result {
            Ok(_) => {
                let res = create_response(&state, StatusCode::Ok, None);
                Ok((state, res))
            }
            Err(err) => Err((state, err.into_handler_error())),
        }))
    }
}
