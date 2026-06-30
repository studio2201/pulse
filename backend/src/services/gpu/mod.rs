pub mod amd;
pub mod intel;
pub mod nvidia;

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
    if let Some(mut nvidia_stats) = nvidia::get_nvidia_stats() {
        gpus.append(&mut nvidia_stats);
    }
    if let Some(mut amd_stats) = amd::get_amd_stats() {
        gpus.append(&mut amd_stats);
    }
    if let Some(mut intel_stats) = intel::get_intel_stats() {
        gpus.append(&mut intel_stats);
    }
    gpus
}
