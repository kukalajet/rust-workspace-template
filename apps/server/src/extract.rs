use axum::extract::FromRequest;

use crate::error::ApiError;

/// Like [`axum::Json`], but a malformed body is reported as an [`ApiError`],
/// so clients get the same JSON error shape for every failure.
#[derive(Debug, FromRequest)]
#[from_request(via(axum::Json), rejection(ApiError))]
pub struct ApiJson<T>(pub T);
