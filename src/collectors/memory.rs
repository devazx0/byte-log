use sysinfo::System;

#[derive(Debug, Clone)]
pub struct MemoryMetrics {
    pub total_memory_bytes: u64,
    pub used_memory_bytes: u64,
    pub free_memory_bytes: u64,
    pub available_memory_bytes: u64,
    pub total_swap_bytes: u64,
    pub used_swap_bytes: u64,
}

pub struct MemoryCollector;

impl MemoryCollector {
    pub fn collect(sys: &mut System) -> MemoryMetrics {
        sys.refresh_memory();

        MemoryMetrics {
            total_memory_bytes: sys.total_memory(),
            used_memory_bytes: sys.used_memory(),
            free_memory_bytes: sys.free_memory(),
            available_memory_bytes: sys.available_memory(),
            total_swap_bytes: sys.total_swap(),
            used_swap_bytes: sys.used_swap(),
        }
    }
}
