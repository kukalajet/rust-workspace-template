//! Drives the real router in memory: no port, no network.

// Helpers here are test code too: a panic is how a test fails.
#![allow(clippy::unwrap_used)]

use axum::{
    body::Body,
    http::{Request, StatusCode, header},
};
use http_body_util::BodyExt;
use serde_json::{Value, json};
use tower::ServiceExt;

/// Sends one request through the router and returns status + body text.
async fn send(request: Request<Body>) -> (StatusCode, String) {
    let response = server::router().oneshot(request).await.unwrap();
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

    let (status, body) = send(request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, "ok");
}

#[tokio::test]
async fn valid_email_is_normalized() {
    let request = post_json(
        "/emails/validate",
        &json!({ "email": "  Jane@Example.COM " }),
    );

    let (status, body) = send(request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(parse_json(&body), json!({ "email": "jane@example.com" }));
}

#[tokio::test]
async fn invalid_email_is_unprocessable() {
    let request = post_json("/emails/validate", &json!({ "email": "jane@" }));

    let (status, body) = send(request).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    assert_eq!(
        parse_json(&body),
        json!({ "error": "email must look like name@example.com" })
    );
}

#[tokio::test]
async fn missing_field_returns_json_error() {
    let request = post_json("/emails/validate", &json!({ "mail": "jane@example.com" }));

    let (status, body) = send(request).await;

    assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
    let error = parse_json(&body)["error"].as_str().unwrap().to_owned();
    assert!(error.contains("missing field `email`"), "{error}");
}

#[tokio::test]
async fn missing_content_type_returns_json_error() {
    let request = Request::post("/emails/validate")
        .body(Body::from(r#"{"email":"jane@example.com"}"#))
        .unwrap();

    let (status, body) = send(request).await;

    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    let error = parse_json(&body)["error"].as_str().unwrap().to_owned();
    assert!(error.contains("Content-Type: application/json"), "{error}");
}
