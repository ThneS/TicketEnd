use thiserror::Error;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("not found")] NotFound,
    #[error("unauthorized")] Unauthorized,
    #[error("forbidden")] Forbidden,
    #[error("bad request: {0}")] BadRequest(String),
    #[error("conflict: {0}")] Conflict(String),
    #[error("internal error")] Internal,
}

#[derive(Serialize)]
struct ErrorBody { error_code: &'static str, message: String }

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, msg) = match &self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "UNAUTHORIZED", self.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "FORBIDDEN", self.to_string()),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", m.clone()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, "CONFLICT", m.clone()),
            ApiError::Internal => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL", self.to_string()),
        };
        let body = axum::Json(ErrorBody { error_code: code, message: msg });
        (status, body).into_response()
    }
}
