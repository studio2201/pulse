use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::config::AppConfig;
use crate::services::monitor::SystemStats;

#[derive(Clone)]
pub struct AppState {
    pub config: AppConfig,
    pub active_sessions: Arc<RwLock<HashSet<String>>>,
    pub rate_limiter: Arc<RwLock<HashMap<IpAddr, Vec<Instant>>>>,
    pub shared_stats: Arc<RwLock<Option<SystemStats>>>,
}

impl AppState {
    pub fn new(config: AppConfig, shared_stats: Arc<RwLock<Option<SystemStats>>>) -> Self {
        Self {
            config,
            active_sessions: Arc::new(RwLock::new(HashSet::new())),
            rate_limiter: Arc::new(RwLock::new(HashMap::new())),
            shared_stats,
        }
    }

    pub async fn update_stats(&self, stats: SystemStats) {
        let mut stats_lock = self.shared_stats.write().await;
        *stats_lock = Some(stats);
    }

    pub async fn check_rate_limit(
        &self,
        ip: IpAddr,
        max_requests: usize,
        window: Duration,
    ) -> bool {
        let now = Instant::now();
        let mut map = self.rate_limiter.write().await;
        let timestamps = map.entry(ip).or_insert_with(Vec::new);

        timestamps.retain(|&t| now.duration_since(t) < window);

        if timestamps.len() >= max_requests {
            false
        } else {
            timestamps.push(now);
            true
        }
    }

    pub async fn clean_old_rate_limits(&self, window: Duration) {
        let now = Instant::now();
        let mut map = self.rate_limiter.write().await;
        map.retain(|_, timestamps| {
            timestamps.retain(|&t| now.duration_since(t) < window);
            !timestamps.is_empty()
        });
    }
}
