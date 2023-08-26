use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, WebSocket};
use serde::{Serialize, Deserialize};
use futures::{channel::mpsc::UnboundedReceiver, StreamExt};
use std::{cell::RefCell, rc::Rc};
use crate::networks::webrtc::{Configuration, ConfigurationBuilder, PeerConnection, DataChannel, IceCandidate};

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
impl From<JsValue> for SocketMessage {
    fn from(value: JsValue) -> Self {
        let buffer = value.dyn_into::<js_sys::ArrayBuffer>().unwrap();
        let u8arr = js_sys::Uint8Array::new(&buffer);
        let u8vec: Vec<u8> = u8arr.to_vec();
        bincode::deserialize::<SocketMessage>(&u8vec).unwrap()
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
    peers: [Option<PeerData>; 256],
    callbacks: Vec<Box<dyn Drop>>,
}
#[derive(Clone)]
pub struct PeerNetwork(Rc<RefCell<PeerNetworkData>>);
impl PeerNetwork {
    pub fn new() -> Self {
        const INIT: Option<PeerData> = None;
        Self(Rc::new(RefCell::new(PeerNetworkData {
            id: 0,
            peers: [INIT; 256],
            callbacks: Vec::new(),
        })))
    }
    pub fn broadcast_str(&self, data: &str) {
        for peer in self.0.borrow().peers.iter().filter_map(|v| v.as_ref()) {
            log::info!("Sending \"{}\" to {}; state: {:?}", data, peer.name, peer.dc.ready_state());
            peer.dc.send_str(data);
        }
    }
    fn create_peer_data(&self, configuration: Configuration, peer_name: Rc<str>, peer_id: u8, channel_id: u16) -> (PeerData, UnboundedReceiver<IceCandidate>) {
        let peer_network = self.clone();
        let peer_connection = PeerConnection::new_with_configuration(configuration);
        let peer_connection_clone = peer_connection.clone();
        let onconnectionstatechange_callback = peer_connection.set_onconnectionstatechange(move || {
            log::info!("Connection state to {} change: {:?}", peer_id, peer_connection_clone.connection_state());
        });
        let data_channel = peer_connection.create_data_channel_negotiated("Data Channel", channel_id);
        let onopen_callback = data_channel.set_onopen(move || {
            log::info!("Data Channel to {} opened!", peer_id);
        });
        let onclose_callback = data_channel.set_onclose(move || {
            log::info!("Data Channel to {} closed!", peer_id);
            peer_network.0.borrow_mut().peers[peer_id as usize] = None;
        });
        let onclosing_callback = data_channel.set_onclosing(move || {
            log::info!("Data Channel to {} closing!", peer_id);
        });
        let peer_network_clone = self.clone();
        let onerror_callback = data_channel.set_onerror(move || {
            log::info!("Data Channel to {} error!", peer_id);
            peer_network_clone.0.borrow_mut().peers[peer_id as usize] = None;
        });
        let onmessage_callback = data_channel.set_onmessage(move |event| {
            match event.data().as_string() {
                Some(data) => log::info!("{data}"),
                None => log::info!("Message was not a string.")
            }
        });
        let (sender, receiver) = futures::channel::mpsc::unbounded::<IceCandidate>();
        let onicecandidate_callback = peer_connection.set_onicecandidate(move |event| {
            match event.candidate() {
                Some(candidate) => sender.unbounded_send(candidate.into()).unwrap(),
                None => sender.close_channel(),
            }
        });
        (
            PeerData {
                pc: peer_connection,
                dc: data_channel,
                name: peer_name,
                callbacks: vec![
                    Box::new(onconnectionstatechange_callback),
                    Box::new(onicecandidate_callback),
                    Box::new(onopen_callback),
                    Box::new(onclose_callback),
                    Box::new(onclosing_callback),
                    Box::new(onmessage_callback),
                    Box::new(onerror_callback)],
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
        let configuration = ConfigurationBuilder::new()
            .add_stun_server("stun:stun.l.google.com:19302")
            .add_stun_server("stun:stun1.l.google.com:19302")
            .build();
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            log::info!("Received websocket message.");
            let configuration = configuration.clone();
            match event.data().into() {
                SocketMessage::ConnectSuccess {
                    lobby_id,
                    assigned_id,
                    peers_id
                } => {
                    log::info!("Invite code to lobby: https://joongle.dev/yahtzee?lobby_id={lobby_id}");
                    peer_network.0.borrow_mut().id = assigned_id;
                    for peer_id in peers_id {
                        let username = username.clone();
                        let websocket = websocket.clone();
                        let peer_network = peer_network.clone();
                        let configuration = configuration.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            let channel_id = (peer_id as u16) << 8 | assigned_id as u16;
                            let (peer_data, candidates) = peer_network.create_peer_data(configuration, "".into(), peer_id, channel_id);
                            let offer_sdp = peer_data.pc.create_offer_sdp().await;
                            peer_network.0.borrow_mut().peers[peer_id as usize] = Some(peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: assigned_id,
                                target: peer_id,
                                username,
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
                    ice_candidates
                } => {
                    if let Some(peer_data) = peer_network.0.borrow_mut().peers[peer_id as usize].as_mut() {
                        log::info!("Received SDP answer from {peer_name}");
                        peer_data.name = peer_name;
                        let peer_connection = peer_data.pc.clone();
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
                            let channel_id = (peer_id as u16) << 8 | user_id as u16;
                            let (peer_data, candidates) = peer_network.create_peer_data(configuration, peer_name, peer_id, channel_id);
                            peer_data.pc.receive_offer_sdp(sdp).await;
                            let answer_sdp = peer_data.pc.create_answer_sdp().await;
                            for ice_candidate in ice_candidates {
                                peer_data.pc.add_ice_candidate(ice_candidate).await;
                            }
                            peer_network.0.borrow_mut().peers[peer_id as usize] = Some(peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: user_id,
                                target: peer_id,
                                username,
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