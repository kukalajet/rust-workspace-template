use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use domain::{model::EmailError, usecases::RegisterUserError};
use serde::Serialize;

/// Every error a handler can return, and how it maps to HTTP.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    InvalidEmail(#[from] EmailError),

    #[error(transparent)]
    InvalidJson(#[from] JsonRejection),

    #[error(transparent)]
    RegisterUser(#[from] RegisterUserError),
}

#[derive(Debug, Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::InvalidEmail(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::InvalidJson(rejection) => rejection.status(),
            Self::RegisterUser(RegisterUserError::AlreadyRegistered(_)) => StatusCode::CONFLICT,
            Self::RegisterUser(RegisterUserError::Repository(_)) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        // Never leak internal details to clients; log them instead.
        let message = if status.is_server_error() {
            eprintln!("internal error: {self:?}");
            "internal server error".to_owned()
        } else {
            self.to_string()
        };

        (status, Json(ErrorBody { error: message })).into_response()
    }
}
