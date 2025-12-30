use std::collections::HashMap;
use std::time::{Duration, Instant};

use crate::history::HistoryCollector;
use crate::types::{AdapterInfo, PortCounters};

#[derive(Debug, Clone)]
pub struct PortMetrics {
    pub rx_bytes_per_sec: f64,
    pub tx_bytes_per_sec: f64,
    pub rx_packets_per_sec: f64,
    pub tx_packets_per_sec: f64,
    pub error_rate: f64,
}

impl Default for PortMetrics {
    fn default() -> Self {
        Self {
            rx_bytes_per_sec: 0.0,
            tx_bytes_per_sec: 0.0,
            rx_packets_per_sec: 0.0,
            tx_packets_per_sec: 0.0,
            error_rate: 0.0,
        }
    }
}

#[derive(Debug)]
pub struct MetricsCollector {
    previous_counters: HashMap<String, PortCounters>,
    current_metrics: HashMap<String, PortMetrics>,
    last_collection: Option<Instant>,
    pub history: HistoryCollector,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            previous_counters: HashMap::new(),
            current_metrics: HashMap::new(),
            last_collection: None,
            history: HistoryCollector::new(),
        }
    }

    pub fn update(&mut self, adapters: &[AdapterInfo]) {
        let now = Instant::now();
        let time_delta = self
            .last_collection
            .map_or(Duration::from_secs(1), |last| now.duration_since(last));

        // Track current port keys to clean up stale entries
        let mut current_port_keys = std::collections::HashSet::new();
        let mut active_ports = Vec::new();

        for adapter in adapters {
            for port in &adapter.ports {
                let port_key = format!("{}:{}", adapter.name, port.port_number);
                current_port_keys.insert(port_key.clone());
                active_ports.push((adapter.name.clone(), port.port_number));

                if let Some(prev_counters) = self.previous_counters.get(&port_key) {
                    let metrics = Self::calculate_rates(prev_counters, &port.counters, time_delta);

                    // Record to history
                    self.history.record(
                        &adapter.name,
                        port.port_number,
                        metrics.rx_bytes_per_sec,
                        metrics.tx_bytes_per_sec,
                        metrics.rx_packets_per_sec,
                        metrics.tx_packets_per_sec,
                        metrics.error_rate,
                    );

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
        self.history.retain_ports(&active_ports);

        self.last_collection = Some(now);
    }

    #[allow(clippy::cast_precision_loss)]
    fn calculate_rates(
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
        let rx_packets_delta = current.rx_packets.saturating_sub(prev.rx_packets);
        let tx_packets_delta = current.tx_packets.saturating_sub(prev.tx_packets);

        let prev_errors = prev.rx_errors + prev.tx_errors;
        let current_errors = current.rx_errors + current.tx_errors;
        let error_delta = current_errors.saturating_sub(prev_errors);

        PortMetrics {
            rx_bytes_per_sec: rx_bytes_delta as f64 / delta_seconds,
            tx_bytes_per_sec: tx_bytes_delta as f64 / delta_seconds,
            rx_packets_per_sec: rx_packets_delta as f64 / delta_seconds,
            tx_packets_per_sec: tx_packets_delta as f64 / delta_seconds,
            error_rate: error_delta as f64 / delta_seconds,
        }
    }

    pub fn get_metrics(&self, adapter_name: &str, port_number: u16) -> Option<&PortMetrics> {
        let port_key = format!("{adapter_name}:{port_number}");
        self.current_metrics.get(&port_key)
    }

    /// Get historical data for a port
    pub fn get_history(
        &self,
        adapter_name: &str,
        port_number: u16,
    ) -> Option<&crate::history::PortHistory> {
        self.history.get(adapter_name, port_number)
    }
}
