use crate::config::AppConfig;
use crate::routes::auth::pin_required;
use crate::routes::auth::verify_pin::verify_pin;
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
    use shared_backend::server::ServerConfig;
    let mut server = ServerConfig::from_env("TEST");
    server.pin = pin;
    server.port = 4406;
    server.site_title = "TestPulse".to_string();
    let config = AppConfig {
        server: Arc::new(server),
        refresh_interval: 2,
        monitor_cpu: true,
        monitor_memory: true,
        monitor_storage: true,
        monitor_network: true,
        monitor_gpu: true,
        enable_coffee: true,
    };
    AppState::new(config, Arc::new(RwLock::new(None)))
}

#[test]
fn session_id_format() {
    let id = crate::session_id::generate_session_id();
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
