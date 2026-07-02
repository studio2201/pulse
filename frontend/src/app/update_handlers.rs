use gloo_timers::callback::Timeout;
use yew::prelude::*;
use shared_frontend::i18n::strings::{lookup, StringKey};

use crate::app::App;
use crate::app::Msg;
use crate::types::SystemStats;

impl App {
    pub fn handle_pin_response(
        &mut self,
        ctx: &Context<Self>,
        success: bool,
        error: Option<String>,
        attempts_left: Option<usize>,
        lockout_minutes: Option<u64>,
     ) -> bool {
        self.is_authenticated = success;
        self.pin_input.clear();
        if success {
            self.error_message = None;
            self.attempts_left = None;
            self.lockout_minutes = None;
            self.connect_ws(ctx);
            self.show_notification(ctx, lookup(StringKey::StatusPinSuccess, self.language).to_string(), "success".to_string());
        } else {
            self.error_message = error.clone();
            self.attempts_left = attempts_left;
            self.lockout_minutes = lockout_minutes;
            if error.is_some() || attempts_left.is_some() || lockout_minutes.is_some() {
                self.show_notification(ctx, lookup(StringKey::StatusPinFailure, self.language).to_string(), "error".to_string());
            } else {
                self.show_notification(ctx, lookup(StringKey::StatusLogout, self.language).to_string(), "success".to_string());
            }
        }
        true
    }

    pub fn handle_update_stats(&mut self, stats: SystemStats) -> bool {
        self.cpu_history.push(stats.cpu_global);
        if self.cpu_history.len() > 15 {
            self.cpu_history.remove(0);
        }

        let ram_percent = if stats.ram_total > 0 {
            (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        self.ram_history.push(ram_percent);
        if self.ram_history.len() > 15 {
            self.ram_history.remove(0);
        }

        let disk_percent = if stats.disk_total > 0 {
            (stats.disk_used as f32 / stats.disk_total as f32 * 100.0).clamp(0.0, 100.0)
        } else {
            0.0
        };
        self.disk_history.push(disk_percent);
        if self.disk_history.len() > 15 {
            self.disk_history.remove(0);
        }

        let net_total = (stats.net_in + stats.net_out) as f32;
        self.net_history.push(net_total);
        if self.net_history.len() > 15 {
            self.net_history.remove(0);
        }

        while self.gpu_histories.len() < stats.gpus.len() {
            self.gpu_histories.push(Vec::new());
        }
        while self.gpu_histories.len() > stats.gpus.len() {
            self.gpu_histories.pop();
        }
        for (idx, gpu) in stats.gpus.iter().enumerate() {
            self.gpu_histories[idx].push(gpu.usage);
            if self.gpu_histories[idx].len() > 15 {
                self.gpu_histories[idx].remove(0);
            }
        }

        let mut warning = None;
        if stats.cpu_global > 95.0 {
            warning = Some((
                crate::i18n::lookup(crate::i18n::PulseKey::CpuLoadHigh(stats.cpu_global), self.language),
                "warning".to_string(),
            ));
        } else if stats.cpu_temp.unwrap_or(0.0) > 80.0 {
            warning = Some((
                crate::i18n::lookup(crate::i18n::PulseKey::CpuTempHigh(stats.cpu_temp.unwrap()), self.language),
                "warning".to_string(),
            ));
        } else if ram_percent > 90.0 {
            warning = Some((
                crate::i18n::lookup(crate::i18n::PulseKey::RamSpaceLow(ram_percent), self.language),
                "warning".to_string(),
            ));
        } else if disk_percent > 90.0 {
            warning = Some((
                crate::i18n::lookup(crate::i18n::PulseKey::DiskSpaceLow(disk_percent), self.language),
                "warning".to_string(),
            ));
        } else {
            let mut gpu_warning = None;
            for (idx, gpu) in stats.gpus.iter().enumerate() {
                if gpu.usage > 95.0 {
                    gpu_warning = Some((
                        crate::i18n::lookup(crate::i18n::PulseKey::GpuLoadHigh(idx + 1, gpu.usage), self.language),
                        "warning".to_string(),
                    ));
                    break;
                }
                if let Some(temp) = gpu.temp {
                    if temp > 85.0 {
                        gpu_warning = Some((
                            crate::i18n::lookup(crate::i18n::PulseKey::GpuTempHigh(idx + 1, temp), self.language),
                            "warning".to_string(),
                        ));
                        break;
                    }
                }
            }

            if let Some(gw) = gpu_warning {
                warning = Some(gw);
            } else if net_total > 52_428_800.0 { // 50 MB/s
                warning = Some((
                    crate::i18n::lookup(crate::i18n::PulseKey::HighNetworkTraffic(self.format_bytes(stats.net_in + stats.net_out)), self.language),
                    "warning".to_string(),
                ));
            } else if stats.uptime < 300 {
                let minutes = stats.uptime / 60;
                let secs = stats.uptime % 60;
                let uptime_str = format!("{}m {}s", minutes, secs);
                warning = Some((
                    crate::i18n::lookup(crate::i18n::PulseKey::SystemRecentlyRebooted(uptime_str), self.language),
                    "info".to_string(),
                ));
            }
        }

        if let Some(warn) = warning {
            self.active_notification = Some(warn);
        } else if self
            .active_notification
            .as_ref()
            .map(|(_, cls)| cls == "warning" || cls == "success" || cls == "error" || cls == "info")
            .unwrap_or(true)
        {
            self.active_notification = None;
        }

        self.stats = Some(stats);
        true
    }

    pub fn notify(&mut self, ctx: &Context<Self>, msg: String) {
        self.active_notification = Some((msg.clone(), "info".to_string()));
        let link = ctx.link().clone();
        Timeout::new(3000, move || {
            link.send_message(Msg::ClearNotification(msg));
        })
        .forget();
    }
}
