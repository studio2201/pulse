use axum::{
    Json,
    extract::{State, WebSocketUpgrade, ws::{WebSocket, Message}},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use std::time::Duration;


use crate::state::AppState;
use crate::routes::auth::is_authenticated;

/// HTTP GET stats handler
pub async fn handle_get_stats(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    if !is_authenticated(&headers, &state).await {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let stats_lock = state.shared_stats.read().await;
    match &*stats_lock {
        Some(stats) => Ok(Json(stats.clone())),
        None => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

/// WebSocket stats handler
pub async fn handle_ws_stats(
    headers: HeaderMap,
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if !is_authenticated(&headers, &state).await {
        return StatusCode::UNAUTHORIZED.into_response();
    }

    ws.on_upgrade(move |socket| handle_ws_connection(socket, state))
}

async fn handle_ws_connection(mut socket: WebSocket, state: AppState) {
    tracing::info!("New WebSocket connection established!");
    let interval_secs = state.config.refresh_interval;
    let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        let stats_opt = {
            let stats_lock = state.shared_stats.read().await;
            stats_lock.clone()
        };

        if let Some(stats) = stats_opt {
            let json_str = match serde_json::to_string(&stats) {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to serialize stats: {:?}", e);
                    continue;
                }
            };
            tracing::debug!("Sending stats payload to WebSocket client");
            if let Err(e) = socket.send(Message::Text(json_str.into())).await {
                tracing::error!("WebSocket send error: {:?}", e);
                break;
            }
        } else {
            tracing::warn!("shared_stats is None, skipping WebSocket push");
        }
    }
    tracing::info!("WebSocket connection closed.");
}
