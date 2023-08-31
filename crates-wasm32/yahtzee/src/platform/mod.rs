use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent};
use std::{rc::Rc, cell::RefCell};

pub enum MouseButton {
    Unknown, Left, Middle, Right,
}
pub enum MouseAction {
    Move, Down(MouseButton), Up(MouseButton),
}
pub enum Event {
    AnimationFrame(f64),
    MouseEvent((f32, f32), MouseAction),
}
struct Platform<F: FnMut(Event)-> bool + 'static> {
    event_handler: F,
    animation_frame_id: i32,
    animation_frame_closure: Option<Closure<dyn FnMut(JsValue)>>,
    mouse_move_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
}
pub fn run_event_loop<F: FnMut(Event) -> bool + 'static>(canvas: HtmlCanvasElement, event_handler: F) {
    let platform = Rc::new(RefCell::new(Platform {
        event_handler,
        animation_frame_id: 0,
        animation_frame_closure: None,
        mouse_move_closure: None,
        mouse_down_closure: None,
        mouse_up_closure: None,
    }));
    let platform_ref = platform.borrow_mut();

    //Animation frame callback setup
    let window = web_sys::window().unwrap_throw();
    let window_clone = web_sys::window().unwrap_throw();
    let platform_clone = platform.clone();
    let animation_frame_closure = Closure::new(move |time: JsValue| {
        let time = time.as_f64().unwrap_throw();
        let platform_clone_ref = platform_clone.borrow_mut();
        if (platform_clone_ref.event_handler)(Event::AnimationFrame(time)) == false {
            window_clone.cancel_animation_frame(platform_clone_ref.animation_frame_id).unwrap_throw();
        } else if let Some(animation_frame_closure) = platform_clone_ref.animation_frame_closure.as_ref() {
            platform_clone_ref.animation_frame_id = window_clone.request_animation_frame(animation_frame_closure.as_ref().unchecked_ref()).unwrap_throw();
        }
    });
    platform_ref.animation_frame_id = window.request_animation_frame(animation_frame_closure.as_ref().unchecked_ref()).unwrap_throw();
    platform_ref.animation_frame_closure = Some(animation_frame_closure);

    //Mouse move callback setup
    let platform_clone = platform.clone();
    let mouse_move_closure = Closure::new(move |event: MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let event = Event::MouseEvent((x, y), MouseAction::Move);
        let platform_ref = platform_clone.borrow_mut();
        (platform_ref.event_handler)(event);
    });
    canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
    platform_ref.mouse_move_closure = Some(mouse_move_closure);

    //Mouse down callback setup
    let platform_clone = platform.clone();
    let mouse_down_closure = Closure::new(move |event: MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let button = match event.button() { 1 => MouseButton::Left, 2 => MouseButton::Right, 4 => MouseButton::Middle, _ => MouseButton::Unknown };
        let event = Event::MouseEvent((x, y), MouseAction::Down(button));
        (platform_clone.borrow_mut().event_handler)(event);
    });
    canvas.set_onmousedown(Some(mouse_down_closure.as_ref().unchecked_ref()));
    platform_ref.mouse_down_closure = Some(mouse_down_closure);

    //Mouse up callback setup
    let platform_clone = platform.clone();
    let mouse_up_closure = Closure::new(move |event: MouseEvent| {
        let x = event.offset_x() as f32;
        let y = event.offset_y() as f32;
        let button = match event.button() { 1 => MouseButton::Left, 2 => MouseButton::Right, 4 => MouseButton::Middle, _ => MouseButton::Unknown };
        let event = Event::MouseEvent((x, y), MouseAction::Up(button));
        (platform_clone.borrow_mut().event_handler)(event);
    });
    canvas.set_onmouseup(Some(mouse_up_closure.as_ref().unchecked_ref()));
    platform_ref.mouse_up_closure = Some(mouse_up_closure);
}