use axum::{
    Json, Router,
    routing::{get, post},
};
use domain::model::Email;
use serde::{Deserialize, Serialize};

use crate::error::ApiError;

/// All HTTP routes of the server.
pub fn router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/emails/validate", post(validate_email))
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Debug, Deserialize)]
struct ValidateEmailRequest {
    email: String,
}

#[derive(Debug, Serialize)]
struct ValidateEmailResponse {
    email: String,
}

async fn validate_email(
    Json(body): Json<ValidateEmailRequest>,
) -> Result<Json<ValidateEmailResponse>, ApiError> {
    let email = Email::parse(&body.email)?;

    Ok(Json(ValidateEmailResponse {
        email: email.to_string(),
    }))
}
