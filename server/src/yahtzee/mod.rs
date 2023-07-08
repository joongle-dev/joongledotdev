use self::lobby::{LobbyCollection, LobbyEvent, User};
use crate::error::Result;
use axum::extract::ws::{Message, WebSocket};
use axum::extract::{Query, State, WebSocketUpgrade};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use futures::{sink::SinkExt, stream::StreamExt};
use rand::Rng;
use serde::Deserialize;

mod lobby;

pub fn routes() -> Router {
    let lobbies = LobbyCollection::new();
    Router::new()
        .route("/ws", get(lobby_connection_handler))
        .with_state(lobbies)
}

#[derive(Deserialize)]
struct LobbyQuery {
    lobby_id: Option<u64>,
}
async fn lobby_connection_handler(
    ws: WebSocketUpgrade,
    Query(lobby_query): Query<LobbyQuery>,
    State(lobbies): State<LobbyCollection>,
) -> Result<impl IntoResponse> {
    let user = match lobby_query.lobby_id {
        Some(lobby_id) => lobbies.join_lobby(lobby_id)?,
        None => lobbies.create_lobby(rand::thread_rng().gen::<u64>())?,
    };
    Ok(ws.on_upgrade(|ws| lobby_handle_socket(ws, user)))
}
async fn lobby_handle_socket(stream: WebSocket, user: User) {
    let (mut sender, mut receiver) = stream.split();
    let User { id, tx, mut rx } = user;

    //Receive broadcast from other users and relay to client if client is target.
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let LobbyEvent::SdpOffer { target, .. } | LobbyEvent::SdpAnswer { target, .. } =
                event
            {
                if target != id {
                    continue;
                }
            }
            if bincode::serialize(&event)
                .map(|serialized_event| sender.send(Message::Binary(serialized_event)))
                .is_err()
            {
                break;
            }
        }
    });
    //Receive message from client and broadcast to other users.
    let tx_clone = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(Message::Binary(serialized_event))) = receiver.next().await {
            if let Ok(event) = bincode::deserialize::<LobbyEvent>(serialized_event.as_slice()) {
                let _ = tx_clone.send(event);
            } else {
                break;
            }
        }
    });
    //Abort the other if one of the task ends.
    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    };

    if tx.send(LobbyEvent::Leave { id: id }).is_err() {
        //TODO: Remove lobby if there are no more users subscribed to the channel.
    }
}
