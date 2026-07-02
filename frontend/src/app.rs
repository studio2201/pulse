use gloo_net::http::Request;
use serde_json::Value;
use web_sys::WebSocket;
use yew::prelude::*;

use crate::storage::StorageService;
use crate::types::SystemStats;
use shared_frontend::{Footer, Header, i18n::Language};

mod login;
mod update;
mod update_handlers;
mod view;
mod view_cards;
mod view_sys_info;
mod view_sys_info_art;
mod ws;

pub enum Msg {
    LoadConfig(Value),
    PinInputChanged(String),
    SubmitPin,
    PinResponse(bool, Option<String>, Option<usize>, Option<u64>),
    Logout,
    UpdateStats(SystemStats),
    WsError(String),
    WsLog(String),
    Reconnect,
    ToggleTheme,
    ChangeLanguage(Language),
    CheckFallback,
    ClearNotification(String),
    CycleOsOverride,
    Print,
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
    pub attempts_left: Option<usize>,
    pub lockout_minutes: Option<u64>,
    pub stats: Option<SystemStats>,
    pub ws: Option<WebSocket>,
    pub enable_translation: bool,
    pub enable_themes: bool,
    pub enable_print: bool,
    pub cpu_history: Vec<f32>,
    pub ram_history: Vec<f32>,
    pub disk_history: Vec<f32>,
    pub net_history: Vec<f32>,
    pub gpu_histories: Vec<Vec<f32>>,
    pub active_notification: Option<(String, String)>,
    pub monitor_cpu: bool,
    pub monitor_memory: bool,
    pub monitor_storage: bool,
    pub monitor_network: bool,
    pub monitor_gpu: bool,
    pub os_override: Option<usize>,
    pub enable_coffee: bool,
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let language = Language::from_code(&StorageService::get_item("language", "en"));
        let theme = StorageService::get_item("theme", "crateria");

        let link = ctx.link().clone();
        wasm_bindgen_futures::spawn_local(async move {
            match Request::get("/config").send().await {
                Ok(resp) => match resp.json::<Value>().await {
                    Ok(json) => {
                        link.send_message(Msg::LoadConfig(json));
                    }
                    Err(err) => {
                        link.send_message(Msg::WsError(format!(
                            "[ERROR] Failed to parse config JSON: {:?}",
                            err
                        )));
                    }
                },
                Err(err) => {
                    link.send_message(Msg::WsError(format!(
                        "[ERROR] Failed to fetch /config: {:?}",
                        err
                    )));
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
            attempts_left: None,
            lockout_minutes: None,
            stats: None,
            ws: None,
            enable_translation: false,
            enable_themes: true,
            enable_print: false,
            cpu_history: Vec::new(),
            ram_history: Vec::new(),
            disk_history: Vec::new(),
            net_history: Vec::new(),
            gpu_histories: Vec::new(),
            active_notification: None,
            monitor_cpu: true,
            monitor_memory: true,
            monitor_storage: true,
            monitor_network: true,
            monitor_gpu: true,
            os_override: None,
            enable_coffee: true,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        self.update_app(ctx, msg)
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let link = ctx.link().clone();
            wasm_bindgen_futures::spawn_local(async move {
                loop {
                    gloo_timers::future::sleep(std::time::Duration::from_secs(2)).await;
                    link.send_message(Msg::CheckFallback);
                }
            });

            use wasm_bindgen::JsCast;
            let link_key = ctx.link().clone();
            let closure = wasm_bindgen::prelude::Closure::wrap(Box::new(
                move |event: web_sys::KeyboardEvent| {
                    if event.key() == "i" || event.key() == "I" {
                        link_key.send_message(Msg::CycleOsOverride);
                    }
                },
            )
                as Box<dyn FnMut(web_sys::KeyboardEvent)>);

            let window = web_sys::window().expect("no global `window` exists");
            window
                .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
                .unwrap();
            closure.forget();
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.site_title.clone();
        let toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);
        let on_language_change = ctx.link().callback(Msg::ChangeLanguage);
        let on_logout = ctx.link().callback(|_| Msg::Logout);

        let on_print = Some(ctx.link().callback(|_| Msg::Print));

        html! {
            <>
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
                    print_disabled={self.pin_required && !self.is_authenticated}
                    on_print={on_print}
                    version={Some(env!("CARGO_PKG_VERSION").to_string())}
                />



                <div class="app-body">
                    {if !self.is_authenticated && self.pin_required {
                        self.view_login(ctx)
                    } else {
                        self.view_hud(ctx)
                    }}
                </div>

                <Footer
                    version={env!("CARGO_PKG_VERSION").to_string()}
                    show_coffee={self.enable_coffee}
                >
                    {
                        if let Some((msg, cls)) = &self.active_notification {
                            html! { <div class={format!("footer-status-text {}", cls)}>{ msg }</div> }
                        } else {
                            html! { <div class="footer-status-text success">{crate::i18n::lookup(crate::i18n::PulseKey::Ready, self.language)}</div> }
                        }
                    }
                </Footer>
            </>
        }
    }
}

impl App {
    pub fn show_notification(&mut self, ctx: &Context<Self>, msg: String, cls: String) {
        self.active_notification = Some((msg.clone(), cls));
        let link = ctx.link().clone();
        let clear_msg = msg.clone();
        gloo_timers::callback::Timeout::new(3000, move || {
            link.send_message(Msg::ClearNotification(clear_msg));
        })
        .forget();
    }
}
