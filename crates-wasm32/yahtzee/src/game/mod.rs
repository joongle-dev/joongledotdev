pub mod events;
use events::Event;

mod main;
use main::Main;

mod connecting;
use connecting::Connecting;

mod lobby;
use lobby::Lobby;

use crate::graphics::Renderer;
use events::EventSender;

pub struct Context {
    pub renderer: Renderer,
    pub event_sender: EventSender,
}

enum GameState {
    Main(Main),
    Connecting(Connecting),
    Lobby(Lobby),
    None,
}

pub struct Game {
    state: GameState,
    ctx: Context,
}

impl Game {
    pub fn new(renderer: Renderer, event_sender: EventSender) -> Self {
        let mut ctx = Context {
            renderer,
            event_sender,
        };
        Self {
            state: GameState::Main(Main::new(&mut ctx)),
            ctx,
        }
    }
    pub fn update(&mut self, timestamp: f64) {
        if let Err(err) = self.ctx.renderer.render() {
            panic!("Surface error: {:?}", err);
        }
        match &mut self.state {
            GameState::Lobby(state) => state.update(timestamp),
            _ => {}
        }
    }
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::SubmitName(name) => {
                self.state = GameState::Connecting(Connecting::new(&mut self.ctx, name))
            },
            Event::LobbyJoin(lobby) => if let GameState::Connecting(state) = std::mem::replace(&mut self.state, GameState::None) {
                self.state = GameState::Lobby(Lobby::new(&mut self.ctx, lobby, state.web_socket, state.name));
            },
            Event::WebSocketEvent(message) => match &mut self.state {
                GameState::Connecting(state) => state.web_socket_event(message),
                GameState::Lobby(state) => state.web_socket_event(message),
                _ => {}
            },
            Event::PeerNetworkEvent(message) => match &mut self.state {
                GameState::Lobby(state) => state.peer_network_event(message),
                _ => {}
            },
        }
    }
    pub fn mousedown(&mut self, offset: (f32, f32)) {
    }
}