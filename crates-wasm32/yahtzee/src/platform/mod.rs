use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, MouseEvent, Window};
use std::{rc::Rc, cell::RefCell, marker::PhantomData};

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

struct Platform<E, F: FnMut(Event<E>)->bool + 'static> {
    window: Window,
    event_handler: F,
    animation_frame_id: i32,
    animation_frame_closure: Option<Closure<dyn FnMut(JsValue)>>,
    mouse_move_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_closure: Option<Closure<dyn FnMut(MouseEvent)>>,
    phantom_data: PhantomData<E>,
}
impl<E, F: FnMut(Event<E>)->bool + 'static> Platform<E, F> {
    pub fn call(&mut self, event: Event<E>) {
        if !(self.event_handler)(event) {
            self.window.cancel_animation_frame(self.animation_frame_id).unwrap_throw();
        }
    }
}

#[derive(Clone)]
pub struct EventHandlerProxy<E: 'static, F: FnMut(Event<E>)->bool + 'static>(Rc<RefCell<Platform<E, F>>>);
impl<E: 'static, F: FnMut(Event<E>)->bool + 'static> EventHandlerProxy<E, F> {
    pub fn call(&self, event: E) {
        self.0.borrow_mut().call(Event::UserEvent(event));
    }
}

pub struct EventLoop<E: 'static, F: FnMut(Event<E>)->bool + 'static>(Rc<RefCell<Platform<E, F>>>);
impl<E: 'static, F: FnMut(Event<E>)->bool + 'static> EventLoop<E, F> {
    pub fn new(canvas: HtmlCanvasElement, event_handler: F) -> Self {
        let window = web_sys::window().unwrap_throw();
        let platform = Rc::new(RefCell::new(Platform {
            window,
            event_handler,
            animation_frame_id: 0,
            animation_frame_closure: None,
            mouse_move_closure: None,
            mouse_down_closure: None,
            mouse_up_closure: None,
            phantom_data: PhantomData,
        }));
        {
            let mut platform_ref = platform.borrow_mut();

            //Animation frame callback setup
            let platform_clone = platform.clone();
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
            let platform_clone = platform.clone();
            let mouse_move_closure = Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                let event = PlatformEvent::MouseMove { timestamp, offset };
                platform_clone.borrow_mut().call(Event::PlatformEvent(event));
            });
            canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
            platform_ref.mouse_move_closure = Some(mouse_move_closure);

            //Mouse down callback setup
            let platform_clone = platform.clone();
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
            let platform_clone = platform.clone();
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
        }
        Self(platform)
    }
    pub fn event_handler_proxy(&self) -> EventHandlerProxy<E, F> {
        EventHandlerProxy(self.0.clone())
    }
    pub fn run(&self) {
        let mut platform_ref = self.0.borrow_mut();
        platform_ref.animation_frame_id = platform_ref.window.request_animation_frame(
            platform_ref.animation_frame_closure.as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap_throw();
    }
}