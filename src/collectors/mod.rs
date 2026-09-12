pub mod battery;
pub mod cpu;
pub mod disk;
pub mod gpu;
pub mod memory;
pub mod network;
pub mod thermal;

pub use battery::{BatteryCollector, BatteryMetrics};
pub use cpu::{CpuCollector, CpuMetrics};
pub use disk::{DiskCollector, DiskMetrics};
pub use gpu::{GpuCollector, GpuSnapshot};
pub use memory::{MemoryCollector, MemoryMetrics};
pub use network::{NetworkCollector, NetworkMetrics};
pub use thermal::{ThermalCollector, ThermalMetrics};

use crate::platform::Platform;
use chrono::{DateTime, Utc};
use sysinfo::System;

#[derive(Debug, Clone)]
pub struct SystemSnapshot {
    pub timestamp: DateTime<Utc>,
    pub cpu_utilization: f32,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub swap_used_bytes: u64,
    pub disk_read_bytes_sec: u64,
    pub disk_write_bytes_sec: u64,
    pub net_rx_bytes_sec: u64,
    pub net_tx_bytes_sec: u64,
    pub cpu_temp_celsius: Option<f32>,
    pub gpu_utilization: Option<f32>,
    pub battery_percent: Option<f32>,
    pub battery_charging: Option<bool>,
}

pub struct SystemTelemetryCollector {
    sys: System,
    disk_collector: DiskCollector,
    net_collector: NetworkCollector,
    thermal_collector: ThermalCollector,
}

impl Default for SystemTelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemTelemetryCollector {
    pub fn new() -> Self {
        let mut sys = System::new();
        sys.refresh_cpu_all();
        sys.refresh_memory();

        Self {
            sys,
            disk_collector: DiskCollector::new(),
            net_collector: NetworkCollector::new(),
            thermal_collector: ThermalCollector::new(),
        }
    }

    pub fn collect_snapshot(&mut self, platform: &Platform) -> SystemSnapshot {
        let cpu = CpuCollector::collect(&mut self.sys);
        let mem = MemoryCollector::collect(&mut self.sys);
        let disk = self.disk_collector.collect(&mut self.sys);
        let net = self.net_collector.collect();
        let thermal = self.thermal_collector.collect();
        let battery = BatteryCollector::collect(platform);
        let gpu = GpuCollector::collect(platform);

        SystemSnapshot {
            timestamp: Utc::now(),
            cpu_utilization: cpu.global_utilization,
            memory_used_bytes: mem.used_memory_bytes,
            memory_total_bytes: mem.total_memory_bytes,
            swap_used_bytes: mem.used_swap_bytes,
            disk_read_bytes_sec: disk.read_bytes_sec,
            disk_write_bytes_sec: disk.write_bytes_sec,
            net_rx_bytes_sec: net.rx_bytes_sec,
            net_tx_bytes_sec: net.tx_bytes_sec,
            cpu_temp_celsius: thermal.package_temp_celsius,
            gpu_utilization: gpu.and_then(|g| g.utilization_percent),
            battery_percent: battery.as_ref().map(|b| b.percentage),
            battery_charging: battery.as_ref().map(|b| b.is_charging),
        }
    }
}
