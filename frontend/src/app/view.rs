use crate::app::App;
use crate::app::Msg;
use yew::prelude::*;

impl App {
    pub fn view_login(&self, ctx: &Context<Self>) -> Html {
        let error_html = self.error_message.as_ref().map(|err| {
            html! { <div class="login-error-banner">{err}</div> }
        });

        let pin_display = (0..self.pin_length)
            .map(|i| {
                let dot_class = if i < self.pin_input.len() {
                    "login-dot active"
                } else {
                    "login-dot"
                };
                html! { <div class={dot_class}></div> }
            })
            .collect::<Html>();

        let keypad_buttons = ["1", "2", "3", "4", "5", "6", "7", "8", "9"]
            .iter()
            .map(|&digit| {
                let d = digit.to_string();
                let on_click = ctx.link().callback(move |_| Msg::PinInput(d.clone()));
                html! {
                    <button class="login-keypad-btn" onclick={on_click}>{digit}</button>
                }
            })
            .collect::<Html>();

        let on_backspace = ctx.link().callback(|_| Msg::PinBackspace);

        html! {
            <div class="login-container">
                <div class="login-card">
                    <div class="login-header">
                        <div class="login-icon-frame">
                            <img src="/favicon.svg" class="login-app-icon" alt="Pulse" />
                        </div>
                        <h2>{"PULSE DETECTOR"}</h2>
                        <p>{"ENTER SECURITY PIN TO ACCESS VISOR HUD"}</p>
                    </div>

                    {error_html}

                    <div class="login-dots-wrapper">
                        {pin_display}
                    </div>

                    <div class="login-keypad-grid">
                        {keypad_buttons}
                        <button class="login-keypad-btn fn-btn" disabled=true></button>
                        <button class="login-keypad-btn" onclick={ctx.link().callback(|_| Msg::PinInput("0".to_string()))}>{"0"}</button>
                        <button class="login-keypad-btn fn-btn" onclick={on_backspace}>
                            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5">
                                <path d="M21 4H8l-7 8 7 8h13a2 2 0 0 0 2-2V6a2 2 0 0 0-2-2zM18 9l-6 6M12 9l6 6" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        }
    }

