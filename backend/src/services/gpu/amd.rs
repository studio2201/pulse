use super::GpuStats;
use std::fs;
use std::path::Path;

pub fn get_amd_stats() -> Option<Vec<GpuStats>> {
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
    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}
