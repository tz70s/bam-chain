//! The block moudle contains the implementation of 'block' in blockchain.

use chrono::Utc;
use serde_json;
use sha3::{Digest, Sha3_256};
use std::mem::transmute;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub time: String,
    pub hash: Vec<u8>,
    pub pre_hash: Vec<u8>,
    pub data: String,
}

impl Block {
    fn new(index: u32, time: String, hash: Vec<u8>, pre_hash: Vec<u8>, data: String) -> Self {
        Block {
            index,
            time,
            hash,
            pre_hash,
            data,
        }
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
        // Hardcoded genesis block.
        let mut chain = Vec::new();
        let time = format!("{}", Utc::now());
        let pre_hash = Vec::<u8>::new();
        let data = "Genesis block.".to_string();
        let hash: Vec<u8> = vec![
            58, 189, 197, 52, 175, 113, 254, 248, 138, 252, 216, 166, 7, 222, 247, 172, 174, 235,
            214, 143, 214, 32, 59, 211, 64, 58, 123, 29, 148, 66, 54, 185,
        ];
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

    pub fn add_new_block(&mut self, block: Block) -> bool {
        // Get the latest block.
        let is_valid;
        {
            let latest_block = self.chain.last().unwrap();
            is_valid = validate_block(latest_block, &block)
        }
        if is_valid {
            self.chain.push(block);
        }
        is_valid
    }

    pub fn len(&self) -> usize {
        self.chain.len()
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_json::to_vec(self).unwrap()
    }

    pub fn get_latest(&self) -> Option<&Block> {
        self.chain.last()
    }

    pub fn pop_latest(&mut self) -> Option<Block> {
        self.chain.pop()
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
        pre_block = next_block;
    }
    true
}

pub fn replace_to_new_chain(old_chain: &BlockChain, new_chain: BlockChain) -> Option<BlockChain> {
    if validate_chain(&new_chain) && new_chain.len() > old_chain.len() {
        Some(new_chain)
    } else {
        None
    }
}
