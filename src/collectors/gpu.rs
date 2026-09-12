use crate::platform::traits::GpuProvider;

#[derive(Debug, Clone)]
pub struct GpuSnapshot {
    pub model_name: String,
    pub utilization_percent: Option<f32>,
    pub memory_used_bytes: Option<u64>,
    pub memory_total_bytes: Option<u64>,
    pub temperature_celsius: Option<f32>,
}

pub struct GpuCollector;

impl GpuCollector {
    pub fn collect<P: GpuProvider>(provider: &P) -> Option<GpuSnapshot> {
        provider.get_gpu_info().map(|info| GpuSnapshot {
            model_name: info.model_name,
            utilization_percent: info.utilization_percent,
            memory_used_bytes: info.memory_used_bytes,
            memory_total_bytes: info.memory_total_bytes,
            temperature_celsius: info.temperature_celsius,
        })
    }
}
