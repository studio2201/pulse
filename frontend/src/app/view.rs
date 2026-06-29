use crate::app::App;
use crate::app::Msg;
use yew::prelude::*;

impl App {
    pub fn view_hud(&self, ctx: &Context<Self>) -> Html {
        let uptime_str = if let Some(stats) = &self.stats {
            let seconds = stats.uptime;
            let days = seconds / 86400;
            let hours = (seconds % 86400) / 3600;
            let minutes = (seconds % 3600) / 60;
            let secs = seconds % 60;
            if days > 0 {
                format!("{}d {}h {}m", days, hours, minutes)
            } else if hours > 0 {
                format!("{}h {}m {}s", hours, minutes, secs)
            } else {
                format!("{}m {}s", minutes, secs)
            }
        } else {
            "--".to_string()
        };

        let console_title = if let Some(stats) = &self.stats {
            stats.hostname.to_uppercase()
        } else {
            "CONSOLE MONITOR".to_string()
        };

        html! {
            <div class="hud-visor-container">
                <div class="hud-visor-grid">
                    // CPU Card
                    <div class="hud-metric-card">
                        <h3>{"CPU"}</h3>
                        {if let Some(stats) = &self.stats {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-main-val">{format!("{:.1}%", stats.cpu_global)}</div>
                                    <div class="card-subtext">{format!("{} Threads", stats.cpu_cores.len())}</div>
                                    <div class="hud-bar-frame">
                                        <div class="hud-bar-fill" style={format!("width: {}%;", stats.cpu_global)}></div>
                                    </div>
                                    { self.render_sparkline(&self.cpu_history, 100.0) }
                                </div>
                            }
                        } else {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-loading">{"Connecting..."}</div>
                                    { self.render_sparkline(&self.cpu_history, 100.0) }
                                </div>
                            }
                        }}
                    </div>

