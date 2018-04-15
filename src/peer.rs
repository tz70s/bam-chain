//! The peer module builds metadata for tracking all nodes.

use futures::{future, Future, Stream};
use gotham::handler::{HandlerFuture, IntoHandlerError};
use gotham::http::response::create_response;
use gotham::state::{FromState, State};
use hyper::header::{ContentLength, ContentType};
use hyper::{self, Body, Client, Method, Request, Response, StatusCode, Uri};
use mime;
use miner::MinerMessage;
use serde_json;
use std::mem;
use std::sync::{Arc, RwLock};
use tokio_core::reactor::Handle;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Liveness {
    Live,
    Failed,
    Unknown,
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct Peer {
    pub address: String,
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
        // FIXME: should remove this whole replacement.
        let mut own_peers = self_peers.write().unwrap();
        mem::replace(&mut *own_peers, other_peers);
        // FIXME: this can't work now.
        /*
        for carrier in other_peers.carriers.into_iter() {
            let own_peers = self_peers.read().unwrap();
            let mut carriers_iter = own_peers.carriers.iter();
            if !carriers_iter.any(|p| p == &carrier) {
                let mut own_peers = self_peers.write().unwrap();
                own_peers.carriers.push(carrier);
            }
        }
        */    }
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
                    let res = create_response(&state, StatusCode::Ok, None);
                    future::ok((state, res))
                }
                Err(e) => return future::err((state, e.into_handler_error())),
            });
        Box::new(parse_future)
    }

    pub fn list_peers(&self, state: State) -> (State, Response) {
        let cloned_peers = self.peers.clone();
        let res = create_response(
            &state,
            StatusCode::Ok,
            Some((
                serde_json::to_vec(&*cloned_peers.read().unwrap()).unwrap(),
                mime::APPLICATION_JSON,
            )),
        );
        (state, res)
    }

    /// Broadcast something to all nodes.
    pub fn broadcast(&self, handle: Handle, msg: MinerMessage) -> BroadcastFuture {
        match msg {
            MinerMessage::PublishLatestBlock(content) => {
                let mut broadcast_futures = Vec::new();
                for peer in self.peers.read().unwrap().carriers.iter() {
                    let clone_content = content.clone();
                    let dst_path = format!("http://{}/{}", peer.address, "publish_blocks");
                    let fut = http_post(&handle.clone(), &dst_path, clone_content);
                    trace!("publish blocks to : {} ...", dst_path);
                    broadcast_futures.push(fut);
                }
                Box::new(future::join_all(broadcast_futures).then(|_| Ok(None)))
            }
            // Ignore other message types currently.
            _ => Box::new(future::ok(None)),
        }
    }
}

type ResponseContentFuture = Box<Future<Item = Vec<u8>, Error = hyper::Error>>;
type BroadcastFuture = Box<Future<Item = Option<()>, Error = hyper::Error>>;

fn http_get(handle: &Handle, url_str: &str) -> ResponseContentFuture {
    let client = Client::new(handle);
    let url: Uri = url_str.parse().unwrap();
    let f = client.get(url).and_then(|response| {
        response
            .body()
            .concat2()
            .and_then(|full_body| Ok(full_body.to_vec()))
    });
    Box::new(f)
}

fn http_post(handle: &Handle, url_str: &str, msg: Vec<u8>) -> ResponseContentFuture {
    let client = Client::new(handle);
    let url: Uri = url_str.parse().unwrap();
    let mut request = Request::new(Method::Post, url);
    request.headers_mut().set(ContentType::json());
    request.headers_mut().set(ContentLength(msg.len() as u64));
    request.set_body(msg);
    let f = client.request(request).and_then(|response| {
        response
            .body()
            .concat2()
            .and_then(|full_body| Ok(full_body.to_vec()))
    });
    Box::new(f)
}
