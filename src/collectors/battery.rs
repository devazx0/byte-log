use crate::platform::traits::BatteryProvider;

#[derive(Debug, Clone)]
pub struct BatteryMetrics {
    pub percentage: f32,
    pub is_charging: bool,
    pub health_percent: Option<f32>,
    pub cycle_count: Option<u32>,
}

pub struct BatteryCollector;

impl BatteryCollector {
    pub fn collect<P: BatteryProvider>(provider: &P) -> Option<BatteryMetrics> {
        provider.get_battery_info().map(|info| BatteryMetrics {
            percentage: info.percentage,
            is_charging: info.is_charging,
            health_percent: info.health_percent,
            cycle_count: info.cycle_count,
        })
    }
}
