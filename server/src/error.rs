use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Clone, Debug, Serialize)]
pub enum Error {
    YahtzeeLobbyAlreadyExists,
    YahtzeeLobbyNotFound,
    YahtzeeMessageSerializationError,
}

impl core::fmt::Display for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter) -> core::result::Result<(), core::fmt::Error> {
        write!(fmt, "{self:?}")
    }
}

impl std::error::Error for Error {}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let mut response = StatusCode::INTERNAL_SERVER_ERROR.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

impl Error {
    pub fn client_status_and_error(&self) -> (StatusCode, &'static str) {
        match self {
            Self::YahtzeeLobbyNotFound => (StatusCode::BAD_REQUEST, "INVALID_LOBBY"),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "SERVICE_ERROR"),
        }
    }
}
