use std::net::SocketAddr;

use axum::{extract::{ConnectInfo, WebSocketUpgrade}, response::IntoResponse, routing::get, Router};
use axum::extract::ws::Message;

pub fn routes() -> Router {
    Router::new()
        .route("/ws", get(lobby_connection_handler))
}

async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("->> New connection at {addr}");
    websocket_upgrade.on_upgrade(move |mut websocket| async move {
        match websocket.send(Message::Text("pong".to_owned())).await {
            Ok(_) => println!("->> Successfully ponged {addr}"),
            Err(_) => println!("->> Error ponging {addr}")
        }
    })
}
