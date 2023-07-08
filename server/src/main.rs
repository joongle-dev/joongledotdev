pub mod error;
pub mod yahtzee;

use axum::Router;
use std::net::SocketAddr;
use tower_http::services::ServeDir;

pub use crate::error::{Error, Result};

#[tokio::main]
async fn main() -> Result<()> {
    let fallback_service = ServeDir::new("assets");
    let routes = Router::new()
        .nest("/yahtzee", yahtzee::routes())
        .fallback_service(fallback_service);
    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&addr)
        .serve(routes.into_make_service())
        .await
        .unwrap();

    Ok(())
}
