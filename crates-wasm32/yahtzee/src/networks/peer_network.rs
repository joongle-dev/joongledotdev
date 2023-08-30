use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent, RtcPeerConnectionState, WebSocket};
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
        for (peer_id, peer) in self.0.borrow().peers.iter().enumerate().filter_map(|(i, v)| v.as_ref().map(|v| (i, v))) {
            log::info!("Sending \"{}\" to {}; state: {:?}", data, peer_id, peer.dc.ready_state());
            peer.dc.send_str(data);
        }
    }
    fn create_peer_data(&self, configuration: Configuration, peer_id: u8) -> (PeerData, UnboundedReceiver<IceCandidate>) {
        let peer_connection = PeerConnection::new_with_configuration(configuration);
        let peer_connection_clone = peer_connection.clone();
        let peer_network_clone = self.0.clone();
        let onconnectionstatechange_callback = peer_connection.set_onconnectionstatechange(move || {
            match peer_connection_clone.connection_state() {
                RtcPeerConnectionState::Closed | RtcPeerConnectionState::Failed | RtcPeerConnectionState::Disconnected => {
                    let peer_data_ref = &mut peer_network_clone.borrow_mut().peers[peer_id as usize];
                    if peer_data_ref.is_some() {
                        log::info!("Connection to {} closed.", peer_id);
                        let _ = peer_data_ref.take();
                    }
                }
                _ => {}
            }
        });
        let data_channel = peer_connection.create_data_channel_negotiated("Data Channel", 0);
        let onopen_callback = data_channel.set_onopen(move || {
            log::info!("Data Channel to {} opened!", peer_id);
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
                callbacks: vec![
                    Box::new(onconnectionstatechange_callback),
                    Box::new(onicecandidate_callback),
                    Box::new(onopen_callback),
                    Box::new(onmessage_callback)],
            },
            receiver
        )
    }
    pub fn connect(&self, address: String) {
        let websocket = WebSocket::new(address.as_str()).unwrap();
            websocket.set_binary_type(BinaryType::Arraybuffer);
        let websocket_clone = websocket.clone();
        let peer_network = self.clone();
        let configuration = ConfigurationBuilder::new()
            .add_turn_server("turn:turn.joongle.dev:3478", "guest", "guest1234")
            .add_turn_server("turn:turn.joongle.dev:5349", "guest", "guest1234")
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
                    log::info!("Invite code to lobby: http://localhost/yahtzee?lobby_id={lobby_id}");
                    peer_network.0.borrow_mut().id = assigned_id;
                    for peer_id in peers_id {
                        let websocket = websocket.clone();
                        let peer_network = peer_network.clone();
                        let configuration = configuration.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            //let channel_id = (peer_id as u16) << 8 | assigned_id as u16;
                            let (peer_data, candidates) = peer_network.create_peer_data(configuration, peer_id);
                            let offer_sdp = peer_data.pc.create_offer_sdp().await;
                            peer_network.0.borrow_mut().peers[peer_id as usize] = Some(peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: assigned_id,
                                target: peer_id,
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
                    sdp_description: sdp,
                    ice_candidates
                } => {
                    if let Some(peer_data) = peer_network.0.borrow_mut().peers[peer_id as usize].as_mut() {
                        log::info!("Received SDP answer from {peer_id}");
                        let peer_connection = peer_data.pc.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            peer_connection.receive_answer_sdp(sdp).await;
                            for ice_candidate in ice_candidates {
                                peer_connection.add_ice_candidate(ice_candidate).await;
                            }
                        });
                    }
                    else {
                        log::info!("Received SDP offer from {peer_id}");
                        let websocket = websocket.clone();
                        let peer_network = peer_network.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            //let channel_id = (peer_id as u16) << 8 | user_id as u16;
                            let (peer_data, candidates) = peer_network.create_peer_data(configuration, peer_id);
                            peer_data.pc.receive_offer_sdp(sdp).await;
                            let answer_sdp = peer_data.pc.create_answer_sdp().await;
                            for ice_candidate in ice_candidates {
                                peer_data.pc.add_ice_candidate(ice_candidate).await;
                            }
                            peer_network.0.borrow_mut().peers[peer_id as usize] = Some(peer_data);
                            let message = SocketMessage::WebRtcHandshake {
                                source: user_id,
                                target: peer_id,
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