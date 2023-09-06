use wasm_bindgen::prelude::*;
use web_sys::{Window, HtmlCanvasElement, MouseEvent};
use std::{rc::Rc, cell::RefCell};

pub enum MouseButton {
    Unknown, Left, Middle, Right,
}

pub enum PlatformEvent {
    AnimationFrame { timestamp: f64 },
    MouseMove { timestamp: f64, offset: (f32, f32) },
    MouseDown { timestamp: f64, offset: (f32, f32), button: MouseButton },
    MouseUp { timestamp: f64, offset: (f32, f32), button: MouseButton },
}

pub struct EventLoop<F: FnMut(PlatformEvent)->bool + 'static> {
    window: Window,
    event_handler: F,
    animation_frame_id: i32,
    animation_frame_closure: Option<Closure<dyn FnMut(JsValue)>>,
    mouse_move_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
}

impl<F: FnMut(PlatformEvent)->bool + 'static> EventLoop<F> {
    pub fn run(canvas: HtmlCanvasElement, event_handler: F) {
        let event_loop = Rc::new(RefCell::new(Self {
            window: web_sys::window().unwrap_throw(),
            event_handler,
            animation_frame_id: 0,
            animation_frame_closure: None,
            mouse_move_closure: None,
            mouse_down_closure: None,
            mouse_up_closure: None,
        }));
        let mut event_loop_ref = event_loop.borrow_mut();

        //Animation frame callback setup
        let animation_frame_closure = {
            let event_loop = event_loop.clone();
            Closure::new(move |time: JsValue| {
                let timestamp = time.as_f64().unwrap_throw();
                let mut event_loop_ref = event_loop.borrow_mut();
                event_loop_ref.request_animation_frame();
                event_loop_ref.call(PlatformEvent::AnimationFrame { timestamp });
            })
        };
        event_loop_ref.animation_frame_closure = Some(animation_frame_closure);

        //Mouse move callback setup
        let mouse_move_closure = {
            let event_loop = event_loop.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                event_loop.borrow_mut().call(PlatformEvent::MouseMove { timestamp, offset });
            })
        };
        canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
        event_loop_ref.mouse_move_closure = Some(mouse_move_closure);

        //Mouse down callback setup
        let mouse_down_closure = {
            let event_loop = event_loop.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                let button = match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                };
                event_loop.borrow_mut().call(PlatformEvent::MouseDown { timestamp, offset, button });
            })
        };
        canvas.set_onmousedown(Some(mouse_down_closure.as_ref().unchecked_ref()));
        event_loop_ref.mouse_down_closure = Some(mouse_down_closure);

        //Mouse up callback setup
        let mouse_up_closure = {
            let event_loop = event_loop.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                let button = match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                };
                event_loop.borrow_mut().call(PlatformEvent::MouseUp { timestamp, offset, button });
            })
        };
        canvas.set_onmouseup(Some(mouse_up_closure.as_ref().unchecked_ref()));
        event_loop_ref.mouse_up_closure = Some(mouse_up_closure);

        //Begin loop
        event_loop_ref.request_animation_frame();
    }
    fn call(&mut self, event: PlatformEvent) {
        if !(self.event_handler)(event) {
            self.window.cancel_animation_frame(self.animation_frame_id).unwrap_throw();
        }
    }
    fn request_animation_frame(&mut self) {
        if let Some(ref animation_frame_closure) = self.animation_frame_closure {
            self.animation_frame_id = self.window.request_animation_frame(animation_frame_closure.as_ref().unchecked_ref()).unwrap();
        }
    }
}