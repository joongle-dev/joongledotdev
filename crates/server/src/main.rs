pub mod error;
pub mod yahtzee;

use axum::{
    Router,
    routing::get,
    http::{Uri, StatusCode},
    extract::Host,
    response::{Html, Redirect, IntoResponse},
    handler::HandlerWithoutStateExt
};
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
    let routes = Router::new()
        .fallback_service(ServeDir::new("assets")
            .precompressed_gzip()
            .not_found_service(ServeFile::new("assets/not_found.html")))
        .nest("/yahtzee", yahtzee::routes())
        .route("/hello", get(hello));

    match RustlsConfig::from_pem_file(CERT_FILE, KEY_FILE).await {
        Ok(config) => {
            println!("->> Found certificates!, Running in encrypted mode.");
            tokio::spawn(redirect_http_to_https());
            let addr = SocketAddr::from((IP_ADDR, HTTPS_PORT));
            println!("->> Listening on {addr}.");
            let _ = axum_server::bind_rustls(addr, config).serve(routes.into_make_service_with_connect_info::<SocketAddr>()).await;
        }
        Err(error) => {
            println!("->> Failed to validate certificates: {error}. Running in unencrypted mode.");
            let addr = SocketAddr::from((IP_ADDR, HTTP_PORT));
            println!("->> Listening on {addr}.");
            let _ = axum_server::bind(addr).serve(routes.into_make_service_with_connect_info::<SocketAddr>()).await;
        }
    }

    Ok(())
}

async fn redirect_http_to_https() {
    fn make_https(host: String, uri: Uri) -> core::result::Result<Uri, axum::BoxError> {
        let mut parts = uri.into_parts();
        parts.scheme = Some(axum::http::uri::Scheme::HTTPS);
        if parts.path_and_query.is_none() {
            parts.path_and_query = Some("/".parse().unwrap());
        }
        let https_host = host.replace(&HTTP_PORT.to_string(), &HTTPS_PORT.to_string());
        parts.authority = Some(https_host.parse()?);
        Ok(Uri::from_parts(parts)?)
    }

    let redirect = move |Host(host): Host, uri: Uri| async move {
        match make_https(host, uri) {
            Ok(uri) => Ok(Redirect::permanent(&uri.to_string())),
            Err(_) => Err(StatusCode::BAD_REQUEST)
        }
    };

    let addr = SocketAddr::from((IP_ADDR, HTTP_PORT));
    let _ = axum_server::bind(addr).serve(redirect.into_make_service()).await;
}

async fn hello() -> impl IntoResponse {
    Html("Hello")
}