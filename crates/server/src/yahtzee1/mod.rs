use std::net::SocketAddr;
use std::sync::Arc;
use axum::{extract::{ConnectInfo, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use axum::extract::{Query, State};
use axum::extract::ws::{Message, WebSocket};
use dashmap::DashMap;
use futures::stream::{FuturesUnordered, SplitSink};
use futures::{FutureExt, SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc::UnboundedSender;
use signaling_protocol::{RoomID, PeerID, PeerRequest, PeerEvent};

const ROOM_SIZE: usize = 4;

enum RoomMessage {
    Connect(WebSocket),
    Disconnect(PeerID),
    Forward(PeerID, Message),
}
#[derive(Clone)]
struct Rooms(Arc<DashMap<RoomID, UnboundedSender<RoomMessage>>>);
impl Rooms {
    pub fn join(&self, room_id: Option<RoomID>, socket: WebSocket) {

        if let Some(room_id) = room_id {

        }
        else {
            let (mut room_sender, mut room_receiver) = tokio::sync::mpsc::unbounded_channel::<RoomMessage>();
            let (socket_sender, socket_receiver) = socket.split();
            let room_id = rand::random::<RoomID>();
            self.0.insert(room_id, room_sender.clone());

            // Task: Manage connected peers and handle signal forwarding
            tokio::spawn(async move {
                let mut slots: [Option<SplitSink<WebSocket, Message>>; ROOM_SIZE] = Default::default();
                slots[0] = Some(socket_sender);
                while let Some(room_message) = room_receiver.recv().await {
                    match room_message {
                        // Peer connect request
                        RoomMessage::Connect(socket) => {
                            if let Some(peer_id) = slots.iter().position(|s| s.is_none()).map(|x| x as PeerID) {
                                let (socket_sender, mut socket_receiver) = socket.split();
                                let message_serialized = bincode::serialize(&PeerEvent::PeerConnect(peer_id)).unwrap();
                                slots.iter_mut()
                                    .filter_map(|slot| slot.as_mut())
                                    .map(|peer| peer.send(Message::Binary(message_serialized.into())))
                                    .collect::<FuturesUnordered<_>>()
                                    .collect::<Vec<_>>();
                                slots[peer_id as usize] = Some(socket_sender);

                                // Task: Forward incoming peer request
                                let room_sender = room_sender.clone();
                                tokio::spawn(async move {
                                    while let Some(Ok(Message::Binary(message_serialized))) = socket_receiver.next().await {
                                        if let PeerRequest::Signal(target, data) = bincode::deserialize::<PeerRequest>(&message_serialized).unwrap() {
                                            room_sender.send(RoomMessage::Forward(target, Message::Binary(bincode::serialize(&PeerEvent::Signal(peer_id, data)).unwrap().into()))).unwrap();
                                        }
                                    }
                                });
                            }
                        }
                        // Peer disconnected
                        RoomMessage::Disconnect(peer_id) => {
                            let message_serialized = bincode::serialize(&PeerEvent::PeerDisconnect(peer_id)).unwrap();
                            slots[peer_id as usize] = None;
                            slots.iter_mut()
                                .filter_map(|slot| slot.as_mut())
                                .map(|peer| peer.send(Message::Binary(message_serialized.into())))
                                .collect::<FuturesUnordered<_>>()
                                .collect::<Vec<_>>()
                                .await;
                        }
                        // Forward signal
                        RoomMessage::Forward(peer_id, socket_message) => {
                            if let Some(Some(peer)) = slots.get_mut(peer_id as usize) {
                                peer.send(socket_message).await.unwrap();
                            }
                        }
                    }
                }
            });
        }
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/ws", get(lobby_connection_handler))
        .with_state(Rooms(Default::default()))
}

#[derive(Deserialize)]
struct RoomQuery {
    room_id: Option<u64>,
}
async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    State(rooms): State<Rooms>,
    Query(query): Query<RoomQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("->> New connection at {addr}");
    websocket_upgrade.on_upgrade(move |mut websocket| async move {
        match websocket.send(Message::Text("pong".into())).await {
            Ok(_) => println!("->> Successfully ponged {addr}"),
            Err(_) => println!("->> Error ponging {addr}")
        }
    })
}
