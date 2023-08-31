use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, RtcPeerConnectionIceEvent};
use std::{rc::Rc, cell::RefCell, collections::BTreeMap};
use crate::networks::webrtc::{Configuration, ConfigurationBuilder, DataChannel, PeerConnection};
pub use crate::networks::webrtc::IceCandidate;

pub struct PeerHandshakeData {
    source_id: u32,
    target_id: u32,
    sdp_description: String,
    ice_candidates: Vec<IceCandidate>,
}
enum PeerStatus {
    Connecting(PeerHandshakeData),
    Connected,
}
struct PeerData {
    status: PeerStatus,
    connection: PeerConnection,
    channel: DataChannel,
    connection_state_callback: Closure<dyn FnMut()>,
    ice_candidate_callback: Closure<dyn FnMut(RtcPeerConnectionIceEvent)>,
    open_callback: Closure<dyn FnMut()>,
    message_callback: Closure<dyn FnMut(MessageEvent)>,
}
struct PeerNetworkData {
    id: u32,
    peers: BTreeMap<u32, PeerData>,
    configuration: Configuration,
}
pub struct PeerNetwork {
    network_data: Rc<RefCell<PeerNetworkData>>,
}
impl PeerNetwork {
    pub fn new() -> Self {
        let configuration = ConfigurationBuilder::new()
            .add_turn_server("turn:turn.joongle.dev:3478", "guest", "guest1234")
            .add_turn_server("turn:turn.joongle.dev:5349", "guest", "guest1234")
            .build();
        Self {
            network_data: Rc::new(RefCell::new(PeerNetworkData {
                id: 0,
                peers: BTreeMap::new(),
                configuration,
            }))
        }
    }
    pub fn set_id(&self, id: u32) {
        self.network_data.borrow_mut().id = id;
    }
    pub async fn create_handshake_data<F: FnMut(PeerHandshakeData)>(&self, peer_id: u32, ) -> PeerHandshakeData {

        todo!()
    }
}