use crate::app::App;
use yew::prelude::*;

impl App {
    pub fn view_sys_info_card(&self) -> Html {
        if let Some(stats) = &self.stats {
            let uptime_str = {
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
            };

            let os_name = match self.os_override {
                None => stats.os_name.clone(),
                Some(0) => "UBI".to_string(),
                Some(1) => "ubuntu".to_string(),
                Some(2) => "debian".to_string(),
                Some(3) => "arch".to_string(),
                Some(4) => "fedora".to_string(),
                Some(5) => "pop".to_string(),
                Some(6) => "unraid".to_string(),
                Some(7) => "gentoo".to_string(),
                Some(8) => "guix".to_string(),
                Some(9) => "win11".to_string(),
                Some(10) => "talos".to_string(),
                Some(11) => "bottlerocket".to_string(),
                Some(12) => "flatcar".to_string(),
                Some(13) => "alpine".to_string(),
                Some(14) => "generic".to_string(),
                _ => "generic".to_string(),
            };

            let os_ascii = self.get_os_logo(&os_name);

            html! {
                <div class="hud-metric-card sys-info-card" title="System Information">
                    <div class="sys-info-left">
                        <h3>{crate::i18n::lookup(crate::i18n::PulseKey::SystemInfo, self.language)}</h3>
                        <div class="card-metric-block sys-details">
                            <div class="sys-detail-row">
                                <span class="sys-detail-label">{format!("{}:", crate::i18n::lookup(crate::i18n::PulseKey::Hostname, self.language).to_uppercase())}</span>
                                <span class="sys-detail-val hostname-glow">{ &stats.hostname }</span>
                            </div>
                            <div class="sys-detail-row">
                                <span class="sys-detail-label">{format!("{}:", crate::i18n::lookup(crate::i18n::PulseKey::Os, self.language).to_uppercase())}</span>
                                <span class="sys-detail-val">{
                                    if self.os_override.is_some() {
                                        os_name.to_uppercase()
                                    } else {
                                        format!("{} {}", stats.os_name, stats.os_version).trim().to_string()
                                    }
                                }</span>
                            </div>
                            <div class="sys-detail-row">
                                <span class="sys-detail-label">{format!("{}:", crate::i18n::lookup(crate::i18n::PulseKey::Kernel, self.language).to_uppercase())}</span>
                                <span class="sys-detail-val">{ &stats.kernel_version }</span>
                            </div>
                            <div class="sys-detail-row">
                                <span class="sys-detail-label">{format!("{}:", crate::i18n::lookup(crate::i18n::PulseKey::Uptime, self.language).to_uppercase())}</span>
                                <span class="sys-detail-val">{ uptime_str }</span>
                            </div>
                        </div>
                    </div>
                    <div class="sys-info-right ascii-container">
                        { os_ascii }
                    </div>
                </div>
            }
        } else {
            html! {
                <div class="hud-metric-card sys-info-card">
                    <div class="sys-info-left">
                        <h3>{crate::i18n::lookup(crate::i18n::PulseKey::SystemInfo, self.language)}</h3>
                        <div class="card-metric-block">
                            <div class="card-loading">{"Connecting..."}</div>
                        </div>
                    </div>
                </div>
            }
        }
    }
}
