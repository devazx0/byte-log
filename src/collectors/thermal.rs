use sysinfo::Components;

#[derive(Debug, Clone)]
pub struct ThermalMetrics {
    pub package_temp_celsius: Option<f32>,
}

pub struct ThermalCollector {
    components: Components,
}

impl Default for ThermalCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl ThermalCollector {
    pub fn new() -> Self {
        Self {
            components: Components::new_with_refreshed_list(),
        }
    }

    pub fn collect(&mut self) -> ThermalMetrics {
        self.components.refresh(true);

        let mut max_temp: Option<f32> = None;

        for component in self.components.list() {
            if let Some(temp) = component.temperature() {
                if temp > 0.0 {
                    max_temp = Some(max_temp.map_or(temp, |curr| curr.max(temp)));
                }
            }
        }

        ThermalMetrics {
            package_temp_celsius: max_temp,
        }
    }
}
