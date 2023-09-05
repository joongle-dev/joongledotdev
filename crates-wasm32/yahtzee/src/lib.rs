use wasm_bindgen::prelude::*;

mod util;

mod networks;

mod ui;

mod game;
use game::{Context, Game};

mod platform;
use platform::{EventLoop, Event, PlatformEvent};
mod graphics;
use graphics::Renderer;


#[wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Failed to initialize logger!");

    let event_loop = EventLoop::new();

    let context = Context::new(canvas.clone()).await;
    let mut game = game::Game::new(renderer, event_loop.event_handler_proxy());

    //Run application
    event_loop.run(canvas, move |event: Event<game::events::Event>| {
        match event {
            Event::PlatformEvent(event) => {
                match event {
                    PlatformEvent::AnimationFrame { timestamp } => game.update(timestamp),
                    PlatformEvent::MouseDown { offset, .. } => game.mousedown(offset),
                    _ => {},
                }
            }
            Event::UserEvent(event) => game.handle_event(event),
        }
        true
    });
}