use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Query, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::get,
    Router
};
use serde::Deserialize;

pub mod lobby;
use lobby::LobbyCollection;

pub fn routes() -> Router {
    let lobby_collection = LobbyCollection::default();
    Router::new()
        .route("/ws", get(lobby_connection_handler))
        .with_state(lobby_collection)
}

#[derive(Deserialize)]
struct LobbyQuery {
    lobby_id: Option<u64>,
}
async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    State(lobby_collection): State<LobbyCollection>,
    Query(lobby_query): Query<LobbyQuery>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
    println!("->> New connection at {addr}");
    websocket_upgrade.on_upgrade(move |websocket| async move {
        let lobby_id = match lobby_query.lobby_id {
            Some(lobby_id) => lobby_id,
            None => lobby_collection.create(),
        };
        lobby_collection.join(lobby_id, websocket);
    })
}
