use wasm_bindgen::prelude::*;
use web_sys::{BinaryType, MessageEvent};
use js_sys::{ArrayBuffer, Uint8Array};

pub enum WebSocketMessage {
    String(String),
    Binary(Vec<u8>),
}
pub struct WebSocket {
    websocket: web_sys::WebSocket,
    onmessage_callback: Closure<dyn FnMut(MessageEvent)>,
}
impl WebSocket {
    pub fn new<F: FnMut(WebSocketMessage) + 'static>(url: &str, mut message_callback: F) -> Self {
        let onmessage_callback = Closure::<dyn FnMut(MessageEvent)>::new(move |event: MessageEvent| {
            if let Some(string) = event.data().as_string() {
                message_callback(WebSocketMessage::String(string));
            }
            else if let Ok(arr_buf) = event.data().dyn_into::<ArrayBuffer>() {
                message_callback(WebSocketMessage::Binary(Uint8Array::new(&arr_buf).to_vec()));
            }
            else {
                log::warn!("Unhandled WebSocket message type.");
            }
        });
        let websocket = web_sys::WebSocket::new(url).unwrap_throw();
            websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
            websocket.set_binary_type(BinaryType::Arraybuffer);
        Self {
            websocket,
            onmessage_callback,
        }
    }
    pub fn send_with_str(&self, data: &str) {
        self.websocket.send_with_str(data).unwrap_throw();
    }
    pub fn send_with_u8_array(&self, data: &[u8]) {
        self.websocket.send_with_u8_array(data).unwrap_throw();
    }
}