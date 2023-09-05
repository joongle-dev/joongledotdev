use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent};
use js_sys::{ArrayBuffer, Uint8Array};
use serde::{Serialize, de::DeserializeOwned};
use std::{rc::Rc, cell::RefCell, marker::PhantomData};

pub enum WebSocketEvent<T> {
    Connect,
    Disconnect,
    Message(T),
}
pub struct WebSocket<T: Serialize + DeserializeOwned + 'static> {
    websocket: web_sys::WebSocket,
    _onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
    _onopen_callback: Closure<dyn FnMut()>,
    _onclose_callback: Closure<dyn FnMut()>,
    _phantom_data: PhantomData<T>,
}
impl<T: Serialize + DeserializeOwned + 'static> WebSocket<T> {
    pub fn new<F: FnMut(WebSocketEvent<T>) + 'static>(url: &str, message_callback: F) -> Self {
        let message_callback = Rc::new(RefCell::new(message_callback));
        let onmessage_callback: Closure<dyn FnMut(MessageEvent)> = {
            let message_callback = message_callback.clone();
            Closure::new(move |event: MessageEvent| {
                if let Ok(data) = event.data().dyn_into::<ArrayBuffer>() {
                    let data = Uint8Array::new(&data).to_vec();
                    let message: T = bincode::deserialize(data.as_slice()).unwrap();
                    message_callback.borrow_mut()(WebSocketEvent::Message(message));
                }
            })
        };
        let onopen_callback: Closure<dyn FnMut()> = {
            let message_callback = message_callback.clone();
            Closure::new(move || {
                message_callback.borrow_mut()(WebSocketEvent::Connect);
            })
        };
        let onclose_callback: Closure<dyn FnMut()> = {
            let message_callback = message_callback;
            Closure::new(move || {
                message_callback.borrow_mut()(WebSocketEvent::Disconnect);
            })
        };

        let websocket = web_sys::WebSocket::new(url).unwrap_throw();
            websocket.set_binary_type(BinaryType::Arraybuffer);
            websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
            websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        Self {
            websocket,
            _onmessage_callback: onmessage_callback,
            _onopen_callback: onopen_callback,
            _onclose_callback: onclose_callback,
            _phantom_data: PhantomData::default(),
        }
    }
    pub fn send(&self, message: T) {
        let serialized = bincode::serialize(&message).unwrap();
        self.websocket.send_with_u8_array(serialized.as_slice()).unwrap();
    }
}