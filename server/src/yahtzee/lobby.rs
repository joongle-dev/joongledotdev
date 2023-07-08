use crate::error::{Error, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast::{channel, Receiver, Sender};

#[derive(Clone, Serialize, Deserialize)]
pub enum LobbyEvent {
    Join {
        id: u64,
    },
    SdpOffer {
        source: u64,
        target: u64,
        sdp: Arc<str>,
    },
    SdpAnswer {
        source: u64,
        target: u64,
        sdp: Arc<str>,
    },
    Leave {
        id: u64,
    },
    Begin,
}
pub struct User {
    pub id: u64,
    pub tx: Sender<LobbyEvent>,
    pub rx: Receiver<LobbyEvent>,
}
struct LobbyData {
    broadcast: Sender<LobbyEvent>,
    user_id_counter: u64,
}
#[derive(Clone)]
pub struct Lobby(Arc<Mutex<LobbyData>>);
impl Lobby {
    fn new() -> Self {
        let (sender, _) = channel(32);
        Self(Arc::new(Mutex::new(LobbyData {
            broadcast: sender,
            user_id_counter: 0,
        })))
    }
    pub fn join(&self) -> User {
        //TODO: Error handling
        let mut lobby = self.0.lock().unwrap();
        lobby.user_id_counter += 1;
        User {
            id: lobby.user_id_counter,
            tx: lobby.broadcast.clone(),
            rx: lobby.broadcast.subscribe(),
        }
    }
}
#[derive(Clone)]
pub struct LobbyCollection {
    lobbies: Arc<DashMap<u64, Lobby>>,
}
impl LobbyCollection {
    pub fn new() -> Self {
        Self {
            lobbies: Arc::new(DashMap::new()),
        }
    }
    pub fn create_lobby(&self, lobby_id: u64) -> Result<User> {
        let lobby = Lobby::new();
        if self.lobbies.insert(lobby_id, lobby.clone()).is_some() {
            return Err(Error::YahtzeeLobbyAlreadyExists);
        }
        Ok(lobby.join())
    }
    pub fn join_lobby(&self, lobby_id: u64) -> Result<User> {
        self.lobbies
            .get(&lobby_id)
            .map(|lobby| lobby.join())
            .ok_or(Error::YahtzeeLobbyNotFound)
    }
}
