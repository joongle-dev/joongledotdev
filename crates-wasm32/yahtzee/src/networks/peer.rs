use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcPeerConnection, RtcSessionDescriptionInit, RtcSdpType, RtcDataChannel};

pub struct Peer {
    connection: RtcPeerConnection,
}
impl Peer {
    fn new() -> Self {
        let connection = RtcPeerConnection::new().unwrap();
        Self {
            connection,
        }
    }
    async fn create_sdp_answer(&self, offer_sdp: String) -> String {
        let mut offer_description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            offer_description.sdp(&offer_sdp);
        JsFuture::from(self.connection.set_remote_description(&offer_description))
            .await
            .unwrap();
        let answer = JsFuture::from(self.connection.create_answer())
            .await
            .unwrap();
        let answer_sdp = js_sys::Reflect::get(&answer, &JsValue::from_str("sdp"))
            .unwrap()
            .as_string()
            .unwrap();
        let answer_description = RtcSessionDescriptionInit::from(answer);
        JsFuture::from(self.connection.set_local_description(&answer_description))
            .await
            .unwrap();
        answer_sdp
    }
    async fn receive_sdp_answer(&self, answer_sdp: String) {
        let mut answer_description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
            answer_description.sdp(&answer_sdp);
        JsFuture::from(self.connection.set_remote_description(&answer_description))
            .await
            .unwrap();
    }
}