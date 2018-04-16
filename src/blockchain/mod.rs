//! This module contains of external and internal apis.

pub mod blockchain;
pub mod blockchain_external;
pub mod blockchain_sync;

use self::blockchain::BlockChain;
use self::blockchain_external::BlockChainExternalAPIs;
use self::blockchain_sync::BlockChainSyncAPIs;
use gotham::handler::HandlerFuture;
use gotham::state::State;
use hyper::Response;
use peers::PeerAPIs;
use std::sync::{Arc, RwLock};

pub struct BlockChainAPIs {
    blockchain_external_apis: BlockChainExternalAPIs,
    blockchain_sync_apis: BlockChainSyncAPIs,
}

impl BlockChainAPIs {
    pub fn new(peer_apis: Arc<PeerAPIs>) -> Self {
        let block_chain = Arc::new(RwLock::new(BlockChain::new()));

        BlockChainAPIs {
            blockchain_external_apis: BlockChainExternalAPIs::new(
                block_chain.clone(),
                peer_apis.clone(),
            ),
            blockchain_sync_apis: BlockChainSyncAPIs::new(block_chain, peer_apis.clone()),
        }
    }

    /// Re-export external apis list block chain method.
    pub fn list_block_chain(&self, state: State) -> (State, Response) {
        self.blockchain_external_apis.list_block_chain(state)
    }

    /// Re-export external apis mine block method.
    pub fn mine_block(&self, state: State) -> Box<HandlerFuture> {
        self.blockchain_external_apis.mine_block(state)
    }

    /// Re-export sync apis response latest block method.
    pub fn response_latest_block(&self, state: State) -> (State, Response) {
        self.blockchain_sync_apis.response_latest_block(state)
    }

    /// Response the whole chain, same as listing whole block chain in this node.
    pub fn response_whole_chain(&self, state: State) -> (State, Response) {
        self.list_block_chain(state)
    }

    /// Re-export sync apis publish block handler method.
    pub fn publish_block_handler(&self, state: State) -> Box<HandlerFuture> {
        self.blockchain_sync_apis.publish_block_handler(state)
    }
}
