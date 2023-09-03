use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, RtcPeerConnectionIceEvent};
use std::{rc::Rc, cell::RefCell, collections::BTreeMap};
use js_sys::{ArrayBuffer, Uint8Array};
use crate::networks::webrtc::{ConfigurationBuilder, Configuration, PeerConnection, PeerConnectionState, DataChannel};
pub use crate::networks::webrtc::IceCandidate;

pub enum PeerMessage {
    String(String),
    Binary(Vec<u8>),
}
#[derive(Default)]
pub struct PeerHandshake {
    pub source_id: u32,
    pub target_id: u32,
    pub sdp_description: String,
    pub ice_candidates: Vec<IceCandidate>,
}
enum PeerStatus {
    Connecting(PeerHandshake),
    Connected,
}
struct PeerData {
    status: PeerStatus,
    peer_connection: PeerConnection,
    data_channel: DataChannel,
    _onconnectionstatechange_callback: Closure<dyn FnMut()>,
    _onicecandidate_callback: Closure<dyn FnMut(RtcPeerConnectionIceEvent)>,
    _onopen_callback: Closure<dyn FnMut()>,
    _onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
}
struct PeerNetworkData {
    peers: BTreeMap<u32, PeerData>,
    handshake_callback: Box<dyn FnMut(PeerHandshake)>,
    message_callback: Box<dyn FnMut(PeerMessage)>,
}
pub struct PeerNetwork {
    user_id: u32,
    configuration: Configuration,
    network_data: Rc<RefCell<PeerNetworkData>>,
}
impl PeerNetwork {
    pub fn new() -> Self {
        let configuration = ConfigurationBuilder::new()
            .add_turn_server("turn:turn.joongle.dev:3478", "guest", "guest1234")
            .add_turn_server("turn:turn.joongle.dev:5349", "guest", "guest1234")
            .build();
        Self {
            user_id: 0,
            configuration,
            network_data: Rc::new(RefCell::new(PeerNetworkData {
                peers: BTreeMap::new(),
                handshake_callback: Box::new(move |_: PeerHandshake| log::info!("Peer Network handshake callback is not initialized.")),
                message_callback: Box::new(move |_: PeerMessage| log::info!("Peer Network message callback is not initialized.")),
            }))
        }
    }
    pub fn set_user_id(&mut self, id: u32) {
        self.user_id = id;
    }
    pub fn broadcast_str(&self, data: &str) {
        for peer in self.network_data.borrow().peers.values().filter(|peer| matches!(peer.status, PeerStatus::Connected)) {
            peer.data_channel.send_str(data);
        }
    }
    pub fn initiate_handshake(&self, peer_id: u32) {
        let mut peer_data = self.create_peer_data(peer_id);
        let peer_network_clone = self.network_data.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let PeerStatus::Connecting(ref mut handshake_data) = peer_data.status {
                handshake_data.sdp_description = peer_data.peer_connection.create_offer_sdp().await;
            }
            peer_network_clone.borrow_mut().peers.insert(peer_id, peer_data);
        });
    }
    pub fn receive_handshake(&self, mut handshake: PeerHandshake) {
        let peer_network_clone = self.network_data.clone();
        if self.network_data.borrow().peers.contains_key(&handshake.source_id) {
            log::info!("Received handshake answer from {}", handshake.source_id);
            wasm_bindgen_futures::spawn_local(async move {
                if let Some(
                    PeerData {
                        peer_connection,
                        status: PeerStatus::Connecting(handshake_out),
                        ..
                    }
                ) = peer_network_clone.borrow_mut().peers.get_mut(&handshake.source_id) {
                    peer_connection.receive_answer_sdp(handshake.sdp_description.as_str()).await;
                    for ice_candidate in std::mem::take(&mut handshake_out.ice_candidates) {
                        peer_connection.add_ice_candidate(ice_candidate).await;
                    }
                }
            });
        }
        else {
            log::info!("Received handshake offer from {}", handshake.source_id);
            let mut peer_data = self.create_peer_data(handshake.source_id);
            wasm_bindgen_futures::spawn_local(async move {
                peer_data.peer_connection.receive_offer_sdp(handshake.sdp_description.as_str()).await;
                for ice_candidate in std::mem::take(&mut handshake.ice_candidates) {
                    peer_data.peer_connection.add_ice_candidate(ice_candidate).await;
                }
                if let PeerStatus::Connecting(ref mut handshake_data) = peer_data.status {
                    handshake_data.sdp_description = peer_data.peer_connection.create_answer_sdp().await;
                }
                peer_network_clone.borrow_mut().peers.insert(handshake.source_id, peer_data);
            });
        }
    }
    pub fn set_handshake_callback<F: FnMut(PeerHandshake) + 'static>(&self, f: F) {
        self.network_data.borrow_mut().handshake_callback = Box::new(f);
    }
    pub fn set_message_callback<F: FnMut(PeerMessage) + 'static>(&self, f: F) {
        self.network_data.borrow_mut().message_callback = Box::new(f);
    }
    fn create_peer_data(&self, peer_id: u32) -> PeerData {
        //Create peer connection and data channel.
        let peer_connection = PeerConnection::new_with_configuration(&self.configuration);
        let data_channel = peer_connection.create_data_channel_negotiated("Data Channel", 0);

        //Initialize peer connection connectionstatechange event handler.
        let peer_connection_clone = peer_connection.clone();
        let peer_network_clone = self.network_data.clone();
        let _onconnectionstatechange_callback = peer_connection.set_onconnectionstatechange(move || {
            if let PeerConnectionState::Closed | PeerConnectionState::Failed | PeerConnectionState::Disconnected = peer_connection_clone.connection_state() {
                if let Some(peer_data) = peer_network_clone.borrow_mut().peers.remove(&peer_id) {
                    log::info!("Connection to {peer_id} closed.");
                    peer_data.data_channel.close();
                    peer_data.peer_connection.close();
                }
            }
        });

        //Initialize peer connection icecandidate event handler.
        let peer_network_clone = self.network_data.clone();
        let _onicecandidate_callback = peer_connection.set_onicecandidate(move |event| {
            let mut peer_network_ref = peer_network_clone.borrow_mut();
            let handshake_data = {
                if let Some(PeerData { status: PeerStatus::Connecting(handshake_data), .. }) = peer_network_ref.peers.get_mut(&peer_id) {
                    if let Some(candidate) = event.candidate() {
                        //ICE candidate discovered, push into peer's candidate list.
                        handshake_data.ice_candidates.push(candidate.into());
                        return;
                    } else {
                        //No more ICE candidates to discover, send handshake.
                        log::info!("{} ICE candidates gathered, sending handshake.", handshake_data.ice_candidates.len());
                        std::mem::take(handshake_data)
                    }
                } else {
                    log::warn!("ICE candidate discovered, but peer is not in connecting state.");
                    return;
                }
            };
            peer_network_ref.handshake_callback.as_mut()(handshake_data);
        });

        //Initialize data channel open event handler.
        let peer_network_clone = self.network_data.clone();
        let _onopen_callback = data_channel.set_onopen(move || {
            if let Some(peer_data) = peer_network_clone.borrow_mut().peers.get_mut(&peer_id) {
                log::info!("Data Channel to {} opened!", peer_id);
                peer_data.status = PeerStatus::Connected;
            }
        });

        //Initialize data channel message event handler.
        let peer_network_clone = self.network_data.clone();
        let _onmessage_callback = data_channel.set_onmessage(move |event| {
            if let Some(data) = event.data().as_string() {
                peer_network_clone.borrow_mut().message_callback.as_mut()(PeerMessage::String(data));
            }
            else if let Ok(data) = event.data().dyn_into::<ArrayBuffer>() {
                peer_network_clone.borrow_mut().message_callback.as_mut()(PeerMessage::Binary(Uint8Array::new(&data).to_vec()));
            }
            else {
                log::warn!("Unhandled DataChannel message type.");
            }
        });
        
        PeerData {
            status: PeerStatus::Connecting(PeerHandshake {
                source_id: self.user_id,
                target_id: peer_id,
                ..Default::default()
            }),
            peer_connection,
            data_channel,
            _onconnectionstatechange_callback,
            _onicecandidate_callback,
            _onopen_callback,
            _onmessage_callback,
        }
    }
}