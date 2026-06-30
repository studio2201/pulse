use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use serde_json::Value;
use yew::prelude::*;

use crate::app::App;
use crate::app::Msg;
use crate::storage::StorageService;
use crate::types::SystemStats;

#[derive(serde::Serialize)]
struct VerifyPinPayload {
    pin: Option<String>,
}

impl App {
    pub fn update_app(&mut self, ctx: &Context<Self>, msg: Msg) -> bool {
        match msg {
            Msg::LoadConfig(json) => {
                self.site_title = json["siteTitle"].as_str().unwrap_or("Pulse").to_string();
                self.pin_required = json["pinRequired"].as_bool().unwrap_or(false);
                self.pin_length = json["pinLength"].as_u64().unwrap_or(0) as usize;
                self.enable_translation = json["enableTranslation"].as_bool().unwrap_or(false);
                self.enable_themes = json["enableThemes"].as_bool().unwrap_or(true);
                self.enable_print = json["enablePrint"].as_bool().unwrap_or(false);
                self.monitor_cpu = json["monitorCpu"].as_bool().unwrap_or(true);
                self.monitor_memory = json["monitorMemory"].as_bool().unwrap_or(true);
                self.monitor_storage = json["monitorStorage"].as_bool().unwrap_or(true);
                self.monitor_network = json["monitorNetwork"].as_bool().unwrap_or(true);
                self.monitor_gpu = json["monitorGpu"].as_bool().unwrap_or(true);
                self.monitor_console = json["monitorConsole"].as_bool().unwrap_or(true);

                self.terminal_logs.push(format!(
                    "[SYSTEM] Config loaded. Site: {}, Pin Required: {}",
                    self.site_title, self.pin_required
                ));

                if !self.pin_required {
                    self.is_authenticated = true;
                    self.connect_ws(ctx);
                } else {
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(resp) = Request::get("/api/auth-check").send().await {
                            if resp.status() == 200 {
                                link.send_message(Msg::PinResponse(true, None, None, None));
                            }
                        }
                    });
                }
                true
            }
            Msg::PinInputChanged(val) => {
                let filtered: String = val.chars().filter(|c| c.is_ascii_digit()).collect();
                self.pin_input = filtered;
                if self.pin_input.len() == self.pin_length {
                    ctx.link().send_message(Msg::SubmitPin);
                }
                true
            }
            Msg::SubmitPin => {
                let pin = self.pin_input.clone();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let payload = VerifyPinPayload { pin: Some(pin) };
                    if let Ok(resp) = Request::post("/api/verify-pin")
                        .json(&payload)
                        .unwrap()
                        .send()
                        .await
                    {
                        if resp.status() == 200 {
                            link.send_message(Msg::PinResponse(true, None, None, None));
                        } else if let Ok(json) = resp.json::<Value>().await {
                            let err = json["error"]
                                .as_str()
                                .unwrap_or("Verification failed")
                                .to_string();
                            let attempts = json["attemptsLeft"].as_u64().map(|v| v as usize);
                            let lockout = json["lockoutMinutes"].as_u64().or_else(|| {
                                if err.contains("Please try again in") {
                                    err.split("Please try again in ")
                                        .nth(1)
                                        .and_then(|s| s.split(' ').next())
                                        .and_then(|s| s.parse::<u64>().ok())
                                } else {
                                    None
                                }
                            });
                            link.send_message(Msg::PinResponse(
                                false,
                                Some(err),
                                attempts,
                                lockout,
                            ));
                        }
                    }
                });
                true
            }
            Msg::PinResponse(success, error, attempts_left, lockout_minutes) => {
                self.handle_pin_response(ctx, success, error, attempts_left, lockout_minutes)
            }
            Msg::Logout => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("/api/logout").send().await;
                    link.send_message(Msg::PinResponse(false, None, None, None));
                });
                if let Some(ws) = self.ws.take() {
                    let _ = ws.close();
                }
                self.stats = None;
                true
            }
            Msg::UpdateStats(stats) => self.handle_update_stats(stats),
            Msg::WsError(err) => {
                self.terminal_logs.push(format!("[WS ERROR] {}", err));
                self.active_notification = Some(("Disconnected".to_string(), "error".to_string()));
                self.ws = None;
                let link = ctx.link().clone();
                Timeout::new(5000, move || {
                    link.send_message(Msg::Reconnect);
                })
                .forget();
                true
            }
            Msg::WsLog(msg) => {
                self.terminal_logs.push(msg);
                true
            }
            Msg::Reconnect => {
                if self.is_authenticated {
                    self.connect_ws(ctx);
                }
                true
            }
            Msg::ToggleTheme => {
                let current =
                    shared_frontend::theme::Theme::from_name(&self.theme).unwrap_or_default();
                let idx = shared_frontend::theme::Theme::ALL
                    .iter()
                    .position(|&t| t == current)
                    .unwrap_or(0);
                let next = shared_frontend::theme::Theme::ALL
                    [(idx + 1) % shared_frontend::theme::Theme::ALL.len()];
                self.theme = next.name().to_string();
                StorageService::set_item("theme", &self.theme);

                if let Some(window) = web_sys::window() {
                    let doc = window.document().unwrap();
                    doc.document_element().unwrap().set_class_name(&self.theme);
                    doc.document_element()
                        .unwrap()
                        .set_attribute("data-theme", &self.theme)
                        .unwrap();
                }
                true
            }
            Msg::ChangeLanguage(lang) => {
                self.language = lang;
                StorageService::set_item("language", lang.code());
                true
            }
            Msg::ClearTerminal => {
                self.terminal_logs.clear();
                self.terminal_logs
                    .push("[SYSTEM] Terminal buffer cleared.".to_string());
                self.notify(ctx, "Console logs cleared".to_string());
                true
            }
            Msg::IncreaseFontSize => {
                self.console_font_size = (self.console_font_size + 0.05).min(1.5);
                self.notify(
                    ctx,
                    format!("Font size increased to {:.2}rem", self.console_font_size),
                );
                true
            }
            Msg::DecreaseFontSize => {
                self.console_font_size = (self.console_font_size - 0.05).max(0.65);
                self.notify(
                    ctx,
                    format!("Font size decreased to {:.2}rem", self.console_font_size),
                );
                true
            }
            Msg::TogglePauseConsole => {
                self.console_paused = !self.console_paused;
                let text = if self.console_paused {
                    "Console scrolling paused"
                } else {
                    "Console scrolling resumed"
                };
                self.notify(ctx, text.to_string());
                true
            }
            Msg::ClearNotification(msg_to_clear) => {
                if let Some((current_msg, _)) = &self.active_notification {
                    if current_msg == &msg_to_clear {
                        self.active_notification = None;
                    }
                }
                true
            }
            Msg::ConsoleMouseUp => {
                if shared_frontend::utils::copy_selection_to_clipboard().is_some() {
                    self.notify(ctx, "Copied selection to clipboard".to_string());
                }
                true
            }
            Msg::CycleOsOverride => {
                let next_idx = match self.os_override {
                    None => Some(0),
                    Some(idx) => {
                        if idx >= 6 {
                            None
                        } else {
                            Some(idx + 1)
                        }
                    }
                };
                self.os_override = next_idx;
                let notify_text = match next_idx {
                    None => "OS Logo: Auto-Detect".to_string(),
                    Some(0) => "OS Logo: NixOS".to_string(),
                    Some(1) => "OS Logo: Ubuntu".to_string(),
                    Some(2) => "OS Logo: Debian".to_string(),
                    Some(3) => "OS Logo: Arch Linux".to_string(),
                    Some(4) => "OS Logo: Fedora".to_string(),
                    Some(5) => "OS Logo: Pop!_OS".to_string(),
                    Some(6) => "OS Logo: Fallback/Tux".to_string(),
                    _ => "OS Logo: Custom".to_string(),
                };
                self.notify(ctx, notify_text);
                true
            }
            Msg::CheckFallback => {
                let ws_connected = self
                    .ws
                    .as_ref()
                    .map(|w| w.ready_state() == 1)
                    .unwrap_or(false);
                if !ws_connected && self.is_authenticated {
                    let link = ctx.link().clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(resp) = Request::get("/api/stats").send().await {
                            if let Ok(stats) = resp.json::<SystemStats>().await {
                                link.send_message(Msg::UpdateStats(stats));
                            }
                        }
                    });
                }
                false
            }
        }
    }

    fn handle_pin_response(
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
            self.terminal_logs
                .push("[AUTH] Security clearance granted.".to_string());
        } else {
            self.error_message = error;
            self.attempts_left = attempts_left;
            self.lockout_minutes = lockout_minutes;
        }
        true
    }

    fn handle_update_stats(&mut self, stats: SystemStats) -> bool {
        self.terminal_logs = stats.sys_logs.clone();

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

    fn notify(&mut self, ctx: &Context<Self>, msg: String) {
        self.active_notification = Some((msg.clone(), "info".to_string()));
        let link = ctx.link().clone();
        Timeout::new(3000, move || {
            link.send_message(Msg::ClearNotification(msg));
        })
        .forget();
    }
}
