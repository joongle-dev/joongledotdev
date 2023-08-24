use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use web_sys::{MessageEvent, WebSocket};
use wasm_bindgen::prelude::*;
use futures::StreamExt;
use super::peer::{PeerConnection, DataChannel};

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess {
        lobby_id: u64,
        assigned_id: u8,
        peers_id: Vec<u8>,
    },
    WebRtcHandshake {
        source: u8,
        target: u8,
        sdp_description: String,
        ice_candidates: Vec<String>,
    }
}

struct Peer {
    pc: PeerConnection,
    dc: DataChannel,
    callbacks: Vec<Box<dyn Drop>>,
}
struct PeerNetworkData {
    id: u8,
    peers: BTreeMap<u8, Peer>,
    callbacks: Vec<Box<dyn Drop>>,
}
pub struct PeerNetwork(Rc<RefCell<PeerNetworkData>>);

impl PeerNetwork {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(PeerNetworkData{
            id: 0,
            peers: BTreeMap::new(),
            callbacks: Vec::new(),
        })))
    }
    pub fn connect(&self, address: &str) {
        let websocket = WebSocket::new(address).unwrap();
        let websocket_clone = websocket.clone();
        let network_data = self.0.clone();
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            log::info!("Received websocket message.");
            let buffer = event.data().dyn_into::<js_sys::ArrayBuffer>().unwrap();
            let u8arr = js_sys::Uint8Array::new(&buffer);
            let u8vec: Vec<u8> = u8arr.to_vec();
            let message = bincode::deserialize::<SocketMessage>(&u8vec).unwrap();
            match message {
                SocketMessage::ConnectSuccess { lobby_id, assigned_id, peers_id } => {
                    log::info!("Invite code to lobby: https://joongle.dev/yahtzee?lobby_id={lobby_id}");
                    network_data.borrow_mut().id = assigned_id;
                    for peer_id in peers_id {
                        let websocket = websocket.clone();
                        let network_data = network_data.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let peer_connection = PeerConnection::new();
                            let (sender, receiver) = futures::channel::mpsc::unbounded::<String>();
                            let callback = peer_connection.set_onicecandidate(move |event| {
                                match event.candidate() {
                                    Some(candidate) => sender.unbounded_send(candidate.as_string().unwrap()).unwrap(),
                                    None => sender.close_channel(),
                                }
                            });
                            let data_channel = peer_connection.create_data_channel("Data Channel", 0);
                            let offer_sdp = peer_connection.create_offer_sdp().await;
                            let peer = Peer {
                                pc: peer_connection,
                                dc: data_channel,
                                callbacks: vec![Box::new(callback)],
                            };
                            network_data.borrow_mut().peers.insert(peer_id, peer);
                            let message = SocketMessage::WebRtcHandshake {
                                source: assigned_id,
                                target: peer_id,
                                sdp_description: offer_sdp,
                                ice_candidates: receiver.collect::<Vec<_>>().await,
                            };
                            let serialized = bincode::serialize(&message).unwrap();
                            websocket.send_with_u8_array(serialized.as_slice()).unwrap();
                        });
                    }
                },
                SocketMessage::WebRtcHandshake { source, target, sdp_description: sdp, ice_candidates } => {
                    if let Some(peer) = network_data.borrow_mut().peers.get_mut(&source) {
                        let peer_connection = peer.pc.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            peer_connection.receive_answer_sdp(sdp).await;
                            for ice_candidate in ice_candidates {
                                peer_connection.add_ice_candidate(ice_candidate.as_str()).await;
                            }
                        });
                    }
                    else {
                        let websocket = websocket.clone();
                        let network_data = network_data.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let peer_connection = PeerConnection::new();
                            let (sender, receiver) = futures::channel::mpsc::unbounded::<String>();
                            let callback = peer_connection.set_onicecandidate(move |event| {
                                match event.candidate() {
                                    Some(candidate) => sender.unbounded_send(candidate.as_string().unwrap()).unwrap(),
                                    None => sender.close_channel(),
                                }
                            });
                            let data_channel = peer_connection.create_data_channel("Data Channel", 0);
                            peer_connection.receive_offer_sdp(sdp).await;
                            let answer_sdp = peer_connection.create_answer_sdp().await;
                            let peer = Peer {
                                pc: peer_connection,
                                dc: data_channel,
                                callbacks: vec![Box::new(callback)],
                            };
                            network_data.borrow_mut().peers.insert(source, peer);
                            let message = SocketMessage::WebRtcHandshake {
                                source: target,
                                target: source,
                                sdp_description: answer_sdp,
                                ice_candidates: receiver.collect::<Vec<_>>().await,
                            };
                            let serialized = bincode::serialize(&message).unwrap();
                            websocket.send_with_u8_array(serialized.as_slice()).unwrap();
                        });
                    }
                },
            }
        });
        websocket_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        self.0.borrow_mut().callbacks.push(Box::new(onmessage_callback));
    }
}