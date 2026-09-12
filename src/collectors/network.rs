use std::time::Instant;
use sysinfo::Networks;

#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub total_rx_bytes: u64,
    pub total_tx_bytes: u64,
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
}

pub struct NetworkCollector {
    networks: Networks,
    last_sample: Instant,
    last_rx: u64,
    last_tx: u64,
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl NetworkCollector {
    pub fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let mut initial_rx = 0;
        let mut initial_tx = 0;
        for (_interface_name, data) in &networks {
            initial_rx += data.total_received();
            initial_tx += data.total_transmitted();
        }

        Self {
            networks,
            last_sample: Instant::now(),
            last_rx: initial_rx,
            last_tx: initial_tx,
        }
    }

    pub fn collect(&mut self) -> NetworkMetrics {
        self.networks.refresh(true);

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_sample).as_secs_f64();

        let mut current_rx = 0;
        let mut current_tx = 0;

        for (_interface_name, data) in &self.networks {
            current_rx += data.total_received();
            current_tx += data.total_transmitted();
        }

        let rx_delta = current_rx.saturating_sub(self.last_rx);
        let tx_delta = current_tx.saturating_sub(self.last_tx);

        let rx_rate = if elapsed > 0.0 {
            (rx_delta as f64 / elapsed) as u64
        } else {
            0
        };

        let tx_rate = if elapsed > 0.0 {
            (tx_delta as f64 / elapsed) as u64
        } else {
            0
        };

        self.last_sample = now;
        self.last_rx = current_rx;
        self.last_tx = current_tx;

        NetworkMetrics {
            total_rx_bytes: current_rx,
            total_tx_bytes: current_tx,
            rx_bytes_sec: rx_rate,
            tx_bytes_sec: tx_rate,
        }
    }
}
