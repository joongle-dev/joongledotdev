use dashmap::{DashMap, mapref::entry::Entry};
use serde::{Deserialize, Serialize};
use std::{sync::Arc, collections::BTreeMap};
use futures::{sink::SinkExt, stream::{StreamExt, SplitSink}};
use tokio::sync::mpsc::UnboundedSender;
use axum::extract::ws::{Message, WebSocket};
use bytes::Bytes;

pub type LobbyID = u64;
type UserID = u32;

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess {
        lobby_id: LobbyID,
        user_id: UserID,
        peers_id: Vec<UserID>,
    },
    WebRtcHandshake {
        source_id: UserID,
        target_id: UserID,
        sdp_description: String,
        ice_candidates: Vec<(String, Option<String>, Option<u16>)>,
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
        socket_message_serialized: Bytes,
    }
}

struct Lobby {
    channel: UnboundedSender<LobbyMessage>,
}

#[derive(Clone)]
pub struct LobbyCollection {
    lobbies: Arc<DashMap<LobbyID, Lobby>>,
}
impl Default for LobbyCollection {
    fn default() -> Self {
        Self {
            lobbies: Arc::new(DashMap::new()),
        }
    }
}
impl LobbyCollection {
    pub fn create(&self) -> LobbyID {
        //Create lobby message channel.
        let (lobby_sender, mut lobby_receiver) = tokio::sync::mpsc::unbounded_channel::<LobbyMessage>();

        //Loop until randomly generated lobby ID does not collide with existing lobbies (unlikely to loop more than once).
        let lobby_id = loop {
            let lobby_id = rand::random::<LobbyID>();
            if let Entry::Vacant(v) = self.lobbies.entry(lobby_id) {
                v.insert(Lobby { channel: lobby_sender.clone() });
                break lobby_id
            }
        };

        //Spawn a task that handles lobby logic.
        let lobbies = self.lobbies.clone();
        tokio::spawn(async move {
            let mut user_id_counter = 0;
            let mut users = BTreeMap::<UserID, SplitSink<WebSocket, Message>>::new();
            //Read incoming messages for this lobby.
            while let Some(lobby_message) = lobby_receiver.recv().await {
                match lobby_message {
                    //On client joining this lobby:
                    LobbyMessage::Connect { websocket } => {
                        let (mut socket_sender, mut socket_receiver) = websocket.split();

                        //Generate user ID.
                        let user_id = user_id_counter;
                        user_id_counter += 1;

                        //Send message to client notifying connection to this lobby.
                        let peers_id = users.keys().cloned().collect::<Vec<_>>();
                        let socket_message = SocketMessage::ConnectSuccess { lobby_id, user_id: user_id, peers_id };
                        let socket_message_serialized = match bincode::serialize(&socket_message) {
                            Ok(socket_message_serialized) => socket_message_serialized,
                            Err(_) => break, //Break out of lobby message loop on serialization failure.
                        };
                        let _ = socket_sender.send(Message::Binary(socket_message_serialized.into())).await;

                        //Add client to users list.
                        users.insert(user_id, socket_sender);

                        //Spawn a task that receives websocket messages from the client and relay them to the lobby task.
                        let lobby_sender = lobby_sender.clone();
                        tokio::spawn(async move {
                            println!("->> User {user_id} joined lobby {lobby_id}");
                            //Read incoming messages from the client. Breaks if the connection closes.
                            while let Some(Ok(Message::Binary(socket_message_serialized))) = socket_receiver.next().await {
                                let socket_message = match bincode::deserialize::<SocketMessage>(&socket_message_serialized) {
                                    Ok(socket_message) => socket_message,
                                    Err(_) => break, //Break out of websocket message loop on deserialization failure.
                                };
                                if let SocketMessage::WebRtcHandshake { target_id: target, .. } = socket_message {
                                    let _ = lobby_sender.send(LobbyMessage::Message { target, socket_message_serialized });
                                }
                            }
                            //Remove this user from lobby.
                            println!("->> User {user_id} left lobby {lobby_id}");
                            let _ = lobby_sender.send(LobbyMessage::Disconnect { user_id });
                        }); //End of websocket task.
                    },
                    //On client disconnect from this lobby:
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
            println!("->> Lobby {lobby_id} removed");
            let _ = lobbies.remove(&lobby_id);
        }); //End of lobby task.

        println!("->> Lobby {lobby_id} created");

        lobby_id
    }
    pub fn join(&self, lobby_id: LobbyID, websocket: WebSocket) {
        //Send websocket to lobby if found.
        if let Some(lobby) = self.lobbies.get(&lobby_id) {
            let _ = lobby.channel.send(LobbyMessage::Connect { websocket });
        }
    }
}