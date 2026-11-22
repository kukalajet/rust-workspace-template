use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use domain::{
    model::{Email, User},
    services::{Clock, UserRepository},
    usecases,
};
use serde::{Deserialize, Serialize};

use crate::{AppState, error::ApiError, extract::ApiJson};

/// All HTTP routes of the server, wired to the given services.
pub fn router<R, C>(state: AppState<R, C>) -> Router
where
    R: UserRepository + 'static,
    C: Clock + 'static,
{
    Router::new()
        .route("/health", get(health))
        .route("/emails/validate", post(validate_email))
        .route("/users", post(create_user::<R, C>))
        .with_state(Arc::new(state))
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
    ApiJson(body): ApiJson<ValidateEmailRequest>,
) -> Result<Json<ValidateEmailResponse>, ApiError> {
    let email = Email::parse(&body.email)?;

    Ok(Json(ValidateEmailResponse {
        email: email.to_string(),
    }))
}

#[derive(Debug, Deserialize)]
struct CreateUserRequest {
    email: String,
}

#[derive(Debug, Serialize)]
struct UserResponse {
    email: String,
    /// Unix timestamp in seconds.
    registered_at: u64,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            email: user.email.to_string(),
            registered_at: unix_seconds(user.registered_at),
        }
    }
}

/// Seconds since 1970-01-01 UTC (0 for earlier times, which a real clock never returns).
fn unix_seconds(time: SystemTime) -> u64 {
    time.duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

async fn create_user<R, C>(
    State(state): State<Arc<AppState<R, C>>>,
    ApiJson(body): ApiJson<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError>
where
    R: UserRepository,
    C: Clock,
{
    let email = Email::parse(&body.email)?;
    let user = usecases::register_user(&state.user_repository, &state.clock, email).await?;

    Ok((StatusCode::CREATED, Json(user.into())))
}
