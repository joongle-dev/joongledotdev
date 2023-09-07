use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod util;

mod networks;

mod ui;

mod graphics;
use graphics::Renderer;

mod game;
use game::Game;

mod event_loop;
use event_loop::{Event, EventLoop};
use crate::event_loop::PlatformEvent;

#[wasm_bindgen]
pub async fn run(canvas: HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger!");

    let event_loop = EventLoop::<game::events::GameEvent>::new();

    let event_queue = event_loop.get_event_queue();
    let renderer = Renderer::new(canvas.clone()).await;
    let mut game = Game::new(renderer, event_queue);

    event_loop.run(canvas, move |event| {
        match event {
            Event::PlatformEvent(event) => match event {
                PlatformEvent::AnimationFrame { timestamp } => game.update(timestamp),
                PlatformEvent::MouseMove { .. } => {}
                PlatformEvent::MouseDown { .. } => {}
                PlatformEvent::MouseUp { .. } => {}
            },
            Event::UserEvent(event) => game.handle_event(event),
        }
        true
    });
}