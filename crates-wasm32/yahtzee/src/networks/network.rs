use wasm_bindgen::closure::Closure;
use crate::networks::webrtc::{DataChannel, IceCandidate, PeerConnection};

struct PeerData {
    connection: PeerConnection,
    channel: DataChannel,
    #[allow(clippy::type_complexity)]
    closures: (
        Closure<dyn FnMut()>,
        Closure<dyn FnMut()>,
        Closure<dyn FnMut()>,
    ),
}
enum PeerStatus {
    Disconnected,
    Connecting(PeerData, Vec<IceCandidate>),
    Connected(PeerData),
}
struct PeerNetwork {

}