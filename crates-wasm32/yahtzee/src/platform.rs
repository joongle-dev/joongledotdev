use std::borrow::BorrowMut;
use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

pub enum MouseButton {
    Unknown, Left, Middle, Right,
}
pub enum MouseAction {
    Move, Down(MouseButton), Up(MouseButton),
}
pub enum Event {
    AnimationFrame,
    MouseEvent {
        x: f32,
        y: f32,
        action: MouseAction,
    },
}

pub struct Platform<F: FnMut(Event) + 'static> {
    window: web_sys::Window,
    animation_frame_closure: Option<Closure<dyn FnMut()>>,
    animation_id: Option<i32>,
    event_handler: F,
    closures: Vec<Box<dyn Drop>>,
}
impl<F: FnMut(Event) + 'static> Platform<F> {
    pub fn new(canvas: web_sys::HtmlCanvasElement, event_handler: F) -> Rc<RefCell<Self>> {
        let mut platform = Rc::new(RefCell::new(Self {
            window: web_sys::window().expect("Failed to retrieve Window."),
            animation_frame_closure: None,
            animation_id: None,
            event_handler: event_handler,
            closures: vec![],
        }));
        {
            platform.borrow_mut().as_ref().borrow_mut().animation_frame_closure = Some({
                let mut platform = platform.clone();
                Closure::wrap(Box::new(move || {
                    (platform.borrow_mut().as_ref().borrow_mut().event_handler)(Event::AnimationFrame);
                    platform.borrow_mut().as_ref().borrow_mut().request_redraw();
                }))
            });
        }
        {
            let closure: Closure<dyn FnMut(web_sys::MouseEvent)> = {
                let mut platform = platform.clone();
                Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let event = Event::MouseEvent {
                        x: event.offset_x() as f32,
                        y: event.offset_y() as f32,
                        action: MouseAction::Move,
                    };
                    (platform.borrow_mut().as_ref().borrow_mut().event_handler)(event);
                }))
            };
            canvas.set_onmousemove(Some(closure.as_ref().unchecked_ref()));
            platform.borrow_mut().as_ref().borrow_mut().closures.push(Box::new(closure));
        }
        {
            let closure: Closure<dyn FnMut(web_sys::MouseEvent)> = {
                let mut platform = platform.clone();
                Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let button = match event.button() {
                        1 => MouseButton::Left,
                        2 => MouseButton::Right,
                        4 => MouseButton::Middle,
                        _ => MouseButton::Unknown,
                    };
                    let event = Event::MouseEvent {
                        x: event.offset_x() as f32,
                        y: event.offset_y() as f32,
                        action: MouseAction::Down(button),
                    };
                    (platform.borrow_mut().as_ref().borrow_mut().event_handler)(event);
                }))
            };
            canvas.set_onmousedown(Some(closure.as_ref().unchecked_ref()));
            platform.borrow_mut().as_ref().borrow_mut().closures.push(Box::new(closure));
        }
        {
            let closure: Closure<dyn FnMut(web_sys::MouseEvent)> = {
                let mut platform = platform.clone();
                Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let button = match event.button() {
                        1 => MouseButton::Left,
                        2 => MouseButton::Right,
                        4 => MouseButton::Middle,
                        _ => MouseButton::Unknown,
                    };
                    let event = Event::MouseEvent {
                        x: event.offset_x() as f32,
                        y: event.offset_y() as f32,
                        action: MouseAction::Up(button),
                    };
                    (platform.borrow_mut().as_ref().borrow_mut().event_handler)(event);
                }))
            };
            canvas.set_onmouseup(Some(closure.as_ref().unchecked_ref()));
            platform.borrow_mut().as_ref().borrow_mut().closures.push(Box::new(closure));
        }

        platform
    }
    pub fn run(&mut self) {
        self.request_redraw();
    }
    pub fn stop(&mut self) {
        if let Some(animation_id) = self.animation_id {
            self.window.cancel_animation_frame(animation_id).expect("failed to cancel animation frame");
            self.animation_id = None;
        }
    }
    fn request_redraw(&mut self) {
        if let Some(ref closure) = self.animation_frame_closure {
            let animation_id = self.window.request_animation_frame(closure.as_ref().unchecked_ref()).expect("failed to request animation frame");
            self.animation_id = Some(animation_id);
        }
        else {
            self.animation_id = None;
        }
    }
}