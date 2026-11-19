use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::model::EmailError;
use serde::Serialize;

/// Every error a handler can return, and how it maps to HTTP.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    InvalidEmail(#[from] EmailError),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::InvalidEmail(_) => StatusCode::UNPROCESSABLE_ENTITY,
        };

        let body = ErrorBody {
            error: self.to_string(),
        };

        (status, Json(body)).into_response()
    }
}
