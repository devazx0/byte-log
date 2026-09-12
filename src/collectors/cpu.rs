use sysinfo::System;

#[derive(Debug, Clone)]
pub struct CpuMetrics {
    pub global_utilization: f32,
    pub per_core_utilization: Vec<f32>,
    pub core_count: usize,
    pub frequency_mhz: u64,
    pub load_average_one: Option<f64>,
}

pub struct CpuCollector;

impl CpuCollector {
    pub fn collect(sys: &mut System) -> CpuMetrics {
        sys.refresh_cpu_all();

        let global_utilization = sys.global_cpu_usage();
        let per_core_utilization: Vec<f32> = sys.cpus().iter().map(|c| c.cpu_usage()).collect();
        let core_count = sys.cpus().len();
        let frequency_mhz = sys.cpus().first().map(|c| c.frequency()).unwrap_or(0);

        let load_avg = System::load_average();

        CpuMetrics {
            global_utilization,
            per_core_utilization,
            core_count,
            frequency_mhz,
            load_average_one: Some(load_avg.one),
        }
    }
}