                    // Memory Card
                    <div class="hud-metric-card">
                        <h3>{"MEMORY"}</h3>
                        {if let Some(stats) = &self.stats {
                            let ram_used_gb = stats.ram_used as f32 / 1024.0 / 1024.0 / 1024.0;
                            let ram_total_gb = stats.ram_total as f32 / 1024.0 / 1024.0 / 1024.0;
                            let ram_percent = (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).min(100.0).max(0.0);
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-main-val">{format!("{:.1} / {:.1} GB", ram_used_gb, ram_total_gb)}</div>
                                    <div class="card-subtext">{format!("{:.1}% Used", ram_percent)}</div>
                                    <div class="hud-bar-frame">
                                        <div class="hud-bar-fill" style={format!("width: {}%;", ram_percent)}></div>
                                    </div>
                                    { self.render_sparkline(&self.ram_history, 100.0) }
                                </div>
                            }
                        } else {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-loading">{"Connecting..."}</div>
                                    { self.render_sparkline(&self.ram_history, 100.0) }
                                </div>
                            }
                        }}
                    </div>

                    // Storage Card
                    <div class="hud-metric-card">
                        <h3>{"STORAGE"}</h3>
                        {if let Some(stats) = &self.stats {
                            let disk_used_gb = stats.disk_used as f32 / 1024.0 / 1024.0 / 1024.0;
                            let disk_total_gb = stats.disk_total as f32 / 1024.0 / 1024.0 / 1024.0;
                            let disk_percent = (stats.disk_used as f32 / stats.disk_total as f32 * 100.0).min(100.0).max(0.0);
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-main-val">{format!("{:.1} / {:.1} GB", disk_used_gb, disk_total_gb)}</div>
                                    <div class="card-subtext">{format!("{:.1}% Used", disk_percent)}</div>
                                    <div class="hud-bar-frame">
                                        <div class="hud-bar-fill" style={format!("width: {}%;", disk_percent)}></div>
                                    </div>
                                    { self.render_sparkline(&self.disk_history, 100.0) }
                                </div>
                            }
                        } else {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-loading">{"Connecting..."}</div>
                                    { self.render_sparkline(&self.disk_history, 100.0) }
                                </div>
                            }
                        }}
                    </div>

                    // Network Card
                    <div class="hud-metric-card">
                        <h3>{"NETWORK"}</h3>
                        {if let Some(stats) = &self.stats {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-main-val download-glow">{format!("↓ {}", self.format_bytes(stats.net_in))}</div>
                                    <div class="card-subtext upload-glow">{format!("↑ {}", self.format_bytes(stats.net_out))}</div>
                                    { self.render_sparkline(&self.net_history, 0.0) }
                                </div>
                            }
                        } else {
                            html! {
                                <div class="card-metric-block">
                                    <div class="card-loading">{"Connecting..."}</div>
                                    { self.render_sparkline(&self.net_history, 0.0) }
                                </div>
                            }
                        }}
                    </div>

                    // GPU Card(s)
                    {if let Some(stats) = &self.stats {
                        if stats.gpus.is_empty() {
                            html! {
                                <div class="hud-metric-card">
                                    <h3>{"GPU"}</h3>
                                    <div class="card-metric-block">
                                        <div class="card-main-val" style="color: var(--text-muted); font-size: 1.5rem;">{"OFFLINE"}</div>
                                        <div class="card-subtext">{"No Active GPU"}</div>
                                        { self.render_sparkline(&[], 100.0) }
                                    </div>
                                </div>
                            }
                        } else {
                            stats.gpus.iter().enumerate().map(|(idx, gpu)| {
                                let history = self.gpu_histories.get(idx).map(|h| h.as_slice()).unwrap_or(&[]);
                                let card_title = if stats.gpus.len() > 1 {
                                    format!("GPU {}", idx + 1)
                                } else {
                                    "GPU".to_string()
                                };
                                html! {
                                    <div class="hud-metric-card" key={idx}>
                                        <h3>{card_title}</h3>
                                        <div class="card-metric-block">
                                            <div class="card-main-val">{format!("{:.0}%", gpu.usage)}</div>
                                            <div class="card-subtext" title={gpu.name.clone()}>{&gpu.name}</div>
                                            <div class="hud-bar-frame">
                                                <div class="hud-bar-fill" style={format!("width: {}%;", gpu.usage)}></div>
                                            </div>
                                            { self.render_sparkline(history, 100.0) }
                                        </div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }
                    } else {
                        html! {
                            <div class="hud-metric-card">
                                <h3>{"GPU"}</h3>
                                <div class="card-metric-block">
                                    <div class="card-loading">{"Connecting..."}</div>
                                    { self.render_sparkline(&[], 100.0) }
                                </div>
                            </div>
                        }
                    }}
                </div>

                <div class="hud-console-wrapper">
                    <div class="hud-console-header">
                        <span>{console_title}</span>
                        <div class="hud-console-controls">
                            <span>{format!("Uptime: {}", uptime_str)}</span>
                            <button onclick={ctx.link().callback(|_| Msg::ClearTerminal)}>{"CLEAR"}</button>
                        </div>
                    </div>
                    <div class="hud-console-body">
                        {for self.terminal_logs.iter().rev().map(|log| {
                            html! { <div class="console-line">{log}</div> }
                        })}
                    </div>
                </div>
            </div>
        }
    }

    fn render_sparkline(&self, history: &[f32], max_val: f32) -> Html {
        if history.is_empty() {
            return html! {
                <div style="font-family: monospace; font-size: 0.8rem; color: var(--text-secondary); opacity: 0.5; padding: 0.5rem 0;">
                    {"Awaiting telemetry..."}
                </div>
            };
        }

        let width = 140.0;
        let height = 24.0;
        let points_count = history.len();
        
        let effective_max = if max_val > 0.0 { max_val } else {
            history.iter().copied().fold(0.0f32, f32::max).max(1.0)
        };

        let points = history.iter().enumerate().map(|(idx, &val)| {
            let x = if points_count > 1 { (idx as f32 / (points_count - 1) as f32) * width } else { 0.0 };
            let percent = (val / effective_max).min(1.0).max(0.0);
            let y = height - (percent * (height - 4.0)) - 2.0;
            format!("{:.1},{:.1}", x, y)
        }).collect::<Vec<String>>().join(" ");

        html! {
            <div style="width: 100%; height: 24px; margin-top: 0.5rem; opacity: 0.85;">
                <svg width="100%" height="24" viewBox={format!("0 0 {} {}", width, height)} preserveAspectRatio="none" style="display: block; overflow: visible;">
                    <polyline
                        fill="none"
                        stroke="var(--primary)"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        points={points}
                    />
                </svg>
            </div>
        }
    }

    fn format_bytes(&self, bytes: u64) -> String {
        if bytes >= 1024 * 1024 {
            format!("{:.2} MB/s", bytes as f64 / 1024.0 / 1024.0)
        } else if bytes >= 1024 {
            format!("{:.1} KB/s", bytes as f64 / 1024.0)
        } else {
            format!("{} B/s", bytes)
        }
    }
}
