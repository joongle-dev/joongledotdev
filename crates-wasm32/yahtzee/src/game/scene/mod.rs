use super::events::GameEvent;

pub mod main;
pub mod connecting;
pub mod lobby;


pub trait GameScene {
    fn update(&mut self, time: f64);
    fn handle_event(&mut self, event: GameEvent);
}