use crate::error::{Error, Result};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{sync::Arc, collections::BTreeMap};
use futures::{sink::SinkExt, stream::{StreamExt, SplitSink}};
use rand::Rng;
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use axum::extract::ws::{Message, WebSocket};

type UserID = u8;

#[derive(Serialize, Deserialize, Clone)]
enum SocketMessage {
    ConnectSuccess{
        user_id: UserID,
        existing_users: Vec<u8>,
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
        socket: WebSocket,
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
impl Lobby {
    fn new() {
        let (lobby_sender, mut lobby_receiver) = tokio::sync::mpsc::unbounded_channel::<LobbyMessage>();
        //Spawn a task that handles the lobby
        let lobby_task_handle = tokio::spawn(async move {
            let mut user_id_counter = 0;
            let mut users = BTreeMap::<UserID, SplitSink<WebSocket, Message>>::new();
            while let Some(lobby_message) = lobby_receiver.recv().await {
                match lobby_message {
                    LobbyMessage::Connect { socket } => {
                        
                        let (mut socket_sender, mut socket_receiver) = socket.split();

                        let user_id = user_id_counter;
                        user_id_counter += 1;
                        
                        //Construct websocket message letting the user client know that the lobby connect is successful
                        let existing_users = users.iter().map(|(k, _v)| k.clone()).collect::<Vec<_>>();
                        let socket_message = SocketMessage::ConnectSuccess { user_id, existing_users };
                        let socket_message_serialized = match bincode::serialize(&socket_message) {
                            Ok(socket_message_serialized) => socket_message_serialized,
                            Err(_) => return,
                        };

                        let _ = socket_sender.send(Message::Binary(socket_message_serialized)).await;
                        users.insert(user_id, socket_sender);

                        //Spawn a task that receives websocket messages from the user client and relay to the lobby task
                        let lobby_sender = lobby_sender.clone();
                        tokio::spawn(async move {
                            while let Some(Ok(Message::Binary(socket_message_serialized))) = socket_receiver.next().await {
                                let socket_message = match bincode::deserialize::<SocketMessage>(&socket_message_serialized) {
                                    Ok(socket_message) => socket_message,
                                    Err(_) => return,
                                };
                                match socket_message {
                                    SocketMessage::SdpOffer { target, .. } | SocketMessage::SdpAnswer { target, .. }=> {
                                        let _ = lobby_sender.send(LobbyMessage::Message { target, socket_message_serialized });
                                    },
                                    _ => {},
                                }
                            }
                            let _ = lobby_sender.send(LobbyMessage::Disconnect { user_id });
                        });
                    },
                    LobbyMessage::Disconnect { user_id } => {
                        users.remove(&user_id);
                    },
                    LobbyMessage::Message { target, socket_message_serialized } => {
                        if let Some(user) = users.get_mut(&target) {
                            let _ = user.send(Message::Binary(socket_message_serialized)).await;
                        }
                    },
                }
            }
        });
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
}