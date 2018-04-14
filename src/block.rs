//! The block moudle contains the implementation of 'block' in blockchain.

use chrono::Utc;
use serde_json;
use sha3::{Digest, Sha3_256};
use std::mem::transmute;

#[derive(Debug, Serialize, Deserialize)]
pub struct Block {
    index: u32,
    time: String,
    hash: Vec<u8>,
    pre_hash: Vec<u8>,
    data: String,
}

impl Block {
    fn new(index: u32, time: String, hash: Vec<u8>, pre_hash: Vec<u8>, data: String) -> Block {
        Block {
            index,
            time,
            hash,
            pre_hash,
            data,
        }
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

/// Caclulate the hash from input data.
/// Use sha3 - sha256 algorithm.
fn calculate_hash(index: u32, time: &str, pre_hash: &[u8], data: &str) -> Vec<u8> {
    let mut hasher = Sha3_256::default();
    let index_byte: [u8; 4] = unsafe { transmute(index.to_le()) };
    hasher.input(&index_byte);
    hasher.input(time.as_bytes());
    hasher.input(pre_hash);
    hasher.input(data.as_bytes());
    hasher.result().as_slice().to_vec()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockChain {
    chain: Vec<Block>,
}

impl BlockChain {
    /// Create a new block chain instance.
    /// All blocks are stored into a vector.
    /// The data in argument is assumed to be the data of genesis block.
    pub fn new() -> Self {
        // Hardcoded gensis block.
        let mut chain = Vec::new();
        let time = format!("{}", Utc::now());
        let pre_hash = Vec::<u8>::new();
        let data = "Genesis block.".to_string();
        let hash = calculate_hash(0, &time, &pre_hash, &data);
        chain.push(Block::new(0, time, hash, pre_hash, data));
        BlockChain { chain }
    }

    /// Generate next block, return a block.
    pub fn generate_next_block<T: Into<String>>(&self, data: T) -> Block {
        let pre_block = self.chain.last().unwrap();
        let index = pre_block.index + 1;
        let pre_hash = pre_block.hash.clone();
        let time = format!("{}", Utc::now());
        let data = data.into();
        let hash = calculate_hash(index, &time, &pre_hash, &data);
        Block::new(index, time, hash, pre_hash, data)
    }

    pub fn add_new_block(&mut self, block: Block) {
        self.chain.push(block);
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn to_string(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }
}

/// Validate the new generated block with this chain.
pub fn validate_block(pre_block: &Block, block: &Block) -> bool {
    let recalc_hash = calculate_hash(block.index, &block.time, &block.pre_hash, &block.data);
    if pre_block.index + 1 != block.index {
        debug!("validation failed: invalid block index.");
        return false;
    } else if pre_block.hash != block.pre_hash {
        debug!("validation failed: invalid block previous hash.");
        return false;
    } else if recalc_hash != block.hash {
        debug!("validation failed: invalid block hash.");
        return false;
    }
    true
}

/// Validate a block chain, iterate a blockchain and validate all blocks.
fn validate_chain(block_chain: &BlockChain) -> bool {
    // TODO: make the chain iterable.
    let mut pre_block = block_chain.chain.iter().next().unwrap();
    for next_block in block_chain.chain.iter() {
        if !validate_block(pre_block, next_block) {
            return false;
        }
    }
    true
}

fn replace_to_new_chain(old_chain: &BlockChain, plain_chain: &[u8]) -> Option<BlockChain> {
    let new_chain = serde_json::from_slice(plain_chain).unwrap();
    if validate_chain(&new_chain) && new_chain.len() > old_chain.len() {
        Some(new_chain)
    } else {
        None
    }
}
