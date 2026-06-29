use std::process::Command;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GpuStats {
    pub name: String,
    pub usage: f32, // percentage 0-100
    pub temp: Option<f32>, // Celsius
    pub mem_used: Option<u64>, // bytes
    pub mem_total: Option<u64>, // bytes
}

/// Query GPU stats dynamically based on what hardware is detected.
pub fn get_gpu_stats() -> Option<GpuStats> {
    // 1. Try NVIDIA first via nvidia-smi
    if let Some(stats) = get_nvidia_stats() {
        return Some(stats);
    }

    // 2. Try AMD via sysfs
    if let Some(stats) = get_amd_stats() {
        return Some(stats);
    }

    None
}

fn get_nvidia_stats() -> Option<GpuStats> {
    // Check if nvidia-smi exists in path
    if !Command::new("which").arg("nvidia-smi").status().map(|s| s.success()).unwrap_or(false) {
        // Fallback check: check if /dev/nvidiactl exists
        if !Path::new("/dev/nvidiactl").exists() {
            return None;
        }
    }

    let output = Command::new("nvidia-smi")
        .args([
            "--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total,name",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = stdout.trim().split(',').map(|s| s.trim()).collect();
    if parts.len() < 5 {
        return None;
    }

    let usage = parts[0].parse::<f32>().unwrap_or(0.0);
    let temp = parts[1].parse::<f32>().ok();
    let mem_used_mb = parts[2].parse::<u64>().ok();
    let mem_total_mb = parts[3].parse::<u64>().ok();
    let name = parts[4].to_string();

    Some(GpuStats {
        name,
        usage,
        temp,
        mem_used: mem_used_mb.map(|m| m * 1024 * 1024),
        mem_total: mem_total_mb.map(|m| m * 1024 * 1024),
    })
}

fn get_amd_stats() -> Option<GpuStats> {
    // Search /sys/class/drm/card* directories
    let drm_path = Path::new("/sys/class/drm");
    if !drm_path.exists() {
        return None;
    }

    let entries = fs::read_dir(drm_path).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with("card") && !name.contains('-') {
            let device_path = entry.path().join("device");
            let busy_path = device_path.join("gpu_busy_percent");
            if busy_path.exists() {
                // Detected AMD GPU
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

                // Check temperature in hwmon
                let mut temp = None;
                let hwmon_path = device_path.join("hwmon");
                if let Ok(hwmons) = fs::read_dir(hwmon_path) {
                    for hwmon in hwmons.flatten() {
                        let temp_file = hwmon.path().join("temp1_input");
                        if temp_file.exists() {
                            if let Ok(raw_temp) = fs::read_to_string(temp_file) {
                                if let Ok(milli_celsius) = raw_temp.trim().parse::<f32>() {
                                    temp = Some(milli_celsius / 1000.0);
                                    break;
                                }
                            }
                        }
                    }
                }

                return Some(GpuStats {
                    name: "AMD Radeon GPU".to_string(),
                    usage,
                    temp,
                    mem_used,
                    mem_total,
                });
            }
        }
    }

    None
}
