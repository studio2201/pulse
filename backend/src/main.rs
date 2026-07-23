use axum::{Router, middleware, routing::get};
use shared_backend::middleware::{
    HstsState, TitleState, cors_layer, hsts_layer, security_headers_layer, title_injection_layer,
};
use shared_backend::server::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;

mod config;
mod cookie_auth;
mod routes;
mod services;
mod session_id;
mod state;
mod utils;

#[cfg(test)]
mod tests;

use config::AppConfig;
use routes::{auth, stats};
use services::monitor;
use state::AppState;

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    // 1. Logging setup
    shared_backend::tracing_init::init_tracing(
        shared_backend::tracing_init::default_log_dir().as_deref(),
    );

    // 2. Load Configuration and Shared State
    let config = AppConfig::load();
    let shared_stats = Arc::new(tokio::sync::RwLock::new(None));
    let state = AppState::new(config.clone(), shared_stats);

    // Start system metrics loop
    monitor::start_monitor(state.clone());

    utils::pwa::generate_pwa_manifest(&config.server.site_title);

    // Background cleanup task for per-IP rate-limiting
    let state_clone = state.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(60)).await;
            state_clone.clean_old_rate_limits(RATE_LIMIT_WINDOW).await;
        }
    });

    let server_config = Arc::new(ServerConfig::from_env("PULSE"));
    let cors = cors_layer(&server_config);

    // 3. API Routes setup
    let api_routes = Router::new()
        .route("/stats", get(stats::handle_get_stats))
        .route("/stats/ws", get(stats::handle_ws_stats))
        .route("/verify-pin", axum::routing::post(auth::verify_pin))
        .route("/logout", axum::routing::post(auth::logout))
        .route(
            "/auth-check",
            axum::routing::get(auth::auth_check).layer(middleware::from_fn_with_state(
                state.clone(),
                auth::require_pin,
            )),
        )
        .route("/pin-required", axum::routing::get(auth::pin_required))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::rate_limit_middleware,
        ))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth::origin_validation_middleware,
        ));

    // 4. Main App Routing
    let app = Router::new()
        .nest("/api", api_routes)
        .route("/config", get(serve_config))
        .route("/health", get(health_check))
        .route("/service-worker.js", get(serve_service_worker))
        .fallback_service(ServeDir::new("frontend/dist"))
        .layer(middleware::from_fn_with_state(
            TitleState(server_config.clone()),
            title_injection_layer,
        ))
        .layer(axum::middleware::from_fn(fix_content_length_middleware))
        .layer(middleware::from_fn_with_state(
            HstsState(server_config.clone()),
            hsts_layer,
        ))
        .layer(middleware::from_fn(security_headers_layer))
        .layer(tower_http::trace::TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    let listener = match tokio::net::TcpListener::bind(format!("[::]:{}", config.server.port)).await {
        Ok(l) => {
            tracing::info!("Starting dual-stack server on [::]:{}", config.server.port);
            l
        }
        Err(e) => {
            tracing::warn!(
                "Failed to bind IPv6 [::]:{} ({:?}). Falling back to IPv4 0.0.0.0:{}",
                config.server.port,
                e,
                config.server.port
            );
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.server.port))
                .await
                .expect("failed to bind to IPv4")
        }
    };
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(graceful_shutdown())
    .await
    .expect("server error");
}

async fn serve_config(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "siteTitle": state.config.server.site_title,
        "pinRequired": state.config.server.pin.is_some(),
        "pinLength": state.config.server.pin.as_ref().map_or(0, |p| p.len()),
        "enableTranslation": state.config.server.enable_translation,
        "enable_translation": state.config.server.enable_translation,
        "enableThemes": state.config.server.enable_themes,
        "enable_themes": state.config.server.enable_themes,
        "enablePrint": state.config.server.enable_print,
        "enable_print": state.config.server.enable_print,
        "monitorCpu": state.config.monitor_cpu,
        "monitorMemory": state.config.monitor_memory,
        "monitorStorage": state.config.monitor_storage,
        "monitorNetwork": state.config.monitor_network,
        "monitorGpu": state.config.monitor_gpu,
        "enableCoffee": state.config.enable_coffee,
    }))
}

async fn health_check() -> impl axum::response::IntoResponse {
    (axum::http::StatusCode::OK, "OK")
}

async fn graceful_shutdown() {
    use tokio::signal::unix::{SignalKind, signal};

    let mut sigint = signal(SignalKind::interrupt()).expect("install SIGINT handler");
    let mut sigterm = signal(SignalKind::terminate()).expect("install SIGTERM handler");

    tokio::select! {
        _ = sigint.recv() => tracing::info!("received SIGINT"),
        _ = sigterm.recv() => tracing::info!("received SIGTERM"),
    }

    tracing::info!("draining connections (5s)");
    tokio::time::sleep(Duration::from_secs(5)).await;
}

async fn fix_content_length_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let mut response = next.run(request).await;

    let is_html = response
        .headers()
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|s| s.starts_with("text/html"));

    if is_html {
        response
            .headers_mut()
            .remove(axum::http::header::CONTENT_LENGTH);
    }

    response
}

async fn serve_service_worker() -> impl axum::response::IntoResponse {
    let content = std::fs::read_to_string("frontend/dist/service-worker.js").unwrap_or_default();
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/javascript"),
            (
                axum::http::header::CACHE_CONTROL,
                "no-cache, no-store, must-revalidate",
            ),
            (axum::http::header::EXPIRES, "0"),
        ],
        content,
    )
}
