use crate::config::AppConfig;
use crate::routes::auth::pin_required;
use crate::routes::auth::verify_pin::{generate_session_id, verify_pin};
use crate::state::AppState;
use axum::{
    Json,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

fn test_state(pin: Option<String>) -> AppState {
    AppState::new(
        AppConfig {
            port: 4406,
            site_title: "TestPulse".to_string(),
            pin,
            max_attempts: 5,
            lockout_time_minutes: 15,
            cookie_max_age_hours: 24,
            trust_proxy: false,
            trusted_proxies: vec![],
            allowed_origins: "*".to_string(),
            base_url: "/".to_string(),
            enable_translation: false,
            enable_themes: true,
            enable_print: false,
            show_version: true,
            show_github: true,
            refresh_interval: 2,
            monitor_cpu: true,
            monitor_memory: true,
            monitor_storage: true,
            monitor_network: true,
            monitor_gpu: true,
            monitor_console: false,
        },
        Arc::new(RwLock::new(None)),
    )
}

#[test]
fn session_id_format() {
    let id = generate_session_id();
    assert_eq!(id.len(), 32);
    assert!(id.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn health_check_returns_ok() {
    let res = crate::health_check().await.into_response();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn pin_required_when_set() {
    let state = test_state(Some("123456".to_string()));
    let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();

    let res = pin_required(headers, connect_info, State(state))
        .await
        .into_response();
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn verify_pin_empty_returns_bad_request() {
    let state = test_state(Some("123456".to_string()));
    let connect_info = ConnectInfo(SocketAddr::from(([127, 0, 0, 1], 12345)));
    let headers = HeaderMap::new();
    let payload = crate::routes::auth::VerifyPinPayload {
        pin: Some("".to_string()),
    };

    let res = verify_pin(headers, connect_info, State(state), Json(payload))
        .await
        .into_response();
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}
