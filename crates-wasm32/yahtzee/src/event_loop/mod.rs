use wasm_bindgen::prelude::*;
use web_sys::{Window, HtmlCanvasElement, MouseEvent};
use std::{rc::Rc, cell::{Cell, RefCell, OnceCell}};
use crate::util::ring_buffer::FixedRingBuffer;

pub enum MouseButton {
    Unknown, Left, Middle, Right,
}

pub enum PlatformEvent {
    AnimationFrame { timestamp: f64 },
    MouseMove { timestamp: f64, offset: (f32, f32) },
    MouseDown { timestamp: f64, offset: (f32, f32), button: MouseButton },
    MouseUp { timestamp: f64, offset: (f32, f32), button: MouseButton },
}

pub enum Event<T: 'static> {
    PlatformEvent(PlatformEvent),
    UserEvent(T),
}

pub struct EventHandler<T: 'static> {
    window: Window,
    event_queue: RefCell<FixedRingBuffer<Event<T>, 32>>,
    event_handler: RefCell<Box<dyn FnMut(Event<T>)->bool>>,
    animation_frame_id: Cell<i32>,
    animation_frame_closure: OnceCell<Closure<dyn FnMut(JsValue)>>,
    mouse_move_closure: OnceCell<Closure<dyn FnMut(MouseEvent)>>,
    mouse_down_closure: OnceCell<Closure<dyn FnMut(MouseEvent)>>,
    mouse_up_closure: OnceCell<Closure<dyn FnMut(MouseEvent)>>,
}
impl<T: 'static> EventHandler<T> {
    fn queue_and_call(&self, event: Event<T>) {
        if self.event_queue.borrow_mut().push_back(event).is_err() {
            //TODO: Handle full queue.
        }
        if let Ok(mut handler_ref) = self.event_handler.try_borrow_mut() {
            while let Some(event) = { let ev = self.event_queue.borrow_mut().pop_front(); ev } {
                if !handler_ref(event) {
                    self.window.cancel_animation_frame(self.animation_frame_id.get()).unwrap();
                }
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

pub struct EventSender<T: 'static>(Rc<EventHandler<T>>);
impl<T: 'static> EventSender<T> {
    pub fn send(&self, event: T) {
        self.0.queue_and_call(Event::UserEvent(event));
    }
}
impl<T: 'static> Clone for EventSender<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }

    fn clone_from(&mut self, source: &Self) {
        self.0 = source.0.clone();
    }
}

pub struct EventLoop<T: 'static>(Rc<EventHandler<T>>);
impl<T: 'static> EventLoop<T> {
    pub fn new() -> Self {
        Self(Rc::new(EventHandler {
            window: web_sys::window().unwrap(),
            event_queue: RefCell::new(FixedRingBuffer::new()),
            event_handler: RefCell::new(Box::new(|_| {
                log::warn!("Event handler is not initialized");
                false
            })),
            animation_frame_id: Cell::new(0),
            animation_frame_closure: OnceCell::new(),
            mouse_move_closure: OnceCell::new(),
            mouse_down_closure: OnceCell::new(),
            mouse_up_closure: OnceCell::new(),
        }))
    }
    pub fn get_event_queue(&self) -> EventSender<T> {
        EventSender(self.0.clone())
    }
    pub fn run(self, canvas: HtmlCanvasElement, event_handler: impl FnMut(Event<T>)->bool + 'static) {
        let _ = self.0.event_handler.replace(Box::new(event_handler));
        let event_handler = self.0;

        //Animation frame callback setup
        let animation_frame_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |time: JsValue| {
                let timestamp = time.as_f64().unwrap_throw();
                event_handler.request_animation_frame();
                event_handler.queue_and_call(Event::PlatformEvent(
                    PlatformEvent::AnimationFrame { timestamp }
                ));
            })
        };
        event_handler.animation_frame_closure.set(animation_frame_closure).unwrap();

        //Mouse move callback setup
        let mouse_move_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                event_handler.queue_and_call(Event::PlatformEvent(
                    PlatformEvent::MouseMove { timestamp, offset }
                ));
            })
        };
        canvas.set_onmousemove(Some(mouse_move_closure.as_ref().unchecked_ref()));
        event_handler.mouse_move_closure.set(mouse_move_closure).unwrap();

        //Mouse down callback setup
        let mouse_down_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                let button = match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                };
                event_handler.queue_and_call(Event::PlatformEvent(
                    PlatformEvent::MouseDown { timestamp, offset, button }
                ));
            })
        };
        canvas.set_onmousedown(Some(mouse_down_closure.as_ref().unchecked_ref()));
        event_handler.mouse_down_closure.set(mouse_down_closure).unwrap();

        //Mouse up callback setup
        let mouse_up_closure = {
            let event_handler = event_handler.clone();
            Closure::new(move |event: MouseEvent| {
                let timestamp = event.time_stamp();
                let offset = (event.offset_x() as f32, event.offset_y() as f32);
                let button = match event.button() {
                    1 => MouseButton::Left,
                    2 => MouseButton::Right,
                    4 => MouseButton::Middle,
                    _ => MouseButton::Unknown
                };
                event_handler.queue_and_call(Event::PlatformEvent(
                    PlatformEvent::MouseUp { timestamp, offset, button }
                ));
            })
        };
        canvas.set_onmouseup(Some(mouse_up_closure.as_ref().unchecked_ref()));
        event_handler.mouse_up_closure.set(mouse_up_closure).unwrap();

        event_handler.request_animation_frame();
    }
}