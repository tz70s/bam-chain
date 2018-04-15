//! The miner module builds mining relative routes for miner.

use block::{replace_to_new_chain, BlockChain};
use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, Response, StatusCode};
use mime;
use peer::PeerService;
use serde_json;
use std::io::{self, ErrorKind};
use std::mem;
use std::sync::{Arc, RwLock};
use tokio_core::reactor::Handle;

#[derive(Serialize, Deserialize)]
pub enum MinerMessage {
    QueryLatest,
    QueryAll,
    PublishLatestBlock(Vec<u8>),
}

pub struct MinerService {
    block_chain: Arc<RwLock<BlockChain>>,
    peer_service: Arc<PeerService>,
}

impl MinerService {
    pub fn new() -> Self {
        MinerService {
            block_chain: Arc::new(RwLock::new(BlockChain::new())),
            peer_service: Arc::new(PeerService::new()),
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
        let cloned_peer_service = self.peer_service.clone();
        let notify_future = parse_future.and_then(move |b| {
            trace!("start broadcasting after parsing block...");
            let own_chain = cloned_chain_notify.read().unwrap();
            let broadcast_future = cloned_peer_service
                .broadcast(handle, MinerMessage::PublishLatestBlock(own_chain.to_vec()));
            broadcast_future.and_then(|_| Ok(()))
        });

        let cloned_chain_final = self.block_chain.clone();
        Box::new(notify_future.then(move |result| match result {
            Ok(_) => {
                let res = create_response(
                    &state,
                    StatusCode::Ok,
                    // Response the content of whole chain.
                    Some((
                        cloned_chain_final.read().unwrap().to_vec(),
                        mime::TEXT_PLAIN,
                    )),
                );
                Ok((state, res))
            }
            Err(err) => Err((state, err.into_handler_error())),
        }))
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
            Some(b) => create_response(&state, StatusCode::Ok, Some((b, mime::APPLICATION_JSON))),
            None => create_response(&state, StatusCode::NotFound, None),
        };

        (state, res)
    }

    /// Response the whole chain, same as listing whole block chain in this node.
    pub fn response_whole_chain(&self, state: State) -> (State, Response) {
        self.list_block_chain(state)
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
                // TODO: strictly consider miner message design.
                let self_chain = cloned_chain_parse.read().unwrap();
                let latest_block = self_chain.get_latest().unwrap();
                return future::ok(Some(MinerMessage::PublishLatestBlock(
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
        let cloned_peer_service = self.peer_service.clone();
        let notify_future = parse_future
            .and_then(move |opt| match opt {
                Some(msg) => {
                    let broadcast_future = cloned_peer_service.broadcast(handle, msg);
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

    /// Re-export peer service add peers method.
    pub fn add_peers(&self, state: State) -> Box<HandlerFuture> {
        self.peer_service.add_peers(state)
    }

    /// Re-export peer service list peers method.
    pub fn list_peers(&self, state: State) -> (State, Response) {
        self.peer_service.list_peers(state)
    }
}
