use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod util;

mod networks;

mod ui;

mod game;
use game::{Game, events::EventQueue};

mod event_loop;
use event_loop::{EventLoop, PlatformEvent};

mod graphics;
use graphics::Renderer;

#[wasm_bindgen]
pub async fn run(canvas: HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger!");

    let event_queue = EventQueue::new();
    let renderer = Renderer::new(canvas.clone()).await;
    let event_sender = event_queue.create_sender();
    let mut game = Game::new(renderer, event_sender);

    //Run application
    EventLoop::run(canvas, move |platform_event| {
        match platform_event {
            PlatformEvent::AnimationFrame { timestamp } => game.update(timestamp),
            PlatformEvent::MouseDown { offset, .. } => game.mousedown(offset),
            _ => {}
        }
        while let Some(user_event) = event_queue.deque() {
            game.handle_event(user_event);
        }
        true
    });
}
