use std::time::Instant;
use sysinfo::{Disks, ProcessRefreshKind, System};

#[derive(Debug, Clone)]
pub struct DiskMetrics {
    pub total_space_bytes: u64,
    pub available_space_bytes: u64,
    pub used_space_bytes: u64,
    pub read_bytes_sec: u64,
    pub write_bytes_sec: u64,
}

pub struct DiskCollector {
    disks: Disks,
    last_sample: Instant,
}

impl Default for DiskCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl DiskCollector {
    pub fn new() -> Self {
        Self {
            disks: Disks::new_with_refreshed_list(),
            last_sample: Instant::now(),
        }
    }

    pub fn collect(&mut self, sys: &mut System) -> DiskMetrics {
        self.disks.refresh(true);

        let mut total_space = 0;
        let mut available_space = 0;

        for disk in self.disks.list() {
            total_space += disk.total_space();
            available_space += disk.available_space();
        }

        let used_space = total_space.saturating_sub(available_space);

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample).as_secs_f64();
        self.last_sample = now;

        // Refresh process disk usage specifically
        sys.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_disk_usage(),
        );

        let mut delta_read = 0;
        let mut delta_written = 0;
        for proc in sys.processes().values() {
            let du = proc.disk_usage();
            delta_read += du.read_bytes;
            delta_written += du.written_bytes;
        }

        let read_rate = if elapsed > 0.0 {
            (delta_read as f64 / elapsed) as u64
        } else {
            0
        };

        let write_rate = if elapsed > 0.0 {
            (delta_written as f64 / elapsed) as u64
        } else {
            0
        };

        DiskMetrics {
            total_space_bytes: total_space,
            available_space_bytes: available_space,
            used_space_bytes: used_space,
            read_bytes_sec: read_rate,
            write_bytes_sec: write_rate,
        }
    }
}
