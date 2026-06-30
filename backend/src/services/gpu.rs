use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

static HAS_NVIDIA_SMI: OnceLock<bool> = OnceLock::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuStats {
    pub name: String,
    pub usage: f32,
    pub temp: Option<f32>,
    pub mem_used: Option<u64>,
    pub mem_total: Option<u64>,
}

pub fn get_gpu_stats() -> Vec<GpuStats> {
    let mut gpus = Vec::new();
    if let Some(mut nvidia) = get_nvidia_stats() {
        gpus.append(&mut nvidia);
    }
    if let Some(mut amd) = get_amd_stats() {
        gpus.append(&mut amd);
    }
    if let Some(mut intel) = get_intel_stats() {
        gpus.append(&mut intel);
    }
    gpus
}

fn get_nvidia_stats() -> Option<Vec<GpuStats>> {
    let has_nvidia_smi = *HAS_NVIDIA_SMI.get_or_init(|| {
        let paths = [
            "nvidia-smi",
            "/usr/bin/nvidia-smi",
            "/usr/local/nvidia/bin/nvidia-smi",
            "/run/wrapper/bin/nvidia-smi",
        ];
        for path in paths {
            if Command::new(path).arg("--help").status().is_ok() {
                return true;
            }
        }
        false
    });

    if !has_nvidia_smi && !Path::new("/dev/nvidiactl").exists() {
        return None;
    }

    let paths = [
        "nvidia-smi",
        "/usr/bin/nvidia-smi",
        "/usr/local/nvidia/bin/nvidia-smi",
        "/run/wrapper/bin/nvidia-smi",
    ];

    let mut output = None;
    for path in paths {
        if let Ok(out) = Command::new(path)
            .args([
                "--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total,name",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            if out.status.success() {
                output = Some(out);
                break;
            }
        }
    }

    let output = output?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut gpus = Vec::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
        if parts.len() >= 5 {
            let usage = parts[0].parse::<f32>().unwrap_or(0.0);
            let temp = parts[1].parse::<f32>().ok();
            let mem_used_mb = parts[2].parse::<u64>().ok();
            let mem_total_mb = parts[3].parse::<u64>().ok();
            let name = parts[4].to_string();
            gpus.push(GpuStats {
                name,
                usage,
                temp,
                mem_used: mem_used_mb.map(|m| m * 1024 * 1024),
                mem_total: mem_total_mb.map(|m| m * 1024 * 1024),
            });
        }
    }
    if gpus.is_empty() { None } else { Some(gpus) }
}

fn get_amd_stats() -> Option<Vec<GpuStats>> {
    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        return None;
    }

    let mut gpus = Vec::new();
    let entries = fs::read_dir(drm_path).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("card") && !name.contains('-') {
            let device_path = entry.path().join("device");
            let busy_path = device_path.join("gpu_busy_percent");
            if busy_path.exists() {
                let usage = fs::read_to_string(busy_path)
                    .ok()
                    .and_then(|s| s.trim().parse::<f32>().ok())
                    .unwrap_or(0.0);

                let mut mem_used = None;
                let mut mem_total = None;
                let used_path = device_path.join("mem_info_vram_used");
                let total_path = device_path.join("mem_info_vram_total");
                if used_path.exists() && total_path.exists() {
                    mem_used = fs::read_to_string(used_path)
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok());
                    mem_total = fs::read_to_string(total_path)
                        .ok()
                        .and_then(|s| s.trim().parse::<u64>().ok());
                }

                let mut temp = None;
                let hwmon_path = device_path.join("hwmon");
                if let Ok(hwmons) = fs::read_dir(hwmon_path) {
                    for hwmon in hwmons.flatten() {
                        let temp_file = hwmon.path().join("temp1_input");
                        if let Some(milli_celsius) = fs::read_to_string(temp_file)
                            .ok()
                            .and_then(|s| s.trim().parse::<f32>().ok())
                        {
                            temp = Some(milli_celsius / 1000.0);
                            break;
                        }
                    }
                }

                gpus.push(GpuStats {
                    name: format!("AMD Radeon GPU ({})", name),
                    usage,
                    temp,
                    mem_used,
                    mem_total,
                });
            }
        }
    }
    if gpus.is_empty() { None } else { Some(gpus) }
}

fn get_intel_stats() -> Option<Vec<GpuStats>> {
    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        return None;
    }

    let mut has_intel = false;
    if let Ok(entries) = fs::read_dir(drm_path) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("card") && !name.contains('-') {
                if let Ok(vendor) = fs::read_to_string(entry.path().join("device/vendor")) {
                    if vendor.trim().contains("0x8086") {
                        has_intel = true;
                        break;
                    }
                }
            }
        }
    }

    if !has_intel {
        return None;
    }

    let paths = [
        "intel_gpu_top",
        "/usr/bin/intel_gpu_top",
        "/usr/local/bin/intel_gpu_top",
    ];

    let mut output_str = None;
    for path in paths {
        if let Ok(output) = Command::new(path)
            .args(["-J", "-s", "100", "-n", "1"])
            .output()
        {
            if output.status.success() {
                output_str = Some(String::from_utf8_lossy(&output.stdout).to_string());
                break;
            }
        }
    }

    let stdout = output_str?;
    let mut usage = 0.0f32;
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&stdout) {
        let obj = if let Some(arr) = val.as_array() {
            arr.first().cloned().unwrap_or(serde_json::Value::Null)
        } else {
            val
        };

        if let Some(engines) = obj.get("engines") {
            if let Some(engines_obj) = engines.as_object() {
                let mut max_busy = 0.0f32;
                for (_engine_name, info) in engines_obj {
                    if let Some(busy_val) = info.get("busy").and_then(|b| b.as_f64()) {
                        max_busy = max_busy.max(busy_val as f32);
                    }
                }
                usage = max_busy;
            }
        }
    } else {
        let mut max_busy = 0.0f32;
        let mut search_str = &stdout[..];
        while let Some(idx) = search_str.find("\"busy\"") {
            let rest = &search_str[idx + 6..];
            if let Some(colon_idx) = rest.find(':') {
                let after_colon = &rest[colon_idx + 1..];
                let num_str: String = after_colon
                    .chars()
                    .take_while(|c| c.is_ascii_digit() || *c == '.')
                    .collect();
                if let Ok(b) = num_str.trim().parse::<f32>() {
                    max_busy = max_busy.max(b);
                }
            }
            search_str = &search_str[idx + 6..];
        }
        usage = max_busy;
    }

    Some(vec![GpuStats {
        name: "Intel HD/Iris/Arc Graphics".to_string(),
        usage,
        temp: None,
        mem_used: None,
        mem_total: None,
    }])
}
