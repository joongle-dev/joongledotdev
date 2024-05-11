extern crate alloc;
use alloc::boxed::Box;

mod events;
pub use events::GameEvent;

mod scene;
use scene::{GameScene, main::Main};
use crate::render::Renderer;
use crate::event_loop::EventDispatcherProxy;

pub struct Game {
    scene: Box<dyn GameScene>,
    renderer: Renderer,
}

impl Game {
    pub fn new(renderer: Renderer, event_sender: EventDispatcherProxy<GameEvent>) -> Self {
        Self {
            scene: Box::new(Main::new(event_sender)),
            renderer,
        }
    }
    pub fn update(&mut self, time: f64) {
        self.scene.update(time);
        if let Err(err) = self.renderer.render() {
            panic!("Surface error: {:?}", err);
        }
    }
    pub fn handle_event(&mut self, event: GameEvent) {
        match event {
            GameEvent::ChangeGameScene(scene) => self.scene = scene,
            _ => self.scene.handle_event(event)
        }
    }
}