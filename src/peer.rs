//! The peer module builds metadata for tracking all nodes.

use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::{Body, StatusCode, Uri};
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};
use serde_json;
use std::sync::{Arc, RwLock};

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Liveness {
    Live,
    Failed,
    Unknown,
}

#[derive(PartialEq, Eq, Debug)]
pub struct Address(pub Uri);

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let address_ser = format!("{}:{}", self.0.host().unwrap(), self.0.port().unwrap());
        serializer.serialize_str(&address_ser)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Deserialize::deserialize(deserializer).and_then(|s: &str| {
            let uri = match s.parse() {
                Ok(u) => u,
                Err(e) => return Err(de::Error::custom(format!("parse uri failed. {}", e))),
            };
            Ok(Address(uri))
        })
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub address: Address,
    pub liveness: Liveness,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Peers {
    carriers: Vec<Peer>,
}

impl Peers {
    pub fn new() -> Peers {
        Peers {
            carriers: Vec::new(),
        }
    }

    pub fn compare_and_update(self_peers: Arc<RwLock<Peers>>, other_peers: Peers) {
        // Check out if there's already known peer or not.
        for carrier in other_peers.carriers.into_iter() {
            let own_peers = self_peers.read().unwrap();
            let mut carriers_iter = own_peers.carriers.iter();
            if !carriers_iter.any(|p| p == &carrier) {
                let mut own_peers = self_peers.write().unwrap();
                own_peers.carriers.push(carrier);
            }
        }
    }
}

#[derive(Debug)]
pub struct PeerService {
    peers: Arc<RwLock<Peers>>,
}

impl PeerService {
    pub fn new() -> Self {
        PeerService {
            peers: Arc::new(RwLock::new(Peers::new())),
        }
    }

    /// Add peers from carried known peers via request.
    pub fn add_peers(&self, mut state: State) -> Box<HandlerFuture> {
        let cloned_peers = self.peers.clone();
        let parse_future = Body::take_from(&mut state)
            .concat2()
            .then(move |full_body| match full_body {
                Ok(valid_body) => {
                    let other_peers: Peers = serde_json::from_slice(&valid_body.to_vec()).unwrap();
                    Peers::compare_and_update(cloned_peers, other_peers);

                    // TODO: return something info?
                    let res = create_response(&state, StatusCode::Ok, None);
                    future::ok((state, res))
                }
                Err(e) => return future::err((state, e.into_handler_error())),
            });
        Box::new(parse_future)
    }

    /// Broadcast something to all nodes.
    pub fn broadcast(&self) {
        unimplemented!();
    }
}
