use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// JSON error response body: `{ "error": "message" }`.
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

/// Application-level error type that renders as a JSON error response.
#[derive(Debug)]
pub enum AppError {
    /// 404 Not Found — resource does not exist.
    NotFound(String),
    /// 400 Bad Request — invalid input from client.
    BadRequest(String),
    /// 500 Internal Server Error — unexpected server-side failure.
    /// The inner string is logged via tracing but NOT exposed to the client.
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        // NOTE: 500 errors are logged at the call site (handlers) with full context.
        // into_response only builds the HTTP response — no duplicate logging here.
        let (status, message) = match self {
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal server error".to_string(),
            ),
        };
        (status, Json(ErrorResponse { error: message })).into_response()
    }
}

impl From<JsonRejection> for AppError {
    fn from(rejection: JsonRejection) -> Self {
        AppError::BadRequest(rejection.body_text())
    }
}
