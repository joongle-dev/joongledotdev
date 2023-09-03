use wasm_bindgen::prelude::*;
use web_sys::{HtmlButtonElement, HtmlInputElement, MouseEvent};
use serde::{Deserialize, Serialize};
use std::{rc::Rc, cell::RefCell};

mod graphics;
use graphics::Renderer;

mod platform;
use platform::{EventLoop, Event, PlatformEvent};

mod networks;
use networks::peernetwork::{PeerNetwork, PeerHandshake, PeerMessage};
use crate::networks::websocket::{WebSocket, WebSocketMessage};

#[derive(Serialize, Deserialize)]
enum SocketMessage {
    ConnectSuccess {
        lobby_id: u64,
        user_id: u32,
        peers_id: Vec<u32>,
    },
    WebRtcHandshake {
        source_id: u32,
        target_id: u32,
        sdp_description: String,
        ice_candidates: Vec<(String, Option<String>, Option<u16>)>,
    }
}

enum CustomEvent {
    JoinedLobby(u64, u32, Vec<u32>),
    Ping,
    ReceiveHandshake(PeerHandshake),
    PeerMessage(PeerMessage),
}

#[wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger!");

    let window = web_sys::window().unwrap_throw();

    let location = window.location();
    let protocol = location.protocol().unwrap_throw();
    let host = location.host().unwrap_throw();
    let path = location.pathname().unwrap_throw();
    let search = location.search().unwrap_throw();
    let ws_protocol = if protocol.contains("https:") { "wss:" } else { "ws:" };
    let ws_address = format!("{ws_protocol}//{host}{path}ws{search}");
    let invite_code = format!("{protocol}//{host}{path}?lobby_id=");

    let document = window.document().unwrap_throw();
    let name_input = document.get_element_by_id("name-input").unwrap_throw().dyn_into::<HtmlInputElement>().unwrap_throw();
    let join_btn = document.get_element_by_id("name-submit-btn").unwrap_throw().dyn_into::<HtmlButtonElement>().unwrap_throw();
    let ping_btn = document.get_element_by_id("ping-btn").unwrap_throw().dyn_into::<HtmlButtonElement>().unwrap_throw();

    let event_loop = EventLoop::new();

    let websocket: Rc<RefCell<Option<WebSocket>>> = Rc::new(RefCell::new(None));
    let mut peer_network = PeerNetwork::new();
    let mut renderer = Renderer::new(canvas.clone()).await;

    //Peer network handshake send callback setup
    let websocket_clone = websocket.clone();
    peer_network.set_handshake_callback(move |handshake| {
        let message = SocketMessage::WebRtcHandshake {
            source_id: handshake.source_id,
            target_id: handshake.target_id,
            sdp_description: handshake.sdp_description,
            ice_candidates: handshake.ice_candidates.into_iter().map(|v| (v.into())).collect(),
        };
        let serialized = bincode::serialize(&message).unwrap();
        websocket_clone.borrow().as_ref().unwrap_throw().send_with_u8_array(serialized.as_slice());
    });

    //Peer network message callback setup
    let event_handler_proxy = event_loop.event_handler_proxy();
    peer_network.set_message_callback(move |message| {
        event_handler_proxy.call(CustomEvent::PeerMessage(message));
    });

    //Join Lobby button setup
    let event_handler_proxy = event_loop.event_handler_proxy();
    let join_btn_clone = join_btn.clone();
    let ping_btn_clone = ping_btn.clone();
    let websocket_clone = websocket.clone();
    let join_btn_callback = Closure::<dyn FnMut(MouseEvent)>::new(move |_: MouseEvent| {
        let event_handler_proxy = event_handler_proxy.clone();
        join_btn_clone.set_hidden(true);
        ping_btn_clone.set_hidden(false);
        websocket_clone.borrow_mut().replace(WebSocket::new(ws_address.as_str(), move |message| {
            match message {
                WebSocketMessage::String(_) => {}
                WebSocketMessage::Binary(message) => {
                    match bincode::deserialize::<SocketMessage>(message.as_slice()).unwrap() {
                        SocketMessage::ConnectSuccess { lobby_id, user_id: assigned_id, peers_id } => {
                            event_handler_proxy.call(CustomEvent::JoinedLobby(lobby_id, assigned_id, peers_id));
                        }
                        SocketMessage::WebRtcHandshake { source_id: source, target_id: target, sdp_description, ice_candidates } => {
                            event_handler_proxy.call(CustomEvent::ReceiveHandshake(PeerHandshake{
                                source_id: source,
                                target_id: target,
                                sdp_description,
                                ice_candidates: ice_candidates.into_iter().map(|v| v.into()).collect()
                            }));
                        }
                    }
                }
            }
        }));
    });
    join_btn.set_onclick(Some(join_btn_callback.as_ref().unchecked_ref()));
    join_btn_callback.forget();

    //Ping button setup
    let event_handler_proxy = event_loop.event_handler_proxy();
    let ping_btn_callback = Closure::<dyn FnMut(web_sys::MouseEvent)>::new(move |_event: web_sys::MouseEvent| {
        event_handler_proxy.call(CustomEvent::Ping);
    });
    ping_btn.set_onclick(Some(ping_btn_callback.as_ref().unchecked_ref()));
    ping_btn_callback.forget();

    //Run application
    event_loop.run(canvas, move |event: Event<CustomEvent>| {
        match event {
            Event::PlatformEvent(event) => {
                match event {
                    PlatformEvent::AnimationFrame { .. } => {
                        if let Err(err) = renderer.render() {
                            panic!("Surface error: {err:?}");
                        }
                    },
                    PlatformEvent::MouseDown { offset, .. } => {
                        peer_network.broadcast_str(format!("Click ({}, {})!", offset.0, offset.1).as_str());
                    },
                    _ => {}
                }
            }
            Event::UserEvent(event) => {
                match event {
                    CustomEvent::JoinedLobby(lobby_id, user_id, peers_id) => {
                        log::info!("Lobby invite code: {invite_code}{lobby_id}");
                        peer_network.set_user_id(user_id);
                        for peer_id in peers_id {
                            peer_network.initiate_handshake(peer_id);
                        }
                    }
                    CustomEvent::ReceiveHandshake(handshake) => {
                        peer_network.receive_handshake(handshake);
                    }
                    CustomEvent::Ping => {
                        peer_network.broadcast_str(format!("Ping from {}", name_input.value()).as_str());
                    }
                    CustomEvent::PeerMessage(message) => {
                        match message {
                            PeerMessage::String(data) => log::info!("{data}"),
                            PeerMessage::Binary(_) => {}
                        }
                    }
                }
            }
        }
        true
    });
}