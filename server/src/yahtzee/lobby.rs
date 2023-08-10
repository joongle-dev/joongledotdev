use dashmap::{DashMap, mapref::entry::Entry};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, collections::BTreeMap};
use futures::{sink::SinkExt, stream::{StreamExt, SplitSink}};
use rand::Rng;
use tokio::sync::mpsc::UnboundedSender;
use axum::extract::ws::{Message, WebSocket};

pub type LobbyID = u64;
type UserID = u8;

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess{
        user_id: UserID,
        existing_users: Vec<UserID>,
    },
    SdpOffer{
        source: UserID,
        target: UserID,
        sdp: Arc<str>,
    },
    SdpAnswer{
        source: UserID,
        target: UserID,
        sdp: Arc<str>,
    }
}

enum LobbyMessage {
    Connect{
        websocket: WebSocket,
    },
    Disconnect{
        user_id: UserID,
    },
    Message{
        target: UserID,
        socket_message_serialized: Vec<u8>,
    }
}

struct Lobby {
    channel: UnboundedSender<LobbyMessage>,
}

#[derive(Clone)]
pub struct LobbyCollection {
    lobbies: Arc<DashMap<LobbyID, Lobby>>,
}
impl LobbyCollection {
    pub fn new() -> Self {
        Self {
            lobbies: Arc::new(DashMap::new()),
        }
    }
    pub fn create(&self) -> LobbyID {
        //Loop until randomly generated lobby ID does not collide with existing lobbies (unlikely to loop more than once).
        loop {
            let lobby_id = rand::thread_rng().gen::<LobbyID>();
            let lobbies = self.lobbies.clone();
            if let Entry::Vacant(v) = self.lobbies.entry(lobby_id) {
                //Spawn a task that handles the lobby
                let (lobby_sender, mut lobby_receiver) = tokio::sync::mpsc::unbounded_channel::<LobbyMessage>();
                let lobby_sender_clone = lobby_sender.clone();
                let _ = tokio::spawn(async move {
                    let mut user_id_counter = 0;
                    let mut users = BTreeMap::<UserID, SplitSink<WebSocket, Message>>::new();
                    //Read incoming messages for this lobby.
                    while let Some(lobby_message) = lobby_receiver.recv().await {
                        match lobby_message {
                            //On client joining this lobby:
                            LobbyMessage::Connect { websocket: socket } => {
                                let (mut socket_sender, mut socket_receiver) = socket.split();
                            
                                //Generate user ID.
                                let user_id = user_id_counter;
                                user_id_counter += 1;

                                //Send message to client notifying connection to this lobby.
                                let existing_users = users.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>();
                                let socket_message = SocketMessage::ConnectSuccess { user_id, existing_users };
                                let socket_message_serialized = match bincode::serialize(&socket_message) {
                                    Ok(socket_message_serialized) => socket_message_serialized,
                                    Err(_) => break, //Break out of lobby message loop on serialization failure.
                                };
                                let _ = socket_sender.send(Message::Binary(socket_message_serialized)).await;

                                //Add client to users list.
                                users.insert(user_id, socket_sender);

                                //Spawn a task that receives websocket messages from the client and relay them to the lobby task.
                                let lobby_sender_clone = lobby_sender_clone.clone();
                                tokio::spawn(async move {
                                    //Read incoming messages from the client. Breaks if the connection closes.
                                    while let Some(Ok(Message::Binary(socket_message_serialized))) = socket_receiver.next().await {
                                        let socket_message = match bincode::deserialize::<SocketMessage>(&socket_message_serialized) {
                                            Ok(socket_message) => socket_message,
                                            Err(_) => break, //Break out of socket message loop on deserialization failure.
                                        };
                                        match socket_message {
                                            SocketMessage::SdpOffer { target, .. } | SocketMessage::SdpAnswer { target, .. }=> {
                                                let _ = lobby_sender_clone.send(LobbyMessage::Message { target, socket_message_serialized });
                                            },
                                            _ => {},
                                        }
                                    }
                                    //Remove this user from lobby.
                                    let _ = lobby_sender_clone.send(LobbyMessage::Disconnect { user_id });
                                });
                            },
                            //On client disconnecting from this lobby:
                            LobbyMessage::Disconnect { user_id } => {
                                users.remove(&user_id);
                                if users.is_empty() {
                                    break; //Break out of lobby message loop when no users are connected to this lobby.
                                }
                            },
                            //Relay websocket message to target user:
                            LobbyMessage::Message { target, socket_message_serialized } => {
                                if let Some(user) = users.get_mut(&target) {
                                    let _ = user.send(Message::Binary(socket_message_serialized)).await;
                                }
                            },
                        }
                    }
                    //Remove this lobby from registry.
                    let _ = lobbies.remove(&lobby_id);
                });

                v.insert(Lobby { channel: lobby_sender });
            }
        }
    }
    pub fn join(&self, lobby_id: LobbyID, websocket: WebSocket) {
        if let Some(lobby) = self.lobbies.get(&lobby_id) {
            let _ = lobby.channel.send(LobbyMessage::Connect { websocket });
        }
    }
}