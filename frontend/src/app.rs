use gloo_net::http::Request;
use serde_json::Value;
use web_sys::WebSocket;
use yew::prelude::*;

use crate::storage::StorageService;
use crate::types::SystemStats;
use shared_frontend::{Footer, Header, i18n::Language};

mod update;
mod view;
mod ws;

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
    CheckFallback,
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
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let site_title = self.site_title.clone();
        let toggle_theme = ctx.link().callback(|_| Msg::ToggleTheme);
        let on_language_change = ctx.link().callback(Msg::ChangeLanguage);
        let on_logout = ctx.link().callback(|_| Msg::Logout);

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
                    print_disabled={true}
                    on_print={None::<Callback<MouseEvent>>}
                />

                <div class="app-body">
                    {if !self.is_authenticated && self.pin_required {
                        self.view_login(ctx)
                    } else {
                        self.view_hud(ctx)
                    }}
                </div>

                <Footer
                    show_version={true}
                    version={"3.0.0".to_string()}
                    show_github={true}
                    github_url={Some("https://github.com/UberMetroid/pulse".to_string())}
                />
            </>
        }
    }
}
