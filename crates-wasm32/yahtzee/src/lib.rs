use std::fmt::format;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlButtonElement, HtmlInputElement};

mod graphics;
use graphics::Renderer;

mod platform;
use platform::{Platform, Event as PlatformEvent};

mod lobby_state;
mod networks;

#[wasm_bindgen]
pub async fn run(canvas: web_sys::HtmlCanvasElement) {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info)
        .expect("Failed to initialize logger!");

    let window = web_sys::window()
        .expect("Failed to retrieve Window.");
    let document = window
        .document()
        .expect("Failed to retrieve Document.");
    let name_input = document
        .get_element_by_id("name-input")
        .expect("Failed to retrieve name-input.")
        .dyn_into::<HtmlInputElement>()
        .expect("name-input is not a input.");
    let name_submit_btn = document
        .get_element_by_id("name-submit-btn")
        .expect("Failed to retrieve name-submit-btn")
        .dyn_into::<HtmlButtonElement>()
        .expect("name-submit-btn is not a button.");
    let onclick_callback: Closure<dyn FnMut(web_sys::MouseEvent)> = {
        Closure::wrap(Box::new(move |_event: web_sys::MouseEvent| {
            let name = name_input.value();
            let location = window.location();
            let protocol = location.protocol()
                .map(|protocol| if protocol.contains("https") { "wss" } else { "ws" })
                .expect("Failed to retrieve protocol");
            let host = location.host()
                .expect("Failed to retrieve host.");
            let path = location.pathname()
                .expect("Failed to retrieve pathname.");
            let search = location.search()
                .expect("Failed to retrieve search.");
            let socket_address = format!("{protocol}://{host}{path}ws{search}");
            let lobby_state = match lobby_state::LobbyState::new(name.as_str(), socket_address.as_str()) {
                Ok(lobby_state) => lobby_state,
                Err(err) => panic!("Failed to connect to server: {err:?}"),
            };
        }))
    };
    name_submit_btn.set_onclick(Some(onclick_callback.as_ref().unchecked_ref()));
    onclick_callback.forget();

    let mut renderer = Renderer::new(canvas.clone()).await;
    let platform = {
        Platform::new(canvas, move |event: PlatformEvent| match event {
            PlatformEvent::AnimationFrame => {
                if let Err(err) = renderer.render() {
                    panic!("Surface error: {err}");
                }
            }
            PlatformEvent::MouseEvent { .. } => {}
        })
    };
    platform.borrow_mut().run();
}