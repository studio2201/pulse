use crate::app::App;
use yew::prelude::*;

impl App {
    pub fn view_cpu_card(&self) -> Html {
        let cpu_brand = self.stats.as_ref().map(|s| s.cpu_brand.clone()).unwrap_or_default();
        html! {
            <div class="hud-metric-card" title={cpu_brand.clone()}>
                <h3 style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap; max-width: 100%;" title={cpu_brand.clone()}>
                    {if self.stats.is_some() {
                        format!("CPU: {}", cpu_brand)
                    } else {
                        "CPU".to_string()
                    }}
                </h3>
                {if let Some(stats) = &self.stats {
                    html! {
                        <div class="card-metric-block">
                            <div class="card-main-val">{format!("{:.1}%", stats.cpu_global)}</div>
                            <div class="card-subtext">{format!("{} Cores", stats.cpu_cores.len())}</div>
                            <div class="hud-bar-frame"><div class="hud-bar-fill" style={format!("width: {}%;", stats.cpu_global)}></div></div>
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
        }
    }

    pub fn view_memory_card(&self) -> Html {
        html! {
            <div class="hud-metric-card">
                <h3>{"MEMORY"}</h3>
                {if let Some(stats) = &self.stats {
                    let ram_used_gb = stats.ram_used as f32 / 1024.0 / 1024.0 / 1024.0;
                    let ram_total_gb = stats.ram_total as f32 / 1024.0 / 1024.0 / 1024.0;
                    let ram_percent = if stats.ram_total > 0 {
                        (stats.ram_used as f32 / stats.ram_total as f32 * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };
                    html! {
                        <div class="card-metric-block">
                            <div class="card-main-val">{format!("{:.1} / {:.1} GB", ram_used_gb, ram_total_gb)}</div>
                            <div class="card-subtext">{format!("{:.1}% Used", ram_percent)}</div>
                            <div class="hud-bar-frame"><div class="hud-bar-fill" style={format!("width: {}%;", ram_percent)}></div></div>
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
        }
    }

    pub fn view_storage_card(&self) -> Html {
        html! {
            <div class="hud-metric-card">
                <h3>{"STORAGE"}</h3>
                {if let Some(stats) = &self.stats {
                    let disk_used_gb = stats.disk_used as f32 / 1024.0 / 1024.0 / 1024.0;
                    let disk_total_gb = stats.disk_total as f32 / 1024.0 / 1024.0 / 1024.0;
                    let disk_percent = if stats.disk_total > 0 {
                        (stats.disk_used as f32 / stats.disk_total as f32 * 100.0).clamp(0.0, 100.0)
                    } else {
                        0.0
                    };
                    html! {
                        <div class="card-metric-block">
                            <div class="card-main-val">{format!("{:.1} / {:.1} GB", disk_used_gb, disk_total_gb)}</div>
                            <div class="card-subtext">{format!("{:.1}% Used", disk_percent)}</div>
                            <div class="hud-bar-frame"><div class="hud-bar-fill" style={format!("width: {}%;", disk_percent)}></div></div>
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
        }
    }

    pub fn view_network_card(&self) -> Html {
        html! {
            <div class="hud-metric-card">
                <h3>{"NETWORK"}</h3>
                {if let Some(stats) = &self.stats {
                    html! {
                        <div class="card-metric-block">
                            <div class="card-main-val">{format!("▼ {}  ▲ {}", self.format_bytes(stats.net_in), self.format_bytes(stats.net_out))}</div>
                            <div class="card-subtext">{"Throughput"}</div>
                            <div class="hud-bar-frame"><div class="hud-bar-fill animated-flow" style="width: 100%;"></div></div>
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
        }
    }

    pub fn view_gpu_card(&self) -> Html {
        html! {
            <>
            {if let Some(stats) = &self.stats {
                html! {
                    <>
                    {for stats.gpus.iter().enumerate().map(|(idx, gpu)| {
                        let temp_str = gpu.temp.map(|t| format!("{:.0}°C", t)).unwrap_or_else(|| "--".to_string());
                        let name_str = if gpu.name.is_empty() { format!("GPU {}", idx + 1) } else { gpu.name.clone() };
                        html! {
                            <div class="hud-metric-card" title={name_str.clone()}>
                                <h3 style="text-overflow: ellipsis; overflow: hidden; white-space: nowrap; max-width: 100%;" title={name_str.clone()}>
                                    {format!("GPU {}: {}", idx + 1, name_str)}
                                </h3>
                                <div class="card-metric-block">
                                    <div class="card-main-val">{format!("{:.1}%", gpu.usage)}</div>
                                    <div class="card-subtext">{format!("Core Temp: {}", temp_str)}</div>
                                    <div class="hud-bar-frame"><div class="hud-bar-fill" style={format!("width: {}%;", gpu.usage)}></div></div>
                                    { self.render_sparkline(&self.gpu_histories[idx], 100.0) }
                                </div>
                            </div>
                        }
                    })}
                    </>
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
            </>
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
        let height = 16.0;
        let points_count = history.len();

        let effective_max = if max_val > 0.0 {
            max_val
        } else {
            history.iter().copied().fold(0.0f32, f32::max).max(1.0)
        };

        let points = history
            .iter()
            .enumerate()
            .map(|(idx, &val)| {
                let x = if points_count > 1 {
                    (idx as f32 / (points_count - 1) as f32) * width
                } else {
                    0.0
                };
                let percent = (val / effective_max).clamp(0.0, 1.0);
                let y = height - (percent * (height - 3.0)) - 1.5;
                format!("{:.1},{:.1}", x, y)
            })
            .collect::<Vec<String>>()
            .join(" ");

        html! {
            <div style="width: 100%; height: 16px; margin-top: 0.3rem; opacity: 0.85;">
                <svg width="100%" height="16" viewBox={format!("0 0 {} {}", width, height)} preserveAspectRatio="none" style="display: block; overflow: visible;">
                    <polyline fill="none" stroke="var(--primary)" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" points={points} />
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
