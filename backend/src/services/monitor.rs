use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Networks, System};
use tokio::sync::RwLock;

use super::gpu::{self, GpuStats};
use crate::state::AppState;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStats {
    pub cpu_global: f32,
    pub cpu_cores: Vec<f32>,
    pub ram_used: u64,
    pub ram_total: u64,
    pub net_in: u64,
    pub net_out: u64,
    pub disk_used: u64,
    pub disk_total: u64,
    pub cpu_brand: String,
    pub gpus: Vec<GpuStats>,
    pub uptime: u64,
    pub hostname: String,
    pub os_name: String,
    pub os_version: String,
    pub kernel_version: String,
    pub cpu_temp: Option<f32>,
}

pub struct SystemMonitor {
    sys: System,
    networks: Networks,
    last_net_check: Instant,
    state: Arc<RwLock<Option<SystemStats>>>,
    cpu_brand: String,
    hostname: String,
    os_name: String,
    os_version: String,
    kernel_version: String,
}

impl SystemMonitor {
    pub fn new(shared_stats: Arc<RwLock<Option<SystemStats>>>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        let cpu_brand = sys
            .cpus()
            .first()
            .map(|c| c.brand().trim().to_string())
            .unwrap_or_else(|| "Unknown CPU".to_string());
        let hostname = std::env::var("PULSE_HOSTNAME")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::fs::read_to_string("/etc/host_hostname")
                    .ok()
                    .map(|s| s.trim().to_string())
            })
            .or_else(System::host_name)
            .unwrap_or_else(|| "localhost".to_string());
        let os_name = std::env::var("PULSE_OS")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::fs::read_to_string("/etc/host_os-release")
                    .ok()
                    .and_then(|content| {
                        content
                            .lines()
                            .find(|line| line.starts_with("PRETTY_NAME="))
                            .or_else(|| content.lines().find(|line| line.starts_with("NAME=")))
                            .map(|line| {
                                line.split('=')
                                    .nth(1)
                                    .unwrap_or("")
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string()
                            })
                    })
            })
            .or_else(System::name)
            .unwrap_or_else(|| "Linux".to_string());
        let os_version = std::env::var("PULSE_OS_VERSION")
            .ok()
            .filter(|s| !s.is_empty())
            .or_else(|| {
                std::fs::read_to_string("/etc/host_os-release")
                    .ok()
                    .and_then(|content| {
                        content
                            .lines()
                            .find(|line| line.starts_with("VERSION_ID="))
                            .or_else(|| content.lines().find(|line| line.starts_with("VERSION=")))
                            .map(|line| {
                                line.split('=')
                                    .nth(1)
                                    .unwrap_or("")
                                    .trim_matches('"')
                                    .trim_matches('\'')
                                    .to_string()
                            })
                    })
            })
            .or_else(System::os_version)
            .unwrap_or_default();
        let kernel_version = System::kernel_version().unwrap_or_default();

        Self {
            sys,
            networks,
            last_net_check: Instant::now(),
            state: shared_stats,
            cpu_brand,
            hostname,
            os_name,
            os_version,
            kernel_version,
        }
    }

    pub async fn run_loop(mut self, interval_secs: u64) {
        let interval = Duration::from_secs(interval_secs);
        loop {
            // Refresh CPU and Memory data
            self.sys.refresh_cpu_all();
            self.sys.refresh_memory();

            // Refresh Network data
            self.networks.refresh(true);

            // Calculate CPU global and per-core
            let cpu_global = self.sys.global_cpu_usage();
            let cpu_cores: Vec<f32> = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();

            // Calculate RAM usage
            let ram_used = self.sys.used_memory();
            let ram_total = self.sys.total_memory();

            // Calculate Network throughput (Bytes/sec)
            let now = Instant::now();
            let duration_secs = now
                .duration_since(self.last_net_check)
                .as_secs_f32()
                .max(0.1);
            self.last_net_check = now;

            let mut total_in = 0;
            let mut total_out = 0;
            for (_interface_name, data) in &self.networks {
                total_in += data.received();
                total_out += data.transmitted();
            }

            let net_in = (total_in as f32 / duration_secs) as u64;
            let net_out = (total_out as f32 / duration_secs) as u64;

            // Calculate Disk usage
            let disks = sysinfo::Disks::new_with_refreshed_list();
            let mut total_space = 0;
            let mut total_available = 0;
            for disk in &disks {
                total_space += disk.total_space();
                total_available += disk.available_space();
            }
            let disk_total = total_space;
            let disk_used = total_space - total_available;

            // Use cached static properties
            let cpu_brand = self.cpu_brand.clone();
            let hostname = self.hostname.clone();
            let os_name = self.os_name.clone();
            let os_version = self.os_version.clone();
            let kernel_version = self.kernel_version.clone();

            // Fetch GPU stats
            let gpus = gpu::get_gpu_stats();

            // System Uptime (associated function in sysinfo)
            let uptime = System::uptime();

            // Fetch CPU temperature
            let components = sysinfo::Components::new_with_refreshed_list();
            let cpu_temp = components
                .iter()
                .filter(|c| {
                    let label = c.label().to_lowercase();
                    label.contains("cpu")
                        || label.contains("core")
                        || label.contains("package")
                        || label.contains("temp")
                })
                .filter_map(|c| c.temperature())
                .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            let stats = SystemStats {
                cpu_global,
                cpu_cores,
                ram_used,
                ram_total,
                net_in,
                net_out,
                disk_used,
                disk_total,
                cpu_brand,
                gpus,
                uptime,
                hostname,
                os_name,
                os_version,
                kernel_version,
                cpu_temp,
            };

            // Update shared state
            {
                let mut state_lock = self.state.write().await;
                *state_lock = Some(stats);
            }

            tokio::time::sleep(interval).await;
        }
    }
}

pub fn start_monitor(state: AppState) {
    let interval = state.config.refresh_interval;
    let shared_stats = state.shared_stats.clone();
    tokio::spawn(async move {
        let monitor = SystemMonitor::new(shared_stats);
        monitor.run_loop(interval).await;
    });
}
