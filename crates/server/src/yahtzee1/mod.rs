use std::net::SocketAddr;
use axum::{extract::{ConnectInfo, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use axum::extract::Query;
use axum::extract::ws::Message;
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/ws", get(lobby_connection_handler))
}

#[derive(Deserialize)]
struct RoomQuery {
    room: Option<String>,
}
async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    Query(query): Query<RoomQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    /*
    if query.room.is_none() {
        let _ = websocket_upgrade.on_upgrade(move |mut websocket| async move {
            match websocket.send(Message::Text("pong".into())).await {
                Ok(_) => println!("->> Successfully ponged {addr}"),
                Err(_) => println!("->> Error ponging {addr}")
            }
        });
    }
    */
    let _ = websocket_upgrade.on_upgrade(move |mut websocket| async move {
        match websocket.send(Message::Text("pong".into())).await {
            Ok(_) => println!("->> Successfully ponged {addr}"),
            Err(_) => println!("->> Error ponging {addr}")
        }
    });
}
