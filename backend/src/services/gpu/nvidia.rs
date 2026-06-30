use super::GpuStats;
use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

static HAS_NVIDIA_SMI: OnceLock<bool> = OnceLock::new();

pub fn get_nvidia_stats() -> Option<Vec<GpuStats>> {
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
        tracing::debug!("Nvidia GPU stats skipped: nvidia-smi not found and /dev/nvidiactl does not exist.");
        return None;
    }

    let paths = [
        "nvidia-smi",
        "/usr/bin/nvidia-smi",
        "/usr/local/nvidia/bin/nvidia-smi",
        "/run/wrapper/bin/nvidia-smi",
    ];

    let mut output = None;
    let mut last_error = String::new();
    for path in paths {
        match Command::new(path)
            .args([
                "--query-gpu=utilization.gpu,temperature.gpu,memory.used,memory.total,name",
                "--format=csv,noheader,nounits",
            ])
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    output = Some(out);
                    break;
                } else {
                    last_error = format!(
                        "{} exited with status: {} - stderr: {}",
                        path,
                        out.status,
                        String::from_utf8_lossy(&out.stderr).trim()
                    );
                }
            }
            Err(e) => {
                last_error = format!("failed to execute {}: {}", path, e);
            }
        }
    }

    if output.is_none() && !last_error.is_empty() {
        tracing::warn!("Nvidia GPU stats query failed: {}", last_error);
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
    if gpus.is_empty() {
        None
    } else {
        Some(gpus)
    }
}
