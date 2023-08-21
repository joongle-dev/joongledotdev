use std::sync::Arc;
use serde::{Deserialize, Serialize};

pub type LobbyID = u64;
type UserID = u8;

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess{
        lobby_id: LobbyID,
        user_id: UserID,
        existing_users: Vec<UserID>,
    },
    SdpOffer{
        source: UserID,
        target: UserID,
        name: Arc<str>,
        sdp: Arc<str>,
    },
    SdpAnswer{
        source: UserID,
        target: UserID,
        name: Arc<str>,
        sdp: Arc<str>,
    }
}

pub struct LobbyState {

}
impl LobbyState {
    pub fn new() -> Self {
         Self {}
    }
}