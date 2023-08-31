use wasm_bindgen::prelude::*;
use web_sys::{MessageEvent, RtcPeerConnectionIceEvent};
use std::{rc::Rc, cell::RefCell, collections::BTreeMap};
use crate::networks::webrtc::{Configuration, ConfigurationBuilder, DataChannel, PeerConnection};
use crate::util::fixed_ring_buffer::FixedRingBuffer;
pub use crate::networks::webrtc::IceCandidate;

pub enum PeerMessage {
    Text(String),
    Binary(Vec<u8>),
}
pub struct HandshakeData {
    source_id: u32,
    target_id: u32,
    sdp_description: String,
    ice_candidates: Vec<IceCandidate>,
}
pub enum PeerNetworkEvent {
    Handshake(HandshakeData),
    Message(PeerMessage),
}
enum PeerStatus {
    Connecting(HandshakeData),
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
    event_buffer: FixedRingBuffer<PeerNetworkEvent, 128>,
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
                event_buffer: FixedRingBuffer::new(),
            }))
        }
    }
    pub fn next(&self) -> Option<PeerNetworkEvent> {
        self.network_data.borrow_mut().event_buffer.pop_back().ok()
    }
    pub fn set_id(&self, id: u32) {
        self.network_data.borrow_mut().id = id;
    }
    pub fn initiate_handshake(&self, peer_id: u32) {

    }
}