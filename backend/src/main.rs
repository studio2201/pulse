use axum::{Router, middleware, routing::get};
use shared_backend::middleware::{
    HstsState, TitleState, cors_layer, hsts_layer, security_headers_layer, title_injection_layer,
};
use shared_backend::server::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tower_http::services::ServeDir;
use tracing_subscriber::{Layer, layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod routes;
mod services;
mod state;

use config::AppConfig;
use routes::{auth, stats};
use services::monitor;
use state::AppState;

const RATE_LIMIT_WINDOW: Duration = Duration::from_secs(60);

#[tokio::main]
async fn main() {
    // 1. Logging setup
    let log_dir = std::env::var("LOG_DIR").ok().or_else(|| {
        let data_dir = std::path::Path::new("/app/data");
        if data_dir.is_dir() {
            Some("/app/data/log".to_string())
        } else {
            Some("/app/log".to_string())
        }
    });

    let (file_layer_error, file_layer_app) = if let Some(ref dir) = log_dir {
        if dir == "off" || dir == "none" || dir == "false" {
            (None, None)
        } else {
            let _ = std::fs::create_dir_all(dir);
            let error_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("error.log"))
                .ok();
            let app_file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(std::path::Path::new(dir).join("app.log"))
                .ok();

            let error_layer = error_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::WARN)
            });

            let app_layer = app_file.map(|file| {
                tracing_subscriber::fmt::layer()
                    .with_writer(std::sync::Mutex::new(file))
                    .with_ansi(false)
                    .with_filter(tracing_subscriber::filter::LevelFilter::INFO)
            });

            (error_layer, app_layer)
        }
    } else {
        (None, None)
    };

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .with(file_layer_error)
        .with(file_layer_app)
        .init();

    // 2. Load Configuration and Shared State
    let config = AppConfig::load();
    let shared_stats = Arc::new(tokio::sync::RwLock::new(None));
    let state = AppState::new(config.clone(), shared_stats);

    // Start system metrics loop
    monitor::start_monitor(state.clone());

    generate_pwa_manifest(&config.site_title);

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

    let listener = match tokio::net::TcpListener::bind(format!("[::]:{}", config.port)).await {
        Ok(l) => {
            tracing::info!("Starting dual-stack server on [::]:{}", config.port);
            l
        }
        Err(e) => {
            tracing::warn!("Failed to bind IPv6 [::]:{} ({:?}). Falling back to IPv4 0.0.0.0:{}", config.port, e, config.port);
            tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.port))
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

fn generate_pwa_manifest(site_title: &str) {
    let dist_dir = std::path::Path::new("frontend/dist");
    if !dist_dir.exists() {
        return;
    }

    let pwa_manifest = serde_json::json!({
        "name": site_title,
        "short_name": site_title,
        "description": "Minimalist server status and visor telemetry dashboard",
        "start_url": "/",
        "display": "standalone",
        "background_color": "#ffffff",
        "theme_color": "#000000",
        "icons": [
            { "src": "favicon.svg", "type": "image/svg+xml", "sizes": "any" },
            { "src": "favicon.png", "type": "image/png", "sizes": "192x192" },
            { "src": "favicon.png", "type": "image/png", "sizes": "512x512" }
        ],
        "orientation": "any"
    });

    let _ = std::fs::write(
        dist_dir.join("manifest.json"),
        serde_json::to_string_pretty(&pwa_manifest).unwrap_or_default(),
    );

    // Also generate asset-manifest.json for service worker registration
    let mut assets = Vec::new();
    fn walk_dir(dir: &std::path::Path, prefix: &str, assets: &mut Vec<String>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let rel = if prefix.is_empty() {
                    entry.file_name().to_string_lossy().to_string()
                } else {
                    format!("{}/{}", prefix, entry.file_name().to_string_lossy())
                };
                if path.is_dir() {
                    walk_dir(&path, &rel, assets);
                } else {
                    assets.push(format!("/{}", rel));
                }
            }
        }
    }
    walk_dir(dist_dir, "", &mut assets);
    let _ = std::fs::write(
        dist_dir.join("asset-manifest.json"),
        serde_json::to_string_pretty(&assets).unwrap_or_default(),
    );
}

async fn serve_config(axum::extract::State(state): axum::extract::State<AppState>) -> impl axum::response::IntoResponse {
    axum::Json(serde_json::json!({
        "siteTitle": state.config.site_title,
        "pinRequired": state.config.pin.is_some(),
        "pinLength": state.config.pin.as_ref().map_or(0, |p| p.len()),
        "enableTranslation": state.config.enable_translation,
        "enable_translation": state.config.enable_translation,
        "enableThemes": state.config.enable_themes,
        "enable_themes": state.config.enable_themes,
        "enablePrint": state.config.enable_print,
        "enable_print": state.config.enable_print,
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
        response.headers_mut().remove(axum::http::header::CONTENT_LENGTH);
    }

    response
}

async fn serve_service_worker() -> impl axum::response::IntoResponse {
    let content = std::fs::read_to_string("frontend/dist/service-worker.js").unwrap_or_default();
    (
        [
            (axum::http::header::CONTENT_TYPE, "application/javascript"),
            (axum::http::header::CACHE_CONTROL, "no-cache, no-store, must-revalidate"),
            (axum::http::header::EXPIRES, "0"),
        ],
        content,
    )
}
