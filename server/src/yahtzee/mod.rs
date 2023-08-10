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
    websocket_upgrade: WebSocketUpgrade,
    State(lobby_collection): State<LobbyCollection>,
    Query(lobby_query): Query<LobbyQuery>,
) -> impl IntoResponse {
    websocket_upgrade.on_upgrade(move |websocket| async move {
        let lobby_id = match lobby_query.lobby_id {
            Some(lobby_id) => lobby_id,
            None => lobby_collection.create(),
        };
        lobby_collection.join(lobby_id, websocket);
    })
}