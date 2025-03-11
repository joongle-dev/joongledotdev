use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use axum::{extract::{ConnectInfo, WebSocketUpgrade}, routing::get, Router};
use axum::extract::ws::{Message, WebSocket};
use axum::response::Response;
use futures::stream::SplitSink;
use futures::sink::SinkExt;
use futures::StreamExt;
use serde::{Deserialize, Serialize};

pub fn routes() -> Router {
    Router::new()
        .route("/ws", get(lobby_connection_handler))
}

async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Response {
    websocket_upgrade.on_upgrade(move |mut websocket| async move {
        let (mut sender, mut receiver) = websocket.split();
        while let Some(Ok(Message::Text(message))) = receiver.next().await {
            if let Ok(message) = serde_json::from_str::<ClientEvent>(&message) {
                let _ = sender.send(Message::text("received client event")).await;
            }
            else {
                let _ = sender.send(Message::text("invalid client event")).await;
            }
        }
    })
}

struct Player(SplitSink<WebSocket, Message>);
struct PlayerList([Option<Player>; 4]);
struct Room(Arc<Mutex<PlayerList>>);
#[derive(Deserialize, Clone)]
#[serde(tag = "type")]
enum ClientEvent {
    Join {
        room: Option<String>,
        name: String,
    }
}
#[derive(Serialize, Clone)]
#[serde(tag = "type")]
enum ServerEvent {
    JoinSuccess {
        id: usize,
    },
    JoinFail {
        reason: String,
    },
    Join {
        name: String,
    }
}