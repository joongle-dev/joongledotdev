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

pub enum Event<T> {
    PlatformEvent(PlatformEvent),
    UserEvent(T),
}

struct EventHandler<E> {
    window: Window,
    event_handler: Box<dyn FnMut(Event<E>)->bool + 'static>,
    animation_frame_id: i32,
    animation_frame_closure: Option<Closure<dyn FnMut(JsValue)>>,
    mouse_move_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
}
impl<E> EventHandler<E> {
    pub fn call(&mut self, event: Event<E>) {
        if !self.event_handler.as_mut()(event) {
            self.window.cancel_animation_frame(self.animation_frame_id).unwrap_throw();
        }
    }
}

pub struct EventHandlerProxy<E: 'static>(Rc<RefCell<EventHandler<E>>>);
impl<E: 'static> EventHandlerProxy<E> {
    pub fn call(&self, event: E) {
        self.0.borrow_mut().call(Event::UserEvent(event));
    }
    pub fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub struct EventLoop<E: 'static>(Rc<RefCell<EventHandler<E>>>);
impl<E: 'static> EventLoop<E> {
    pub fn new() -> Self {
        let window = web_sys::window().unwrap_throw();
        Self(Rc::new(RefCell::new(EventHandler {
            window,
            event_handler: Box::new(move |_| false),
            animation_frame_id: 0,
            animation_frame_closure: None,
            mouse_move_closure: None,
            mouse_down_closure: None,
            mouse_up_closure: None,
        })))
    }
    pub fn run<F: FnMut(Event<E>)->bool + 'static>(&self, canvas: HtmlCanvasElement, event_handler: F) {
        let mut platform_ref = self.0.borrow_mut();

        //Event handler setup
        platform_ref.event_handler = Box::new(event_handler);

        //Animation frame callback setup
        let platform_clone = self.0.clone();
        let animation_frame_closure = Closure::new(move |time: JsValue| {
            let timestamp = time.as_f64().unwrap_throw();
            let mut platform_clone_ref = platform_clone.borrow_mut();
            platform_clone_ref.animation_frame_id = platform_clone_ref.window.request_animation_frame(
                platform_clone_ref.animation_frame_closure.as_ref().unwrap().as_ref().unchecked_ref()
            ).unwrap_throw();
            platform_clone_ref.call(Event::PlatformEvent(PlatformEvent::AnimationFrame { timestamp }));
        });
        platform_ref.animation_frame_closure = Some(animation_frame_closure);

        //Mouse move callback setup
        let platform_clone = self.0.clone();
        let mouse_move_closure = Closure::new(move |event: MouseEvent| {
            let timestamp = event.time_stamp();
            let offset = (event.offset_x() as f32, event.offset_y() as f32);
            let event = PlatformEvent::MouseMove { timestamp, offset };
            platform_clone.borrow_mut().call(Event::PlatformEvent(event));
        });
        canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
        platform_ref.mouse_move_closure = Some(mouse_move_closure);

        //Mouse down callback setup
        let platform_clone = self.0.clone();
        let mouse_down_closure = Closure::new(move |event: MouseEvent| {
            let timestamp = event.time_stamp();
            let offset = (event.offset_x() as f32, event.offset_y() as f32);
            let button = match event.button() {
                1 => MouseButton::Left,
                2 => MouseButton::Right,
                4 => MouseButton::Middle,
                _ => MouseButton::Unknown
            };
            let event = PlatformEvent::MouseDown { timestamp, offset, button };
            platform_clone.borrow_mut().call(Event::PlatformEvent(event));
        });
        canvas.set_onmousedown(Some(mouse_down_closure.as_ref().unchecked_ref()));
        platform_ref.mouse_down_closure = Some(mouse_down_closure);

        //Mouse up callback setup
        let platform_clone = self.0.clone();
        let mouse_up_closure = Closure::new(move |event: MouseEvent| {
            let timestamp = event.time_stamp();
            let offset = (event.offset_x() as f32, event.offset_y() as f32);
            let button = match event.button() {
                1 => MouseButton::Left,
                2 => MouseButton::Right,
                4 => MouseButton::Middle,
                _ => MouseButton::Unknown
            };
            let event = PlatformEvent::MouseUp { timestamp, offset, button };
            platform_clone.borrow_mut().call(Event::PlatformEvent(event));
        });
        canvas.set_onmouseup(Some(mouse_up_closure.as_ref().unchecked_ref()));
        platform_ref.mouse_up_closure = Some(mouse_up_closure);

        platform_ref.animation_frame_id = platform_ref.window.request_animation_frame(
            platform_ref.animation_frame_closure.as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap_throw();
    }
    pub fn event_handler_proxy(&self) -> EventHandlerProxy<E> {
        EventHandlerProxy(self.0.clone())
    }
}