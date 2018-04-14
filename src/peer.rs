//! The peer module builds metadata for tracking all nodes.

use std::sync::{Arc, RwLock};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Liveness {
    Live,
    Failed,
    Unknown,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Peer {
    address: String,
    liveness: Liveness,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peers {
    known_peers: Vec<Peer>,
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            known_peers: Vec::new(),
        }
    }

    pub fn compare_and_update(self_peers: Arc<RwLock<Peers>>, other_peers: Peers) {
        // Check out if there's already known peers or not.
        for known_peer in other_peers.known_peers.into_iter() {
            let own_peers = self_peers.read().unwrap();
            let mut known_peers_iter = own_peers.known_peers.iter();
            if !known_peers_iter.any(|p| p == &known_peer) {
                let mut own_peers = self_peers.write().unwrap();
                own_peers.known_peers.push(known_peer);
            }
        }
    }
}
