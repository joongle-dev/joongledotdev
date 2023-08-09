use axum::extract::ws::WebSocket;
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use serde::Deserialize;

mod lobby;
use lobby::LobbyCollection;

pub fn routes() -> Router {
    let lobby_collection = LobbyCollection::new();
    Router::new()
        .route("/ws", get(lobby_connection_handler))
        .with_state(lobby_collection)
}

#[derive(Deserialize)]
struct LobbyQuery {
    lobby_id: Option<u64>,
}
async fn lobby_connection_handler(
    socket_upgrade: WebSocketUpgrade,
    State(lobby_collection): State<LobbyCollection>,
    Query(lobby_query): Query<LobbyQuery>,
) -> impl IntoResponse {
    socket_upgrade.on_upgrade(move |socket| lobby_handle_socket(socket, lobby_collection, lobby_query.lobby_id))
}
async fn lobby_handle_socket(stream: WebSocket, lobby_collection: LobbyCollection, lobby_id: Option<u64>) {
}
