use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Networks, System};
use tokio::sync::RwLock;

use crate::state::AppState;
use super::gpu::{self, GpuStats};

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
    pub gpus: Vec<GpuStats>,
    pub uptime: u64,
    pub hostname: String,
    pub sys_logs: Vec<String>,
}

pub struct SystemMonitor {
    sys: System,
    networks: Networks,
    last_net_check: Instant,
    state: Arc<RwLock<Option<SystemStats>>>,
}

impl SystemMonitor {
    pub fn new(shared_stats: Arc<RwLock<Option<SystemStats>>>) -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();

        let networks = Networks::new_with_refreshed_list();
        Self {
            sys,
            networks,
            last_net_check: Instant::now(),
            state: shared_stats,
        }
    }

    pub async fn run_loop(mut self, interval_secs: u64) {
        let interval = Duration::from_secs(interval_secs);
        loop {
            // Refresh CPU and Memory data
            self.sys.refresh_cpu();
            self.sys.refresh_memory();

            // Refresh Network data
            self.networks.refresh();

            // Calculate CPU global and per-core
            let cpu_global = self.sys.global_cpu_info().cpu_usage();
            let cpu_cores: Vec<f32> = self.sys.cpus().iter().map(|c| c.cpu_usage()).collect();

            // Calculate RAM usage
            let ram_used = self.sys.used_memory();
            let ram_total = self.sys.total_memory();

            // Calculate Network throughput (Bytes/sec)
            let now = Instant::now();
            let duration_secs = now.duration_since(self.last_net_check).as_secs_f32().max(0.1);
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

            // Fetch GPU stats
            let gpus = gpu::get_gpu_stats();

            // System Uptime (associated function in sysinfo)
            let uptime = System::uptime();

            // Fetch Hostname
            let hostname = System::host_name().unwrap_or_else(|| "localhost".to_string());

            // Fetch host system logs
            let sys_logs = get_system_logs();

            let stats = SystemStats {
                cpu_global,
                cpu_cores,
                ram_used,
                ram_total,
                net_in,
                net_out,
                disk_used,
                disk_total,
                gpus,
                uptime,
                hostname,
                sys_logs,
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

fn get_system_logs() -> Vec<String> {
    if let Ok(output) = std::process::Command::new("journalctl")
        .args(["-n", "30", "--no-pager"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return text.lines().map(|s| s.to_string()).collect();
        }
    }
    if let Ok(output) = std::process::Command::new("dmesg")
        .args(["-n", "30"])
        .output()
    {
        if output.status.success() {
            let text = String::from_utf8_lossy(&output.stdout);
            return text.lines().map(|s| s.to_string()).collect();
        }
    }
    vec!["[SYSTEM] Active logger. Monitoring dashboard...".to_string()]
}
