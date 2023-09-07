use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, RtcPeerConnectionIceEvent};
use std::{rc::Rc, cell::RefCell, collections::BTreeMap};
use js_sys::{ArrayBuffer, Uint8Array};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use crate::network::webrtc::{ConfigurationBuilder, Configuration, PeerConnection, PeerConnectionState, DataChannel};

#[derive(Default)]
pub struct PeerHandshake {
    pub source_id: u32,
    pub target_id: u32,
    pub sdp_description: String,
    pub ice_candidates: Vec<(String, Option<String>, Option<u16>)>,
}

#[derive(Serialize, Deserialize)]
struct MessageWrapper<T>(u32, T);

pub enum PeerNetworkEvent<T> {
    Handshake(PeerHandshake),
    Connect(u32),
    Disconnect(u32),
    Message(u32, T),
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

pub struct PeerNetwork<T> {
    user_id: u32,
    configuration: Configuration,
    peer_map: Rc<RefCell<BTreeMap<u32, PeerData>>>,
    event_callback: Rc<RefCell<dyn FnMut(PeerNetworkEvent<T>)>>,
}

impl<T: Serialize + DeserializeOwned + 'static> PeerNetwork<T> {
    pub fn new<F: FnMut(PeerNetworkEvent<T>) + 'static>(user_id: u32, event_handler: F) -> Self {
        let configuration = ConfigurationBuilder::new()
            .add_turn_server("turn:turn.joongle.dev:3478", "guest", "guest1234")
            .add_turn_server("turn:turn.joongle.dev:5349", "guest", "guest1234")
            .build();
        Self {
            user_id,
            configuration,
            peer_map: Rc::new(RefCell::new(BTreeMap::new())),
            event_callback: Rc::new(RefCell::new(event_handler)),
        }
    }
    pub fn user_id(&self) -> u32 {
        self.user_id
    }
    pub fn broadcast(&self, message: &T) {
        let serialized = bincode::serialize(&message).unwrap();
        for peer in self.peer_map.borrow().values().filter(|peer| matches!(peer.status, PeerStatus::Connected)) {
            peer.data_channel.send_u8_array(serialized.as_slice());
        }
    }
    pub fn send(&self, peer_id: u32, message: &T) {
        let serialized = bincode::serialize(&MessageWrapper(self.user_id, message)).unwrap();
        if let Some(peer) = self.peer_map.borrow().get(&peer_id) {
            peer.data_channel.send_u8_array(serialized.as_slice());
        }
    }
    pub fn initiate_handshake(&self, peer_id: u32) {
        let mut peer_data = self.create_peer_data(peer_id);
        let peer_network_clone = self.peer_map.clone();
        wasm_bindgen_futures::spawn_local(async move {
            log::info!("Initiating handshake to {}", peer_id);
            if let PeerStatus::Connecting(ref mut handshake_data) = peer_data.status {
                handshake_data.sdp_description = peer_data.peer_connection.create_offer_sdp().await;
            }
            peer_network_clone.borrow_mut().insert(peer_id, peer_data);
        });
    }
    pub fn receive_handshake(&self, mut handshake: PeerHandshake) {
        if let Some(peer_connection) = self.peer_map.borrow().get(&handshake.source_id).map(|v| v.peer_connection.clone()) {
            log::info!("Received handshake answer from {}", handshake.source_id);
            wasm_bindgen_futures::spawn_local(async move {
                peer_connection.receive_answer_sdp(handshake.sdp_description.as_str()).await;
                for ice_candidate in std::mem::take(&mut handshake.ice_candidates) {
                    peer_connection.add_ice_candidate(ice_candidate.into()).await;
                }
            });
        }
        else {
            log::info!("Received handshake offer from {}", handshake.source_id);
            let peer_map = self.peer_map.clone();
            let mut peer_data = self.create_peer_data(handshake.source_id);
            wasm_bindgen_futures::spawn_local(async move {
                peer_data.peer_connection.receive_offer_sdp(handshake.sdp_description.as_str()).await;
                for ice_candidate in std::mem::take(&mut handshake.ice_candidates) {
                    peer_data.peer_connection.add_ice_candidate(ice_candidate.into()).await;
                }
                if let PeerStatus::Connecting(ref mut handshake_data) = peer_data.status {
                    handshake_data.sdp_description = peer_data.peer_connection.create_answer_sdp().await;
                }
                peer_map.borrow_mut().insert(handshake.source_id, peer_data);
            });
        }
    }
    fn create_peer_data(&self, peer_id: u32) -> PeerData {
        //Create peer connection and data channel.
        let peer_connection = PeerConnection::new_with_configuration(&self.configuration);
        let data_channel = peer_connection.create_data_channel_negotiated("Data Channel", 0);

        //Initialize peer connection connectionstatechange event handler.
        let peer_map = self.peer_map.clone();
        let event_callback = self.event_callback.clone();
        let peer_connection_clone = peer_connection.clone();
        let _onconnectionstatechange_callback = peer_connection.set_onconnectionstatechange(move || {
            if let PeerConnectionState::Closed | PeerConnectionState::Failed | PeerConnectionState::Disconnected = peer_connection_clone.connection_state() {
                if let Some(peer_data) = peer_map.borrow_mut().remove(&peer_id) {
                    log::info!("Connection to {peer_id} closed.");
                    event_callback.borrow_mut()(PeerNetworkEvent::Disconnect(peer_id));
                    peer_data.data_channel.close();
                    peer_data.peer_connection.close();
                }
            }
        });

        //Initialize peer connection icecandidate event handler.
        let peer_map = self.peer_map.clone();
        let event_callback = self.event_callback.clone();
        let _onicecandidate_callback = peer_connection.set_onicecandidate(move |event| {
            let mut handshake = if let Some(PeerData { status: PeerStatus::Connecting(handshake_data), .. }) = peer_map.borrow_mut().get_mut(&peer_id) {
                if let Some(candidate) = event.candidate() {
                    //ICE candidate discovered, push into peer's candidate list.
                    handshake_data.ice_candidates.push((candidate.candidate(), candidate.sdp_mid(), candidate.sdp_m_line_index()));
                    return;
                } else {
                    //No more ICE candidates to discover, send handshake.
                    log::info!("{} ICE candidates gathered, sending handshake.", handshake_data.ice_candidates.len());
                    std::mem::take(handshake_data)
                }
            }
            else {
                PeerHandshake::default()
            };
            event_callback.borrow_mut()(PeerNetworkEvent::Handshake(std::mem::take(&mut handshake)));
        });

        //Initialize data channel open event handler.
        let peer_map = self.peer_map.clone();
        let event_callback = self.event_callback.clone();
        let _onopen_callback = data_channel.set_onopen(move || {
            if let Some(peer_data) = peer_map.borrow_mut().get_mut(&peer_id) {
                log::info!("Data Channel to {} opened!", peer_id);
                peer_data.status = PeerStatus::Connected;
            }
            event_callback.borrow_mut()(PeerNetworkEvent::Connect(peer_id));
        });

        //Initialize data channel message event handler.
        let event_callback = self.event_callback.clone();
        let _onmessage_callback = data_channel.set_onmessage(move |event| {
            if let Ok(data) = event.data().dyn_into::<ArrayBuffer>() {
                let data = Uint8Array::new(&data).to_vec();
                let message: MessageWrapper<T> = bincode::deserialize(data.as_slice()).unwrap();
                event_callback.borrow_mut()(PeerNetworkEvent::Message(message.0, message.1));
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