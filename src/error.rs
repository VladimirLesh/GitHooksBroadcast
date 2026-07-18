use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("route not found")]
    RouteNotFound,
    #[error("unknown source: {0}")]
    UnknownSource(String),
    #[error("missing signature header")]
    MissingSignature,
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("event ignored")]
    Ignored,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, body) = match &self {
            AppError::RouteNotFound      => (StatusCode::NOT_FOUND, self.to_string()),
            AppError::UnknownSource(_)   => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::MissingSignature   => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InvalidSignature   => (StatusCode::UNAUTHORIZED, self.to_string()),
            AppError::InvalidPayload(_)  => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::Ignored            => (StatusCode::NO_CONTENT, String::new()),
        };
        (status, body).into_response()
    }
}
