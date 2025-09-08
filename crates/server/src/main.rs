pub mod error;
pub mod yahtzee;
mod yahtzee1;

use axum::Router;
use axum_server::tls_rustls::RustlsConfig;
use std::net::SocketAddr;
use tower_http::services::{ServeDir, ServeFile};

pub use crate::error::Result;

const IP_ADDR: [u8; 4] = [0, 0, 0, 0];
const HTTP_PORT: u16 = 8000;
const HTTPS_PORT: u16 = 8001;
const CERT_FILE: &str = "certs/cert.pem";
const KEY_FILE: &str = "certs/key.pem";

#[tokio::main]
async fn main() -> Result<()> {
    let https_routes = Router::new()
        .fallback_service(ServeDir::new("assets").precompressed_gzip().not_found_service(ServeFile::new("assets/not_found.html")))
        .nest("/yahtzee", yahtzee::routes())
        .nest("/yahtzee1", yahtzee1::routes());
    let http_routes = Router::new()
        .nest_service("/.well-known/acme-challenge", ServeDir::new("assets/.well-known/acme-challenge"));
    match RustlsConfig::from_pem_file(CERT_FILE, KEY_FILE).await {
        Ok(config) => {
            println!("->> Found certificates!, Running in encrypted mode.");
            let https_addr = SocketAddr::from((IP_ADDR, HTTPS_PORT));
            let https = tokio::task::spawn(axum_server::bind_rustls(https_addr, config).serve(https_routes.into_make_service_with_connect_info::<SocketAddr>()));
            let http_addr = SocketAddr::from((IP_ADDR, HTTP_PORT));
            let http = tokio::task::spawn(axum_server::bind(http_addr).serve(http_routes.into_make_service_with_connect_info::<SocketAddr>()));
            let _ = tokio::join!(https, http);
        }
        Err(error) => {
            println!("->> Failed to validate certificates: {error}.");
        }
    }

    Ok(())
}