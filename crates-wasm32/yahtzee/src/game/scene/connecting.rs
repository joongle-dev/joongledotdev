use wasm_bindgen::prelude::*;
use crate::network::{web_socket::WebSocket};
use crate::event_loop::EventSender;
use crate::game::events::{GameEvent, WebSocketEvent, WebSocketMessage};
use super::{GameScene, lobby::Lobby};

pub struct Connecting {
    event_sender: EventSender<GameEvent>,
    web_socket: Option<WebSocket<WebSocketMessage>>,
    name: String,
}
impl Connecting {
    pub fn new(event_sender: EventSender<GameEvent>, name: String) -> Self {
        let window = web_sys::window().unwrap_throw();
        let location = window.location();
        let protocol = location.protocol().unwrap_throw();
        let host = location.host().unwrap_throw();
        let path = location.pathname().unwrap_throw();
        let search = location.search().unwrap_throw();
        let ws_protocol = if protocol.contains("https:") { "wss:" } else { "ws:" };
        let ws_address = format!("{ws_protocol}//{host}{path}ws{search}");

        let event_sender_clone = event_sender.clone();
        let web_socket = WebSocket::new(ws_address.as_str(), move |message| {
            event_sender_clone.send(GameEvent::WebSocketEvent(message));
        });

        Self {
            event_sender,
            web_socket: Some(web_socket),
            name
        }
    }
}
impl GameScene for Connecting {
    fn update(&mut self, time: f64) {
        todo!()
    }

    fn handle_event(&mut self, event: GameEvent) {
        if let GameEvent::WebSocketEvent(event) = event {
            match event {
                WebSocketEvent::Connect => {}
                WebSocketEvent::Disconnect => {}
                WebSocketEvent::Message(WebSocketMessage::ConnectSuccess { lobby_id, user_id, peers_id }) => {
                    self.event_sender.send(GameEvent::ChangeGameScene(Box::new(
                        Lobby::new(self.event_sender.clone(),
                                   self.web_socket.take().unwrap(),
                                   lobby_id,
                                   std::mem::take(&mut self.name),
                                   user_id,
                                   peers_id
                        )
                    )));
                }
                _ => {}
            }
        }
    }
}