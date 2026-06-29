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
                                link.send_message(Msg::PinResponse(true, None));
                            }
                        }
                    });
                }
                true
            }
            Msg::PinInputChanged(val) => {
                self.pin_input = val;
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
                    if let Ok(resp) = Request::post("/api/verify-pin").json(&payload).unwrap().send().await {
                        if resp.status() == 200 {
                            link.send_message(Msg::PinResponse(true, None));
                        } else if let Ok(json) = resp.json::<Value>().await {
                            let err = json["error"].as_str().unwrap_or("Verification failed").to_string();
                            link.send_message(Msg::PinResponse(false, Some(err)));
                        }
                    }
                });
                true
            }
            Msg::PinResponse(success, error) => {
                self.is_authenticated = success;
                self.pin_input.clear();
                if success {
                    self.error_message = None;
                    self.connect_ws(ctx);
                    self.terminal_logs.push("[AUTH] Security clearance granted.".to_string());
                } else {
                    self.error_message = error;
                }
                true
            }
            Msg::Logout => {
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let _ = Request::post("/api/logout").send().await;
                    link.send_message(Msg::PinResponse(false, None));
                });
                if let Some(ws) = self.ws.take() {
                    let _ = ws.close();
                }
                self.stats = None;
                true
            }
            Msg::UpdateStats(stats) => {
                self.terminal_logs = stats.sys_logs.clone();

                // Update history vectors
                self.cpu_history.push(stats.cpu_global);
                if self.cpu_history.len() > 15 { self.cpu_history.remove(0); }

                let ram_percent = (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).min(100.0).max(0.0);
                self.ram_history.push(ram_percent);
                if self.ram_history.len() > 15 { self.ram_history.remove(0); }

                let disk_percent = (stats.disk_used as f32 / stats.disk_total as f32 * 100.0).min(100.0).max(0.0);
                self.disk_history.push(disk_percent);
                if self.disk_history.len() > 15 { self.disk_history.remove(0); }

                let net_total = (stats.net_in + stats.net_out) as f32;
                self.net_history.push(net_total);
                if self.net_history.len() > 15 { self.net_history.remove(0); }

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

                // Check alert thresholds for built-in footer status
                let mut warning = None;
                if stats.cpu_global > 95.0 {
                    warning = Some((format!("CPU Load High: {:.0}%", stats.cpu_global), "warning".to_string()));
                } else if ram_percent > 90.0 {
                    warning = Some((format!("RAM Space Low: {:.0}%", ram_percent), "warning".to_string()));
                } else if disk_percent > 90.0 {
                    warning = Some((format!("Disk Space Low: {:.0}%", disk_percent), "warning".to_string()));
                } else {
                    for (idx, gpu) in stats.gpus.iter().enumerate() {
                        if gpu.usage > 95.0 {
                            warning = Some((format!("GPU {} Load High: {:.0}%", idx + 1, gpu.usage), "warning".to_string()));
                            break;
                        }
                        if let Some(temp) = gpu.temp {
                            if temp > 85.0 {
                                warning = Some((format!("GPU {} Temp High: {:.0}°C", idx + 1, temp), "warning".to_string()));
                                break;
                            }
                        }
                    }
                }

                if let Some(warn) = warning {
                    self.active_notification = Some(warn);
                } else if self.active_notification.as_ref().map(|(_, cls)| cls == "warning" || cls == "success").unwrap_or(true) {
                    // Revert to default Ready status if warnings cleared
                    self.active_notification = None;
                }

                self.stats = Some(stats);
                true
            }
            Msg::WsError(err) => {
                self.terminal_logs.push(format!("[WS ERROR] {}", err));
                self.active_notification = Some(("Disconnected".to_string(), "error".to_string()));
                self.ws = None; // Reset so reconnect attempts are allowed
                let link = ctx.link().clone();
                Timeout::new(5000, move || {
                    link.send_message(Msg::Reconnect);
                }).forget();
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
                let current = shared_frontend::theme::Theme::from_name(&self.theme).unwrap_or_default();
                let idx = shared_frontend::theme::Theme::ALL.iter().position(|&t| t == current).unwrap_or(0);
                let next = shared_frontend::theme::Theme::ALL[(idx + 1) % shared_frontend::theme::Theme::ALL.len()];
                self.theme = next.name().to_string();
                StorageService::set_item("theme", &self.theme);

                if let Some(window) = web_sys::window() {
                    let doc = window.document().unwrap();
                    doc.document_element().unwrap().set_class_name(&self.theme);
                    doc.document_element().unwrap().set_attribute("data-theme", &self.theme).unwrap();
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
                self.terminal_logs.push("[SYSTEM] Terminal buffer cleared.".to_string());
                true
            }
            Msg::IncreaseFontSize => {
                self.console_font_size = (self.console_font_size + 0.05).min(1.5);
                true
            }
            Msg::DecreaseFontSize => {
                self.console_font_size = (self.console_font_size - 0.05).max(0.65);
                true
            }
            Msg::CheckFallback => {
                let ws_connected = self.ws.as_ref().map(|w| w.ready_state() == 1).unwrap_or(false);
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
}
