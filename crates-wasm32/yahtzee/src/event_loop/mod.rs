use std::{rc::Rc, cell::{Cell, RefCell, OnceCell}};

use wasm_bindgen::prelude::*;
use crate::util::ring_buffer::FixedRingBuffer;

pub enum Event<UserEvent: 'static> {
    FrameUpdate { time: f64 },
    MouseEvent { time: f64, position: (f32, f32), action: MouseAction },
    ResizeEvent { size: (u32, u32) },
    UserEvent(UserEvent),
}

pub enum MouseButton {
    Unknown, Left, Middle, Right,
}

pub enum MouseAction {
    Move,
    ButtonDown(MouseButton),
    ButtonUp(MouseButton),
}

pub struct EventDispatcher<UserEvent: 'static> {
    event_queue: RefCell<FixedRingBuffer<Event<UserEvent>, 32>>,
    event_handler: RefCell<Box<dyn FnMut(Event<UserEvent>)>>,

    window: web_sys::Window,

    animation_frame_id: Cell<i32>,
    animation_frame_closure: OnceCell<Closure<dyn FnMut(JsValue)>>,

    mouse_move_closure: OnceCell<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    mouse_down_closure: OnceCell<Closure<dyn FnMut(web_sys::MouseEvent)>>,
    mouse_up_closure: OnceCell<Closure<dyn FnMut(web_sys::MouseEvent)>>,

    canvas_size: (u32, u32),
    canvas_resize_observer: web_sys::ResizeObserver,
}
impl<T: 'static> EventDispatcher<T> {
    fn dispatch(&self, event: Event<T>) {
        if self.event_queue.borrow_mut().try_push_back(event).is_err() {
            //TODO: Handle full queue.
        }
        if let Ok(mut handler_ref) = self.event_handler.try_borrow_mut() {
            while let Some(event) = self.event_queue.borrow_mut().pop_front() {
                handler_ref(event);
            }
        }
    }
    fn request_animation_frame(&self) {
        if let Some(closure_ref) = self.animation_frame_closure.get() {
            self.animation_frame_id.set(self.window.request_animation_frame(
                closure_ref.as_ref().unchecked_ref()).unwrap_throw()
            );
        }
    }
}

pub struct EventDispatcherProxy<T: 'static>(Rc<EventDispatcher<T>>);
impl<T: 'static> EventDispatcherProxy<T> {
    pub fn send(&self, event: T) {
        self.0.dispatch(Event::UserEvent(event));
    }
}
impl<T: 'static> Clone for EventDispatcherProxy<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0.clone();
    }
}

pub struct EventLoop<T: 'static>(Rc<EventDispatcher<T>>);
impl<T: 'static> EventLoop<T> {
    pub fn new() -> Self {
        Self(Rc::new(EventDispatcher {
            window: web_sys::window().unwrap(),
            event_queue: RefCell::new(FixedRingBuffer::new()),
            event_handler: RefCell::new(Box::new(|_| {

            })),
            animation_frame_id: Cell::new(0),
            animation_frame_closure: OnceCell::new(),
            mouse_move_closure: OnceCell::new(),
            mouse_down_closure: OnceCell::new(),
            mouse_up_closure: OnceCell::new(),

            canvas_size: (0, 0),
            canvas_resize_observer: JsValue::null().into(),
        }))
    }
    pub fn get_event_queue(&self) -> EventDispatcherProxy<T> {
        EventDispatcherProxy(self.0.clone())
    }
    pub fn run<F>(self, canvas: web_sys::HtmlCanvasElement, event_handler: F) where F: 'static + FnMut(Event<T>) {
        let _ = self.0.event_handler.replace(Box::new(event_handler));
        let event_handler = self.0;

        // Animation frame callback setup
        let animation_frame_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |time: JsValue| {
                let time = time.as_f64().unwrap_throw();
                event_handler.dispatch(Event::FrameUpdate{ time });
                event_handler.request_animation_frame();
            })
        };
        event_handler.animation_frame_closure.set(animation_frame_closure).unwrap();

        // Mouse move callback setup
        let mouse_move_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: web_sys::MouseEvent| {
                let time = event.time_stamp();
                let position = (event.offset_x() as f32, event.offset_y() as f32);
                let action = MouseAction::Move;
                event_handler.dispatch(Event::MouseEvent { time, position, action });
            })
        };
        canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
        event_handler.mouse_move_closure.set(mouse_move_closure).unwrap();

        // Mouse down callback setup
        let mouse_down_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: web_sys::MouseEvent| {
                let time = event.time_stamp();
                let position = (event.offset_x() as f32, event.offset_y() as f32);
                let action = MouseAction::ButtonDown(match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                });
                event_handler.dispatch(Event::MouseEvent { time, position, action });
            })
        };
        canvas.set_onmousedown(Some(mouse_down_closure.as_ref().unchecked_ref()));
        event_handler.mouse_down_closure.set(mouse_down_closure).unwrap();

        // Mouse up callback setup
        let mouse_up_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: web_sys::MouseEvent| {
                let time = event.time_stamp();
                let position = (event.offset_x() as f32, event.offset_y() as f32);
                let action = MouseAction::ButtonUp(match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                });
                event_handler.dispatch(Event::MouseEvent { time, position, action });
            })
        };
        canvas.set_onmouseup(Some(mouse_up_closure.as_ref().unchecked_ref()));
        event_handler.mouse_up_closure.set(mouse_up_closure).unwrap();

        event_handler.request_animation_frame();
    }
}

pub trait EventHandler<UserEvent = ()> {
    fn on_frame_update(&mut self, _time: f64) {}
    fn on_mouse_event(&mut self, _time: f64, _position: (f32, f32), _action: MouseAction) {}
    fn on_resize(&mut self, _size: (u32, u32)) {}
    fn on_user_event(&mut self, _event: UserEvent) {}
}