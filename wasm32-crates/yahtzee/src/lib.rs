use wasm_bindgen::prelude::*;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    platform::web::WindowBuilderExtWebSys,
};

mod graphics;

#[wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info)
        .expect("Failed to initialize logger!");

    let event_loop = EventLoop::new();
    let window = match WindowBuilder::new().with_canvas(Some(canvas)).build(&event_loop) {
        Ok(window) => window,
        Err(error) => panic!("Failed to create winit window from canvas: {error}")
    };
    let renderer = graphics::Renderer::new(window).await;

    event_loop.run(move |event, _, control_flow| { 
        match event {
            Event::WindowEvent { window_id, ref event } if window_id == window.id() => {
                match event {
                    WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                    WindowEvent::Resized(new_size) => renderer.resize(new_size),
                    _ => {}
                }
            } 
            Event::RedrawRequested(window_id) if window_id == window.id() => {
                
            }
            Event::MainEventsCleared => {
                window.request_redraw();
            }
            _ => {}
        }
    });
}