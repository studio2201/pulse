use gloo_timers::callback::Timeout;
use yew::prelude::*;

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
        } else {
            self.error_message = error;
            self.attempts_left = attempts_left;
            self.lockout_minutes = lockout_minutes;
        }
        true
    }

    pub fn handle_update_stats(&mut self, stats: SystemStats) -> bool {
        self.cpu_history.push(stats.cpu_global);
        if self.cpu_history.len() > 15 {
            self.cpu_history.remove(0);
        }

        let ram_percent =
            (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).clamp(0.0, 100.0);
        self.ram_history.push(ram_percent);
        if self.ram_history.len() > 15 {
            self.ram_history.remove(0);
        }

        let disk_percent =
            (stats.disk_used as f32 / stats.disk_total as f32 * 100.0).clamp(0.0, 100.0);
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
                format!("CPU Load High: {:.0}%", stats.cpu_global),
                "warning".to_string(),
            ));
        } else if ram_percent > 90.0 {
            warning = Some((
                format!("RAM Space Low: {:.0}%", ram_percent),
                "warning".to_string(),
            ));
        } else if disk_percent > 90.0 {
            warning = Some((
                format!("Disk Space Low: {:.0}%", disk_percent),
                "warning".to_string(),
            ));
        } else {
            for (idx, gpu) in stats.gpus.iter().enumerate() {
                if gpu.usage > 95.0 {
                    warning = Some((
                        format!("GPU {} Load High: {:.0}%", idx + 1, gpu.usage),
                        "warning".to_string(),
                    ));
                    break;
                }
                if let Some(temp) = gpu.temp {
                    if temp > 85.0 {
                        warning = Some((
                            format!("GPU {} Temp High: {:.0}°C", idx + 1, temp),
                            "warning".to_string(),
                        ));
                        break;
                    }
                }
            }
        }

        if let Some(warn) = warning {
            self.active_notification = Some(warn);
        } else if self
            .active_notification
            .as_ref()
            .map(|(_, cls)| cls == "warning" || cls == "success")
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
