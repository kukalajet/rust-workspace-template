//! Drives the real router in memory: no port, no network.

// Helpers here are test code too: a panic is how a test fails.
#![allow(clippy::unwrap_used)]

use std::{
    future,
    time::{Duration, SystemTime},
};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use domain::{
    model::User,
    services::{Clock, RepositoryError, UserRepository},
};
use http_body_util::BodyExt;
use memory::InMemoryUserRepository;
use serde_json::{Value, json};
use server::AppState;
use tower::ServiceExt;

/// A clock frozen at 2023-11-14T22:13:20Z.
struct FixedClock;

impl Clock for FixedClock {
    fn now(&self) -> SystemTime {
        SystemTime::UNIX_EPOCH + Duration::from_secs(1_700_000_000)
    }
}

/// Fails every operation, to test the 500 path.
struct BrokenRepository;

impl UserRepository for BrokenRepository {
    fn insert(&self, _user: &User) -> impl Future<Output = Result<(), RepositoryError>> + Send {
        future::ready(Err(RepositoryError::Unavailable("disk on fire".into())))
    }
}

/// The app wired with an empty in-memory store and a fixed clock.
fn app() -> Router {
    server::router(AppState {
        user_repository: InMemoryUserRepository::default(),
        clock: FixedClock,
    })
}

/// Sends one request through the router and returns status + body text.
async fn send(app: &Router, request: Request<Body>) -> (StatusCode, String) {
    let response = app.clone().oneshot(request).await.unwrap();
    let status = response.status();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();

    (status, String::from_utf8(bytes.to_vec()).unwrap())
}

fn post_json(uri: &str, body: &Value) -> Request<Body> {
    Request::post(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(body.to_string()))
        .unwrap()
}

fn parse_json(body: &str) -> Value {
    serde_json::from_str(body).unwrap()
}

#[tokio::test]
async fn health_returns_ok() {
    let request = Request::get("/health").body(Body::empty()).unwrap();

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn valid_email_is_normalized() {
    let request = post_json(
        "/emails/validate",
        &json!({ "email": "  Jane@Example.COM " }),
    );

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse_json(&body), json!({ "email": "jane@example.com" }));
}

#[tokio::test]
async fn invalid_email_is_unprocessable() {
    let request = post_json("/emails/validate", &json!({ "email": "jane@" }));

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        parse_json(&body),
        json!({ "error": "email must look like name@example.com" })
    );
}

#[tokio::test]
async fn missing_field_returns_json_error() {
    let request = post_json("/emails/validate", &json!({ "mail": "jane@example.com" }));

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let error = parse_json(&body)["error"].as_str().unwrap().to_owned();
    assert!(error.contains("missing field `email`"), "{error}");
}

#[tokio::test]
async fn missing_content_type_returns_json_error() {
    let request = Request::post("/emails/validate")
        .body(Body::from(r#"{"email":"jane@example.com"}"#))
        .unwrap();

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let error = parse_json(&body)["error"].as_str().unwrap().to_owned();
    assert!(error.contains("Content-Type: application/json"), "{error}");
}

#[tokio::test]
async fn creates_a_user() {
    let request = post_json("/users", &json!({ "email": "Jane@Example.com" }));

    let (status, body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(
        parse_json(&body),
        json!({ "email": "jane@example.com", "registered_at": 1_700_000_000 })
    );
}

#[tokio::test]
async fn rejects_a_duplicate_user() {
    let app = app();
    send(
        &app,
        post_json("/users", &json!({ "email": "jane@example.com" })),
    )
    .await;

    let (status, body) = send(
        &app,
        post_json("/users", &json!({ "email": "JANE@example.com" })),
    )
    .await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        parse_json(&body),
        json!({ "error": "a user with email jane@example.com is already registered" })
    );
}

#[tokio::test]
async fn rejects_an_invalid_user_email() {
    let request = post_json("/users", &json!({ "email": "jane@" }));

    let (status, _body) = send(&app(), request).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn hides_storage_failures_from_clients() {
    let app = server::router(AppState {
        user_repository: BrokenRepository,
        clock: FixedClock,
    });
    let request = post_json("/users", &json!({ "email": "jane@example.com" }));

    let (status, body) = send(&app, request).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        parse_json(&body),
        json!({ "error": "internal server error" })
    );
}
