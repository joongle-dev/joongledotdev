use crate::events::Event;
use crate::platform::EventHandlerProxy;
use crate::graphics::Renderer;

mod submit_name;
use submit_name::SubmitName;

mod lobby;
use lobby::Lobby;

enum GameState {
    SubmitName(SubmitName),
    Lobby(Lobby)
}
pub struct Game {
    state: GameState,
    renderer: Renderer,
    event_handler: EventHandlerProxy<Event>,
}
impl Game {
    pub fn new(renderer: Renderer, event_handler: EventHandlerProxy<Event>) -> Self {
        Self {
            state: GameState::SubmitName(SubmitName::new(event_handler.clone())),
            renderer,
            event_handler,
        }
    }
    pub fn update(&mut self, timestamp: f64) {
        if let Err(err) = self.renderer.render() {
            panic!("Surface error: {:?}", err);
        }
        match &mut self.state {
            GameState::SubmitName(_) => {}
            GameState::Lobby(state) => state.update(timestamp),
        }
    }
    pub fn handle_event(&mut self, event: Event) {
        match event {
            Event::JoinLobby(name) => self.state = GameState::Lobby(Lobby::new(self.event_handler.clone(), name)),
            Event::WebSocketEvent(message) => match &mut self.state {
                GameState::SubmitName(_) => {}
                GameState::Lobby(state) => state.web_socket_event(message),
            }
            Event::PeerNetworkEvent(message) => match &mut self.state {
                GameState::SubmitName(_) => {}
                GameState::Lobby(state) => state.peer_network_event(message),
            }
        }
    }
    pub fn mousedown(&mut self, offset: (f32, f32)) {
        match &mut self.state {
            GameState::SubmitName(_) => {}
            GameState::Lobby(state) => state.mousedown(offset),
        }
    }
}