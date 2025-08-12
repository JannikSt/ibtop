use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::types::{AdapterInfo, PortCounters};

#[derive(Debug, Clone)]
pub struct PortMetrics {
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
}

impl Default for PortMetrics {
    fn default() -> Self {
        Self {
            rx_bytes_per_sec: 0.0,
            tx_bytes_per_sec: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct MetricsCollector {
    previous_counters: HashMap<String, PortCounters>,
    current_metrics: HashMap<String, PortMetrics>,
    last_collection: Option<Instant>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            previous_counters: HashMap::new(),
            current_metrics: HashMap::new(),
            last_collection: None,
        }
    }

    pub fn update(&mut self, adapters: &[AdapterInfo]) {
        let now = Instant::now();
        let time_delta = self
            .last_collection
            .map(|last| now.duration_since(last))
            .unwrap_or(Duration::from_secs(1));

        // Track current port keys to clean up stale entries
        let mut current_port_keys = std::collections::HashSet::new();

        for adapter in adapters {
            for port in &adapter.ports {
                let port_key = format!("{}:{}", adapter.name, port.port_number);
                current_port_keys.insert(port_key.clone());

                if let Some(prev_counters) = self.previous_counters.get(&port_key) {
                    let metrics = self.calculate_rates(prev_counters, &port.counters, time_delta);
                    self.current_metrics.insert(port_key.clone(), metrics);
                }

                // Store current counters for next calculation
                self.previous_counters
                    .insert(port_key, port.counters.clone());
            }
        }

        // Remove stale entries to prevent memory leaks
        self.previous_counters
            .retain(|key, _| current_port_keys.contains(key));
        self.current_metrics
            .retain(|key, _| current_port_keys.contains(key));

        self.last_collection = Some(now);
    }

    fn calculate_rates(
        &self,
        prev: &PortCounters,
        current: &PortCounters,
        time_delta: Duration,
    ) -> PortMetrics {
        let delta_seconds = time_delta.as_secs_f64();

        if delta_seconds == 0.0 {
            return PortMetrics::default();
        }

        let rx_bytes_delta = current.rx_bytes.saturating_sub(prev.rx_bytes);
        let tx_bytes_delta = current.tx_bytes.saturating_sub(prev.tx_bytes);

        PortMetrics {
            rx_bytes_per_sec: rx_bytes_delta as f64 / delta_seconds,
            tx_bytes_per_sec: tx_bytes_delta as f64 / delta_seconds,
        }
    }

    pub fn get_metrics(&self, adapter_name: &str, port_number: u16) -> Option<&PortMetrics> {
        let port_key = format!("{}:{}", adapter_name, port_number);
        self.current_metrics.get(&port_key)
    }
}
