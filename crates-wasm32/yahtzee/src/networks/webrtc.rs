use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Event, RtcPeerConnection, RtcSessionDescriptionInit, RtcSdpType, RtcDataChannel, RtcDataChannelInit, RtcPeerConnectionIceEvent, RtcIceCandidateInit, MessageEvent, RtcIceCandidate, RtcIceServer, RtcConfiguration, RtcPeerConnectionState, RtcDataChannelState, RtcDataChannelEvent};
use serde::{Deserialize, Serialize};
use js_sys::{Object, Array, Reflect};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(extends = Event, extends = Object, js_name = RTCErrorEvent, typescript_type = "RTCErrorEvent")]
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub type RTCErrorEvent;
    #[wasm_bindgen(structural, method, getter, js_class = "RTCErrorEvent", js_name = error)]
    pub fn error(this: &RTCErrorEvent) -> JsValue;
}

#[derive(Clone)]
pub struct Configuration(RtcConfiguration);
pub struct ConfigurationBuilder {
    configuration: RtcConfiguration,
    ice_servers: Array,
}
impl ConfigurationBuilder {
    pub fn new() -> Self {
        let ice_servers = Array::new();
        let mut configuration = RtcConfiguration::new();
            configuration.ice_servers(&ice_servers);
        Self {
            configuration,
            ice_servers,
        }
    }
    pub fn add_stun_server(self, urls: &str) -> Self {
        let mut ice_server = RtcIceServer::new();
            ice_server.urls(&urls.into());
        self.ice_servers.push(&ice_server);
        self
    }
    pub fn add_turn_server(self, urls: &str, username: &str, credential: &str) -> Self {
        let mut ice_server = RtcIceServer::new();
            ice_server.urls(&urls.into());
            ice_server.username(username);
            ice_server.credential(credential);
        self.ice_servers.push(&ice_server);
        self
    }
    pub fn build(self) -> Configuration {
        Configuration(self.configuration)
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct IceCandidate(String, Option<String>, Option<u16>);
impl From<RtcIceCandidate> for IceCandidate {
    fn from(value: RtcIceCandidate) -> Self {
        Self(value.candidate(), value.sdp_mid(), value.sdp_m_line_index())
    }
}
impl From<IceCandidate> for RtcIceCandidate {
    fn from(value: IceCandidate) -> Self {
        let mut candidate_dict = RtcIceCandidateInit::new(value.0.as_str());
            candidate_dict.sdp_mid(value.1.as_deref());
            candidate_dict.sdp_m_line_index(value.2);
        RtcIceCandidate::new(&candidate_dict).unwrap()
    }
}

#[derive(Clone)]
pub struct DataChannel(RtcDataChannel);
impl DataChannel {
    pub fn send_str(&self, data: &str) {
        self.0.send_with_str(data).unwrap();
    }
    pub fn ready_state(&self) -> RtcDataChannelState {
        self.0.ready_state()
    }
    pub fn set_onopen<F: FnMut() + 'static>(&self, f: F) -> Closure<dyn FnMut()> {
        let callback = Closure::new(f);
        self.0.set_onopen(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub fn set_onclose<F: FnMut() + 'static>(&self, f: F) -> Closure<dyn FnMut()> {
        let callback = Closure::new(f);
        self.0.set_onclose(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub fn set_onclosing<F: FnMut() + 'static>(&self, f: F) -> Closure<dyn FnMut()> {
        let callback = Closure::new(f);
        self.0.add_event_listener_with_callback("closing", callback.as_ref().unchecked_ref()).unwrap();
        callback
    }
    pub fn set_onerror<F: FnMut() + 'static>(&self, f: F) -> Closure<dyn FnMut()> {
        let callback = Closure::new(f);
        self.0.set_onerror(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub fn set_onmessage<F: FnMut(MessageEvent) + 'static>(&self, f: F) -> Closure<dyn FnMut(MessageEvent)> {
        let callback = Closure::new(f);
        self.0.set_onmessage(Some(callback.as_ref().unchecked_ref()));
        callback
    }
}

#[derive(Clone)]
pub struct PeerConnection(RtcPeerConnection);
impl PeerConnection {
    pub fn new() -> Self {
        Self(RtcPeerConnection::new().unwrap())
    }
    pub fn new_with_configuration(configuration: Configuration) -> Self {
        Self(RtcPeerConnection::new_with_configuration(&configuration.0).unwrap())
    }
    pub fn connection_state(&self) -> RtcPeerConnectionState {
        self.0.connection_state()
    }
    pub fn set_ondatachannel<F: FnMut(RtcDataChannelEvent) + 'static>(&self, f: F) -> Closure<dyn FnMut(RtcDataChannelEvent)> {
        let callback = Closure::new(f);
        self.0.set_ondatachannel(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub fn set_onconnectionstatechange<F: FnMut() + 'static>(&self, f: F) -> Closure<dyn FnMut()> {
        let callback = Closure::new(f);
        self.0.set_onconnectionstatechange(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub fn set_onicecandidate<F: FnMut(RtcPeerConnectionIceEvent) + 'static>(&self, f: F) -> Closure<dyn FnMut(RtcPeerConnectionIceEvent)> {
        let callback = Closure::new(f);
        self.0.set_onicecandidate(Some(callback.as_ref().unchecked_ref()));
        callback
    }
    pub async fn add_ice_candidate(&self, candidate: IceCandidate) {
        JsFuture::from(self.0.add_ice_candidate_with_opt_rtc_ice_candidate(Some(&candidate.into()))).await.unwrap();
    }
    pub fn create_data_channel_negotiated(&self, label: &str, id: u16) -> DataChannel {
        let mut data_channel_dict = RtcDataChannelInit::new();
            data_channel_dict.negotiated(true);
            data_channel_dict.id(id);
        DataChannel(self.0.create_data_channel_with_data_channel_dict(label, &data_channel_dict))
    }
    pub async fn create_offer_sdp(&self) -> String {
        let offer_obj = JsFuture::from(self.0.create_offer()).await.unwrap();
        let offer_sdp = Reflect::get(&offer_obj, &"sdp".into())
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
        let answer_sdp = Reflect::get(&answer_obj, &"sdp".into())
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