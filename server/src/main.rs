pub mod error;
pub mod yahtzee;

use axum::Router;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

pub use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let routes = Router::new()
        .fallback_service(ServeDir::new("assets").fallback(ServeFile::new("assets/not_found.html")))
        .nest("/", yahtzee::routes());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));
    println!("->> Listening on {addr}");
    axum::Server::bind(&addr)
        .serve(routes.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();

    Ok(())
}
