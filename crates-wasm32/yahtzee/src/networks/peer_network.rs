use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, WebSocket};
use serde::{Serialize, Deserialize};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};
use super::peer::{PeerConnection, DataChannel, IceCandidate};

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
        username: Rc<str>,
        sdp_description: String,
        ice_candidates: Vec<IceCandidate>,
    }
}

struct PeerData {
    pc: PeerConnection,
    dc: DataChannel,
    name: Rc<str>,
    callbacks: Vec<Box<dyn Drop>>,
}
struct PeerNetworkData {
    id: u8,
    peers: BTreeMap<u8, PeerData>,
    callbacks: Vec<Box<dyn Drop>>,
}

#[derive(Clone)]
pub struct PeerNetwork(Rc<RefCell<PeerNetworkData>>);
impl PeerNetwork {
    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(PeerNetworkData {
            id: 0,
            peers: BTreeMap::new(),
            callbacks: Vec::new(),
        })))
    }
    fn create_peer_data(&self, name: Rc<str>, peer_id: u8) -> (PeerData, UnboundedReceiver<IceCandidate>) {
        let peer_network = self.clone();
        let peer_connection = PeerConnection::new();
        let (sender, receiver) = futures::channel::mpsc::unbounded::<IceCandidate>();
        let onicecandidate_callback = peer_connection.set_onicecandidate(move |event| {
            match event.candidate() {
                Some(candidate) => sender.unbounded_send(candidate.into()).unwrap(),
                None => sender.close_channel(),
            }
        });
        let data_channel = peer_connection.create_data_channel("Data Channel", 0);
        let onopen_callback = data_channel.set_onopen(move || {
            log::info!("Data Channel to {peer_id} opened!");
        });
        let onclose_callback = data_channel.set_onclose(move || {
            log::info!("Data Channel to {peer_id} closed!");
            peer_network.0.borrow_mut().peers.remove(&peer_id);
        });
        (
            PeerData {
                pc: peer_connection,
                dc: data_channel,
                name: name,
                callbacks: vec![
                    Box::new(onicecandidate_callback),
                    Box::new(onopen_callback),
                    Box::new(onclose_callback)],
            },
            receiver
        )
    }
    pub fn connect(&self, name: String, address: String) {
        let username: Rc<str> = name.into();
        let websocket = WebSocket::new(address.as_str()).unwrap();
            websocket.set_binary_type(BinaryType::Arraybuffer);
        let websocket_clone = websocket.clone();
        let peer_network = self.clone();
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            log::info!("Received websocket message.");
            let buffer = event.data().dyn_into::<js_sys::ArrayBuffer>().unwrap();
            let u8arr = js_sys::Uint8Array::new(&buffer);
            let u8vec: Vec<u8> = u8arr.to_vec();
            let message = bincode::deserialize::<SocketMessage>(&u8vec).unwrap();
            match message {
                SocketMessage::ConnectSuccess { lobby_id, assigned_id, peers_id } => {
                    log::info!("Invite code to lobby: https://joongle.dev/yahtzee?lobby_id={lobby_id}");
                    peer_network.0.borrow_mut().id = assigned_id;
                    for peer_id in peers_id {
                        let username = username.clone();
                        let websocket = websocket.clone();
                        let peer_network = peer_network.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let (peer_data, candidates) = peer_network.create_peer_data("".into(), peer_id);
                            let offer_sdp = peer_data.pc.create_offer_sdp().await;
                            peer_network.0.borrow_mut().peers.insert(peer_id, peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: assigned_id,
                                target: peer_id,
                                username: username,
                                sdp_description: offer_sdp,
                                ice_candidates: candidates.collect::<Vec<_>>().await,
                            };
                            let serialized = bincode::serialize(&message).unwrap();
                            websocket.send_with_u8_array(serialized.as_slice()).unwrap();
                        });
                    }
                },
                SocketMessage::WebRtcHandshake {
                    source: peer_id,
                    target: user_id,
                    username: peer_name,
                    sdp_description: sdp,
                    ice_candidates } => {
                    if let Some(peer) = peer_network.0.borrow_mut().peers.get_mut(&peer_id) {
                        log::info!("Received SDP answer from {peer_name}");
                        peer.name = peer_name;
                        let peer_connection = peer.pc.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            peer_connection.receive_answer_sdp(sdp).await;
                            for ice_candidate in ice_candidates {
                                peer_connection.add_ice_candidate(ice_candidate).await;
                            }
                        });
                    }
                    else {
                        log::info!("Received SDP offer from {peer_name}");
                        let username = username.clone();
                        let websocket = websocket.clone();
                        let peer_network = peer_network.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let (peer_data, candidates) = peer_network.create_peer_data(peer_name, peer_id);
                            peer_data.pc.receive_offer_sdp(sdp).await;
                            let answer_sdp = peer_data.pc.create_answer_sdp().await;
                            peer_network.0.borrow_mut().peers.insert(peer_id, peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: user_id,
                                target: peer_id,
                                username: username,
                                sdp_description: answer_sdp,
                                ice_candidates: candidates.collect::<Vec<_>>().await,
                            };
                            let serialized = bincode::serialize(&message).unwrap();
                            websocket.send_with_u8_array(serialized.as_slice()).unwrap();
                        });
                    }
                }
            }
        });
        websocket_clone.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        self.0.borrow_mut().callbacks.push(Box::new(onmessage_callback));
    }
}