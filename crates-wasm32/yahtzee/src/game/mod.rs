pub mod events;
use events::GameEvent;

mod main;
use main::Main;

mod connecting;
use connecting::Connecting;

mod lobby;
use lobby::Lobby;

use crate::render::Renderer;
use crate::event_loop::EventSender;

pub enum GameState {
    Main(Main),
    Connecting(Connecting),
    Lobby(Lobby),
    None,
}

pub struct Game {
    state: GameState,
    renderer: Renderer,
}

impl Game {
    pub fn new(renderer: Renderer, event_sender: EventSender<GameEvent>) -> Self {
        Self {
            state: GameState::Main(Main::new(event_sender)),
            renderer,
        }
    }
    pub fn update(&mut self, timestamp: f64) {
        if let Err(err) = self.renderer.render() {
            panic!("Surface error: {:?}", err);
        }
        match &mut self.state {
            GameState::Lobby(state) => state.update(timestamp),
            _ => {}
        }
    }
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::ChangeGameState(state) => {
                self.state = state;
            },
            GameEvent::WebSocketEvent(message) => match &mut self.state {
                GameState::Connecting(state) => state.web_socket_event(message),
                GameState::Lobby(state) => state.web_socket_event(message),
                _ => {}
            },
            GameEvent::PeerNetworkEvent(message) => match &mut self.state {
                GameState::Lobby(state) => state.peer_network_event(message),
                _ => {}
            },
        }
    }
    pub fn mousedown(&mut self, offset: (f32, f32)) {
    }
}