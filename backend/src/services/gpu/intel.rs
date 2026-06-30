use super::GpuStats;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn get_intel_stats() -> Option<Vec<GpuStats>> {
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
    let mut last_error = String::new();
    for path in paths {
        match Command::new(path)
            .args(["-J", "-s", "100", "-n", "1"])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    output_str = Some(String::from_utf8_lossy(&output.stdout).to_string());
                    break;
                } else {
                    last_error = format!(
                        "{} exited with status: {} - stderr: {}",
                        path,
                        output.status,
                        String::from_utf8_lossy(&output.stderr).trim()
                    );
                }
            }
            Err(e) => {
                last_error = format!("failed to execute {}: {}", path, e);
            }
        }
    }

    if output_str.is_none() && !last_error.is_empty() {
        tracing::warn!("Intel GPU stats query failed: {}", last_error);
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
