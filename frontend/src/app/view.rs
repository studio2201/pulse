use crate::app::App;
use crate::app::Msg;
use yew::prelude::*;

impl App {
    pub fn view_hud(&self, ctx: &Context<Self>) -> Html {
        html! {
            <div class={classes!("hud-visor-container", (!self.monitor_console).then_some("no-console"))}>
                { if self.monitor_console { self.view_console(ctx) } else { html! {} } }
                { self.view_metrics_grid() }
            </div>
        }
    }

    fn view_console(&self, ctx: &Context<Self>) -> Html {
        let console_title = if let Some(stats) = &self.stats {
            stats.hostname.to_uppercase()
        } else {
            "CONSOLE MONITOR".to_string()
        };

        html! {
            <div class="hud-console-wrapper">
                <div class="hud-console-header">
                    <span class="hostname-glow">
                        {console_title}
                    </span>
                    <div class="hud-console-controls">
                        <button onclick={ctx.link().callback(|_| Msg::DecreaseFontSize)} title="Decrease Font Size" class="font-btn">{"A-"}</button>
                        <button onclick={ctx.link().callback(|_| Msg::IncreaseFontSize)} title="Increase Font Size" class="font-btn">{"A+"}</button>
                        {
                            if self.console_paused {
                                html! {
                                    <button onclick={ctx.link().callback(|_| Msg::TogglePauseConsole)} title="Resume Auto-Scroll" class="font-btn pause-btn active-paused">{"PLAY"}</button>
                                }
                            } else {
                                html! {
                                    <button onclick={ctx.link().callback(|_| Msg::TogglePauseConsole)} title="Pause Auto-Scroll" class="font-btn pause-btn">{"PAUSE"}</button>
                                }
                            }
                        }
                        <button onclick={ctx.link().callback(|_| Msg::ClearTerminal)}>{"CLEAR"}</button>
                    </div>
                </div>
                <div class="hud-console-body" ref={self.console_ref.clone()} onmouseup={ctx.link().callback(|_| Msg::ConsoleMouseUp)} style={format!("font-size: {}rem;", self.console_font_size)}>
                    {for self.terminal_logs.iter().map(|log| {
                        let log_cls = if log.contains("ERROR") || log.contains("Error") || log.contains("Failed") || log.contains("failed") || log.contains("CRITICAL") {
                            "console-line error"
                        } else if log.contains("warning") || log.contains("WARNING") || log.contains("warn") || log.contains("WARN") {
                            "console-line warning"
                        } else if log.contains("[SYSTEM]") || log.contains("[WS]") {
                            "console-line system"
                        } else {
                            "console-line info"
                        };
                        html! { <div class={log_cls}>{log}</div> }
                    })}
                </div>
            </div>
        }
    }

    fn view_metrics_grid(&self) -> Html {
        html! {
            <div class="hud-visor-grid">
                { self.view_sys_info_card() }
                { if self.monitor_cpu { self.view_cpu_card() } else { html! {} } }
                { if self.monitor_memory { self.view_memory_card() } else { html! {} } }
                { if self.monitor_storage { self.view_storage_card() } else { html! {} } }
                { if self.monitor_network { self.view_network_card() } else { html! {} } }
                { if self.monitor_gpu { self.view_gpu_card() } else { html! {} } }
            </div>
        }
    }
}