    pub fn view_hud(&self, ctx: &Context<Self>) -> Html {
        let stats = match &self.stats {
            Some(s) => s,
            None => {
                return html! {
                    <div class="hud-loading-frame">
                        <div class="hud-spinner"></div>
                        <p>{"ACQUIRING HOST TELEMETRY..."}</p>
                    </div>
                };
            }
        };

        let cpu_subtitle = format!("{} Threads", stats.cpu_cores.len());
        let ram_used_gb = stats.ram_used as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_total_gb = stats.ram_total as f32 / 1024.0 / 1024.0 / 1024.0;
        let ram_percent = (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).min(100.0).max(0.0);
        let ram_subtitle = format!("{:.1} / {:.1} GB", ram_used_gb, ram_total_gb);

        let uptime_hours = stats.uptime as f32 / 3600.0;
        let uptime_str = if uptime_hours >= 24.0 {
            format!("{:.1} Days", uptime_hours / 24.0)
        } else {
            format!("{:.1} Hours", uptime_hours)
        };

        html! {
            <div class="hud-visor-container">
                <div class="hud-visor-grid">
                    { self.render_radial_gauge("CPU", stats.cpu_global, &cpu_subtitle) }
                    { self.render_radial_gauge("RAM", ram_percent, &ram_subtitle) }

                    <div class="hud-metric-card network-card">
                        <h3>{"NETWORK TELEMETRY"}</h3>
                        <div class="net-grid">
                            <div class="net-row">
                                <span class="net-label">{"RX (IN)"}</span>
                                <span class="net-value upload-glow">{self.format_bytes(stats.net_in)}</span>
                            </div>
                            <div class="net-row">
                                <span class="net-label">{"TX (OUT)"}</span>
                                <span class="net-value download-glow">{self.format_bytes(stats.net_out)}</span>
                            </div>
                        </div>
                        <div class="net-radar-container">
                            <div class="net-radar-circle"></div>
                            <div class="net-radar-sweep"></div>
                        </div>
                    </div>

                    <div class="hud-metric-card gpu-card">
                        <h3>{"AUXILIARY CORE (GPU)"}</h3>
                        {if let Some(gpu) = &stats.gpu {
                            let mem_used_gb = gpu.mem_used.unwrap_or(0) as f32 / 1024.0 / 1024.0 / 1024.0;
                            let mem_total_gb = gpu.mem_total.unwrap_or(0) as f32 / 1024.0 / 1024.0 / 1024.0;
                            html! {
                                <div class="gpu-details">
                                    <div class="gpu-name">{&gpu.name}</div>
                                    <div class="gpu-bar-frame">
                                        <div class="gpu-bar-fill" style={format!("width: {}%;", gpu.usage)}></div>
                                    </div>
                                    <div class="gpu-metrics">
                                        <span>{format!("LOAD: {:.0}%", gpu.usage)}</span>
                                        <span>{format!("TEMP: {}°C", gpu.temp.unwrap_or(0.0))}</span>
                                    </div>
                                    {if gpu.mem_total.is_some() {
                                        html! { <div class="gpu-mem">{format!("VRAM: {:.2} / {:.2} GB", mem_used_gb, mem_total_gb)}</div> }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        } else {
                            html! {
                                <div class="gpu-offline">
                                    <div class="offline-indicator"></div>
                                    <p>{"NO ACTIVE HARDWARE DETECTED"}</p>
                                </div>
                            }
                        }}
                    </div>
                </div>

                <div class="hud-console-wrapper">
                    <div class="hud-console-header">
                        <span>{"AURA MONITOR TERMINAL"}</span>
                        <div class="hud-console-controls">
                            <span>{format!("Uptime: {}", uptime_str)}</span>
                            <button onclick={ctx.link().callback(|_| Msg::ClearTerminal)}>{"CLEAR"}</button>
                            <button onclick={ctx.link().callback(|_| Msg::ToggleTerminal)}>
                                {if self.terminal_open { "COLLAPSE" } else { "EXPAND" }}
                            </button>
                        </div>
                    </div>
                    {if self.terminal_open {
                        html! {
                            <div class="hud-console-body">
                                {for self.terminal_logs.iter().rev().map(|log| {
                                    html! { <div class="console-line">{log}</div> }
                                })}
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            </div>
        }
    }

    fn render_radial_gauge(&self, label: &str, percentage: f32, subtitle: &str) -> Html {
        let r = 70.0;
        let circumference = 2.0 * std::f32::consts::PI * r;
        let stroke_offset = circumference * (1.0 - (percentage.min(100.0).max(0.0) / 100.0));
        html! {
            <div class="hud-gauge-card">
                <svg width="180" height="180" viewBox="0 0 180 180" class="hud-svg-gauge">
                    <circle cx="90" cy="90" r="85" class="hud-gauge-outer-ring" />
                    <circle cx="90" cy="90" r="70" class="hud-gauge-track" />
                    <circle cx="90" cy="90" r="70" class="hud-gauge-fill"
                        stroke-dasharray={circumference.to_string()}
                        stroke-dashoffset={stroke_offset.to_string()}
                        transform="rotate(-90 90 90)"
                    />
                    <text x="90" y="85" text-anchor="middle" class="hud-gauge-percent">
                        {format!("{:.0}%", percentage)}
                    </text>
                    <text x="90" y="115" text-anchor="middle" class="hud-gauge-label">
                        {label}
                    </text>
                </svg>
                <div class="hud-gauge-sub">{subtitle}</div>
            </div>
        }
    }

    fn format_bytes(&self, bytes: u64) -> String {
        if bytes >= 1024 * 1024 {
            format!("{:.2} MB/s", bytes as f64 / 1024.0 / 1024.0)
        } else {
            format!("{:.1} KB/s", bytes as f64 / 1024.0)
        }
    }
}
