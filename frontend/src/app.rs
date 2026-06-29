use gloo_net::http::Request;
use gloo_timers::callback::Timeout;
use serde_json::Value;
use web_sys::WebSocket;
use yew::prelude::*;

use crate::storage::StorageService;
use crate::types::SystemStats;
use shared_frontend::{Footer, Header, i18n::Language};

mod view;
mod ws;

#[derive(serde::Serialize)]
struct VerifyPinPayload {
    pin: Option<String>,
}

pub enum Msg {
    LoadConfig(Value),
    PinInput(String),
    PinBackspace,
    SubmitPin,
    PinResponse(bool, Option<String>),
    Logout,
    UpdateStats(SystemStats),
    WsError(String),
    ToggleTheme,
    ChangeLanguage(Language),
    ToggleTerminal,
    ClearTerminal,
}

pub struct App {
    pub site_title: String,
    pub theme: String,
    pub language: Language,
    pub pin_required: bool,
    pub pin_length: usize,
    pub is_authenticated: bool,
    pub pin_input: String,
    pub error_message: Option<String>,
    pub stats: Option<SystemStats>,
    pub ws: Option<WebSocket>,
    pub terminal_logs: Vec<String>,
    pub terminal_open: bool,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let language = Language::from_code(&StorageService::get_item("language", "en"));
        let theme = StorageService::get_item("theme", "crateria");

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(resp) = Request::get("/config").send().await {
                if let Ok(json) = resp.json::<Value>().await {
                    link.send_message(Msg::LoadConfig(json));
                }
            }
        });

        Self {
            site_title: "Pulse".to_string(),
            theme,
            language,
            pin_required: false,
            pin_length: 0,
            is_authenticated: false,
            pin_input: String::new(),
            error_message: None,
            stats: None,
            ws: None,
            terminal_logs: vec!["[SYSTEM] Initializing Samus Visor HUD...".to_string()],
            terminal_open: true,
            enable_translation: false,
            enable_themes: true,
            enable_print: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadConfig(json) => {
                self.site_title = json["siteTitle"].as_str().unwrap_or("Pulse").to_string();
                self.pin_required = json["pinRequired"].as_bool().unwrap_or(false);
                self.pin_length = json["pinLength"].as_u64().unwrap_or(0) as usize;
                self.enable_translation = json["enableTranslation"].as_bool().unwrap_or(false);
                self.enable_themes = json["enableThemes"].as_bool().unwrap_or(true);
                self.enable_print = json["enablePrint"].as_bool().unwrap_or(false);

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
            Msg::PinInput(digit) => {
                if self.pin_input.len() < self.pin_length {
                    self.pin_input.push_str(&digit);
                }
                if self.pin_input.len() == self.pin_length {
                    ctx.link().send_message(Msg::SubmitPin);
                }
                true
            }
            Msg::PinBackspace => {
                self.pin_input.pop();
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
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                let gpu_text = stats.gpu.as_ref().map_or("None".to_string(), |g| {
                    format!("{} ({}% / {}°C)", g.name, g.usage, g.temp.unwrap_or(0.0))
                });
                let log = format!(
                    "[{}] CPU: {:.1}%, RAM: {:.1} GB, GPU: {}, Net In: {:.2} MB/s, Net Out: {:.2} MB/s",
                    timestamp,
                    stats.cpu_global,
                    (stats.ram_used as f64 / 1024.0 / 1024.0 / 1024.0),
                    gpu_text,
                    (stats.net_in as f64 / 1024.0 / 1024.0),
                    (stats.net_out as f64 / 1024.0 / 1024.0)
                );
                self.terminal_logs.push(log);
                if self.terminal_logs.len() > 100 {
                    self.terminal_logs.remove(0);
                }
                self.stats = Some(stats);
                true
            }
            Msg::WsError(err) => {
                self.terminal_logs.push(format!("[WS ERROR] {}", err));
                let link = ctx.link().clone();
                Timeout::new(5000, move || {
                    link.send_message(Msg::SubmitPin);
                }).forget();
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
            Msg::ToggleTerminal => {
                self.terminal_open = !self.terminal_open;
                true
            }
            Msg::ClearTerminal => {
                self.terminal_logs.clear();
                self.terminal_logs.push("[SYSTEM] Terminal buffer cleared.".to_string());
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.site_title.clone();
        let toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);
        let on_language_change = ctx.link().callback(Msg::ChangeLanguage);
        let on_logout = ctx.link().callback(|_| Msg::Logout);

        html! {
            <div class="layout-wrapper">
                <Header
                    site_title={site_title}
                    theme={self.theme.clone()}
                    language={self.language}
                    toggle_theme={toggle_theme}
                    on_language_change={on_language_change}
                    is_authenticated={self.is_authenticated}
                    pin_required={self.pin_required}
                    on_logout={on_logout}
                    enable_translation={self.enable_translation}
                    enable_themes={self.enable_themes}
                    enable_print={self.enable_print}
                    print_disabled={true}
                    on_print={None::<Callback<MouseEvent>>}
                />

                <main class="layout-content">
                    {if !self.is_authenticated && self.pin_required {
                        self.view_login(ctx)
                    } else {
                        self.view_hud(ctx)
                    }}
                </main>

                <Footer
                    show_version={true}
                    version={"3.0.0".to_string()}
                    show_github={true}
                    github_url={Some("https://github.com/UberMetroid/pulse".to_string())}
                />
            </div>
        }
    }
}
