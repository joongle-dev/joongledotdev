use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::{mpsc::{Sender, Receiver}};
use serde::{Deserialize, Serialize};
use web_sys::{WebSocket, RtcPeerConnection, MessageEvent, RtcSessionDescription, RtcSessionDescriptionInit, RtcSdpType};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;

pub type LobbyID = u64;
type UserID = u8;

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess{
        lobby_id: LobbyID,
        user_id: UserID,
        existing_users: Vec<UserID>,
    },
    SdpOffer{
        source: UserID,
        target: UserID,
        name: String,
        sdp: String,
    },
    SdpAnswer{
        source: UserID,
        target: UserID,
        name: String,
        sdp: String,
    }
}

struct Peer {
    name: String,
    connection: RtcPeerConnection,
}
impl Peer {
    fn new() -> Self {
        let connection = match RtcPeerConnection::new() {
            Ok(connection) => connection,
            Err(err) => panic!("Failed to create peer connection: {err:?}"),
        };
        Self {
            name: "".into(),
            connection,
        }
    }
    async fn create_sdp_offer(&self) -> String {
        let offer = match JsFuture::from(self.connection.create_offer()).await {
            Ok(offer) => offer,
            Err(err) => panic!("Failed to sdp create offer: {err:?}"),
        };
        let offer_sdp = js_sys::Reflect::get(&offer, &JsValue::from_str("sdp"))
            .expect("Failed to retrieve sdp field from RtcSessionDescription.")
            .as_string()
            .expect("Sdp is field not a string.");
        let offer_description = RtcSessionDescriptionInit::from(offer);
        if JsFuture::from(self.connection.set_local_description(&offer_description)).await.is_err() {
            panic!("Failed to set local description");
        }
        offer_sdp
    }
    async fn create_sdp_answer(&mut self, name: String, offer_sdp: String) -> String {
        self.name = name;
        let mut offer_description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
        offer_description.sdp(&offer_sdp);
        if JsFuture::from(self.connection.set_remote_description(&offer_description)).await.is_err() {
            panic!("Failed to set remote description");
        }
        let answer = match JsFuture::from(self.connection.create_answer()).await {
            Ok(answer) => answer,
            Err(err) => panic!("Failed to create sdp answer: {err:?}"),
        };
        let answer_sdp = js_sys::Reflect::get(&answer, &JsValue::from_str("sdp"))
            .expect("Failed to retrieve sdp field from RtcSessionDescription.")
            .as_string()
            .expect("Sdp is field not a string.");
        let answer_description = RtcSessionDescriptionInit::from(answer);
        if JsFuture::from(self.connection.set_local_description(&answer_description)).await.is_err() {
            panic!("Failed to set local description");
        }
        answer_sdp
    }
    async fn receive_sdp_answer(&mut self, name: String, answer_sdp: String) {
        self.name = name;
        let mut answer_description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
        answer_description.sdp(&answer_sdp);
        if JsFuture::from(self.connection.set_remote_description(&answer_description)).await.is_err() {
            panic!("Failed to set remote description");
        }
    }
}

pub struct LobbyState {
    name: String,
    user_id: u8,
    lobby_id: u64,
    peers: BTreeMap<UserID, Peer>,
}
impl LobbyState {
    pub fn new(name: &str, address: &str) -> Result<Rc<RefCell<Self>>, JsValue> {
        let lobby_state = Rc::new(RefCell::new(Self {
            name: name.into(),
            user_id: 0,
            lobby_id: 0,
            peers: BTreeMap::new(),
        }));
        let websocket = WebSocket::new(address)?;
        let onmessage_callback: Closure<dyn FnMut(MessageEvent)> = {
            log::info!("Received websocket message.");
            let websocket = websocket.clone();
            let lobby_state = lobby_state.clone();
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(buffer) = event.data().dyn_into::<js_sys::ArrayBuffer>() {
                    let u8arr = js_sys::Uint8Array::new(&buffer);
                    let u8vec: Vec<u8> = u8arr.to_vec();
                    if let Ok(message) = bincode::deserialize::<SocketMessage>(&u8vec) {
                        let lobby_state = lobby_state.clone();
                        let websocket = websocket.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match message {
                                SocketMessage::ConnectSuccess { lobby_id, user_id, existing_users } => {
                                    log::info!("Invite code to lobby: https://joongle.dev/yahtzee?lobby_id={lobby_id}");
                                    let name = {
                                        let mut lobby_state = lobby_state.borrow_mut();
                                        lobby_state.lobby_id = lobby_id;
                                        lobby_state.user_id = user_id;
                                        lobby_state.name.clone()
                                    };
                                    for peer_id in existing_users {
                                        let peer = Peer::new();
                                        let offer_sdp = peer.create_sdp_offer().await;
                                        lobby_state.borrow_mut().peers.insert(peer_id, peer);
                                        let message = SocketMessage::SdpOffer {
                                            source: user_id,
                                            target: peer_id,
                                            name: name.clone(),
                                            sdp: offer_sdp
                                        };
                                        let serialized = bincode::serialize(&message)
                                            .expect("Failed to serialize socket message.");
                                        websocket.send_with_u8_array(&serialized)
                                            .expect("Failed to send socket message.");
                                    }
                                }
                                SocketMessage::SdpOffer { source, name, sdp, .. } => {
                                    log::info!("Received SDP offer from {name}: {sdp}");
                                    let mut peer = Peer::new();
                                    let answer_sdp = peer.create_sdp_answer(name, sdp).await;
                                    lobby_state.borrow_mut().peers.insert(source, peer);
                                    let message = SocketMessage::SdpAnswer {
                                        source: lobby_state.borrow().user_id,
                                        target: source,
                                        name: lobby_state.borrow().name.clone(),
                                        sdp: answer_sdp,
                                    };
                                    let serialized = bincode::serialize(&message)
                                        .expect("Failed to serialize socket message.");
                                    websocket.send_with_u8_array(&serialized)
                                        .expect("Failed to send socket message.");
                                }
                                SocketMessage::SdpAnswer { source, name, sdp, .. } => {
                                    log::info!("Received SDP answer from {name}: {sdp}");
                                    if let Some(peer) = lobby_state.borrow_mut().peers.get_mut(&source) {
                                        peer.receive_sdp_answer(name, sdp).await;
                                    }
                                }
                            }
                        });
                    }
                    else {
                        panic!("Failed to deserialize message.");
                    }
                }
                else {
                    panic!("Message was not ArrayBuffer.");
                }
            }))
        };
        websocket.set_binary_type(web_sys::BinaryType::Arraybuffer);
        websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        Ok(lobby_state)
    }
}