use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{RtcPeerConnection, RtcSessionDescriptionInit, RtcSdpType, RtcDataChannel, RtcDataChannelInit, MessageEvent, RtcPeerConnectionIceEvent, RtcIceCandidateInit};

#[derive(Clone)]
pub struct DataChannel(RtcDataChannel);

impl DataChannel {

}

#[derive(Clone)]
pub struct PeerConnection(RtcPeerConnection);

impl PeerConnection {
    pub fn new() -> Self {
        Self(RtcPeerConnection::new().unwrap())
    }
    pub fn set_onicecandidate<F>(&self, f: F) -> Closure::<dyn FnMut(RtcPeerConnectionIceEvent)>
        where F: FnMut(RtcPeerConnectionIceEvent) + 'static {
        let callback = Closure::new(f);
        self.0.set_onicecandidate(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub async fn add_ice_candidate(&self, candidate: &str) {
        let candidate = RtcIceCandidateInit::new(candidate);
        JsFuture::from(self.0.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(&candidate))).await.unwrap();
    }
    pub fn create_data_channel(&self, label: &str, id: u16) -> DataChannel {
        let mut data_channel_dict = RtcDataChannelInit::new();
            data_channel_dict.negotiated(false);
            data_channel_dict.id(id);
        DataChannel(self.0.create_data_channel_with_data_channel_dict(label, &data_channel_dict))
    }
    pub async fn create_offer_sdp(&self) -> String {
        let offer_obj = JsFuture::from(self.0.create_offer()).await.unwrap();
        let offer_sdp = js_sys::Reflect::get(&offer_obj, &JsValue::from_str("sdp"))
            .unwrap().as_string().unwrap();
        let offer_description = RtcSessionDescriptionInit::from(offer_obj);
        JsFuture::from(self.0.set_local_description(&offer_description)).await.unwrap();
        offer_sdp
    }
    pub async fn receive_offer_sdp(&self, offer_sdp: String) {
        let mut offer_description = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
            offer_description.sdp(&offer_sdp);
        JsFuture::from(self.0.set_remote_description(&offer_description)).await.unwrap();
    }
    pub async fn create_answer_sdp(&self) -> String {
        let answer_obj = JsFuture::from(self.0.create_answer()).await.unwrap();
        let answer_sdp = js_sys::Reflect::get(&answer_obj, &JsValue::from_str("sdp"))
            .unwrap().as_string().unwrap();
        let answer_description = RtcSessionDescriptionInit::from(answer_obj);
        JsFuture::from(self.0.set_local_description(&answer_description)).await.unwrap();
        answer_sdp
    }
    pub async fn receive_answer_sdp(&self, answer_sdp: String) {
        let mut answer_description = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
            answer_description.sdp(&answer_sdp);
        JsFuture::from(self.0.set_remote_description(&answer_description)).await.unwrap();
    }
}