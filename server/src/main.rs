pub mod error;
pub mod yahtzee;

use axum::Router;
use axum::extract::{WebSocketUpgrade, State, Query, ConnectInfo};
use axum::response::IntoResponse;
use axum::routing::get;
use yahtzee::lobby::LobbyCollection;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_websockets=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let lobby_collection = LobbyCollection::new();
    let routes = Router::new()
        .fallback_service(ServeDir::new("assets").fallback(ServeFile::new("assets/not_found.html")))
        .route("/ws", get(lobby_connection_handler))
        .with_state(lobby_collection)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );

        /* 
    let routes = Router::new()
        .fallback_service(ServeDir::new("assets").fallback(ServeFile::new("assets/not_found.html")))
        .nest("/yahtzee", yahtzee::routes())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
        */
    

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    println!("->> Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();
    
    Ok(())
}

async fn lobby_connection_handler(
    websocket_upgrade: WebSocketUpgrade,
    State(lobby_collection): State<LobbyCollection>,
    ConnectInfo(addr): ConnectInfo<SocketAddr> 
) -> impl IntoResponse {
    println!("->> New connection at {addr}");
    websocket_upgrade.on_upgrade(move |websocket| async move {
        let lobby_id = lobby_collection.create();
        lobby_collection.join(lobby_id, websocket);
    })
}