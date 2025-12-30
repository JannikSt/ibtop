//! Historical data storage with configurable time windows
//!
//! Provides ring buffer storage for time-series metrics data,
//! enabling sparklines and charts in the TUI.

#![allow(dead_code)] // Many methods are for future use or testing
#![allow(clippy::cast_precision_loss)] // Acceptable for metrics
#![allow(clippy::cast_possible_truncation)] // Acceptable for sparkline values
#![allow(clippy::cast_sign_loss)] // Values are always positive

use std::collections::HashMap;

/// Default history length (number of samples)
pub const DEFAULT_HISTORY_SIZE: usize = 120; // 30 seconds at 4 samples/sec

/// Ring buffer for storing historical values
#[derive(Debug, Clone)]
pub struct RingBuffer<T: Clone + Default> {
    data: Vec<T>,
    capacity: usize,
    write_pos: usize,
    len: usize,
}

impl<T: Clone + Default> RingBuffer<T> {
    /// Create a new ring buffer with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            capacity,
            write_pos: 0,
            len: 0,
        }
    }

    /// Push a new value into the buffer
    pub fn push(&mut self, value: T) {
        self.data[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        self.len = self.len.saturating_add(1).min(self.capacity);
    }

    /// Get the number of elements currently in the buffer
    pub fn len(&self) -> usize {
        self.len
    }

    /// Check if the buffer is empty
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the most recent value
    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            return None;
        }
        let idx = if self.write_pos == 0 {
            self.capacity - 1
        } else {
            self.write_pos - 1
        };
        Some(&self.data[idx])
    }

    /// Get values in chronological order (oldest to newest)
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let start = if self.len < self.capacity {
            0
        } else {
            self.write_pos
        };

        (0..self.len).map(move |i| {
            let idx = (start + i) % self.capacity;
            &self.data[idx]
        })
    }

    /// Get the last N values in chronological order
    pub fn last_n(&self, n: usize) -> impl Iterator<Item = &T> {
        let take_count = n.min(self.len);
        let skip_count = self.len.saturating_sub(take_count);
        self.iter().skip(skip_count)
    }

    /// Get values as a vector (chronological order)
    pub fn to_vec(&self) -> Vec<T> {
        self.iter().cloned().collect()
    }

    /// Clear all data
    pub fn clear(&mut self) {
        self.write_pos = 0;
        self.len = 0;
    }

    /// Get capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}

/// Historical metrics for a single port
#[derive(Debug, Clone)]
pub struct PortHistory {
    pub rx_bytes_per_sec: RingBuffer<f64>,
    pub tx_bytes_per_sec: RingBuffer<f64>,
    pub rx_packets_per_sec: RingBuffer<f64>,
    pub tx_packets_per_sec: RingBuffer<f64>,
    pub error_rate: RingBuffer<f64>,
}

impl PortHistory {
    /// Create a new port history with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_HISTORY_SIZE)
    }

    /// Create a new port history with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rx_bytes_per_sec: RingBuffer::new(capacity),
            tx_bytes_per_sec: RingBuffer::new(capacity),
            rx_packets_per_sec: RingBuffer::new(capacity),
            tx_packets_per_sec: RingBuffer::new(capacity),
            error_rate: RingBuffer::new(capacity),
        }
    }

    /// Record a new data point
    pub fn record(&mut self, rx_bps: f64, tx_bps: f64, rx_pps: f64, tx_pps: f64, errors: f64) {
        self.rx_bytes_per_sec.push(rx_bps);
        self.tx_bytes_per_sec.push(tx_bps);
        self.rx_packets_per_sec.push(rx_pps);
        self.tx_packets_per_sec.push(tx_pps);
        self.error_rate.push(errors);
    }

    /// Get sparkline data for RX throughput (last N samples, normalized to 0-1)
    pub fn rx_sparkline_data(&self, samples: usize) -> Vec<u64> {
        normalize_for_sparkline(self.rx_bytes_per_sec.last_n(samples))
    }

    /// Get sparkline data for TX throughput (last N samples, normalized to 0-1)
    pub fn tx_sparkline_data(&self, samples: usize) -> Vec<u64> {
        normalize_for_sparkline(self.tx_bytes_per_sec.last_n(samples))
    }

    /// Get combined RX+TX sparkline data
    pub fn combined_sparkline_data(&self, samples: usize) -> Vec<u64> {
        let rx: Vec<f64> = self.rx_bytes_per_sec.last_n(samples).copied().collect();
        let tx: Vec<f64> = self.tx_bytes_per_sec.last_n(samples).copied().collect();

        let combined: Vec<f64> = rx.iter().zip(tx.iter()).map(|(r, t)| r + t).collect();
        normalize_for_sparkline(combined.iter())
    }

    /// Get the peak throughput observed
    pub fn peak_throughput(&self) -> f64 {
        let rx_max = self
            .rx_bytes_per_sec
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        let tx_max = self
            .tx_bytes_per_sec
            .iter()
            .copied()
            .fold(0.0_f64, f64::max);
        rx_max + tx_max
    }

    /// Get average throughput
    pub fn avg_throughput(&self) -> f64 {
        if self.rx_bytes_per_sec.is_empty() {
            return 0.0;
        }
        let rx_sum: f64 = self.rx_bytes_per_sec.iter().sum();
        let tx_sum: f64 = self.tx_bytes_per_sec.iter().sum();
        (rx_sum + tx_sum) / self.rx_bytes_per_sec.len() as f64
    }
}

impl Default for PortHistory {
    fn default() -> Self {
        Self::new()
    }
}

/// Normalize values for sparkline display (0-7 range for 8-level sparkline)
fn normalize_for_sparkline<'a>(values: impl Iterator<Item = &'a f64>) -> Vec<u64> {
    let values: Vec<f64> = values.copied().collect();
    if values.is_empty() {
        return vec![];
    }

    let max = values.iter().copied().fold(0.0_f64, f64::max);
    if max <= 0.0 {
        return vec![0; values.len()];
    }

    values
        .iter()
        .map(|v| ((v / max) * 7.0).round() as u64)
        .collect()
}

/// Collection of all port histories
#[derive(Debug, Default)]
pub struct HistoryCollector {
    histories: HashMap<String, PortHistory>,
    capacity: usize,
}

impl HistoryCollector {
    /// Create a new history collector with default capacity
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_HISTORY_SIZE)
    }

    /// Create a new history collector with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            histories: HashMap::new(),
            capacity,
        }
    }

    /// Get or create history for a port
    pub fn get_or_create(&mut self, adapter: &str, port: u16) -> &mut PortHistory {
        let key = format!("{adapter}:{port}");
        self.histories
            .entry(key)
            .or_insert_with(|| PortHistory::with_capacity(self.capacity))
    }

    /// Get history for a port (read-only)
    pub fn get(&self, adapter: &str, port: u16) -> Option<&PortHistory> {
        let key = format!("{adapter}:{port}");
        self.histories.get(&key)
    }

    /// Record metrics for a port
    #[allow(clippy::too_many_arguments)]
    pub fn record(
        &mut self,
        adapter: &str,
        port: u16,
        rx_bps: f64,
        tx_bps: f64,
        rx_pps: f64,
        tx_pps: f64,
        errors: f64,
    ) {
        self.get_or_create(adapter, port)
            .record(rx_bps, tx_bps, rx_pps, tx_pps, errors);
    }

    /// Remove stale entries for ports that no longer exist
    pub fn retain_ports(&mut self, active_ports: &[(String, u16)]) {
        let active_keys: std::collections::HashSet<String> = active_ports
            .iter()
            .map(|(adapter, port)| format!("{adapter}:{port}"))
            .collect();

        self.histories.retain(|key, _| active_keys.contains(key));
    }

    /// Get all port keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.histories.keys()
    }

    /// Get total number of tracked ports
    pub fn port_count(&self) -> usize {
        self.histories.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ring_buffer_basic() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(3);

        assert!(buf.is_empty());
        assert_eq!(buf.len(), 0);

        buf.push(1);
        buf.push(2);
        buf.push(3);

        assert_eq!(buf.len(), 3);
        assert!(!buf.is_empty());

        let values: Vec<i32> = buf.iter().copied().collect();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_ring_buffer_overflow() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(3);

        buf.push(1);
        buf.push(2);
        buf.push(3);
        buf.push(4); // Overwrites 1

        assert_eq!(buf.len(), 3);

        let values: Vec<i32> = buf.iter().copied().collect();
        assert_eq!(values, vec![2, 3, 4]);
    }

    #[test]
    fn test_ring_buffer_last() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(5);

        assert!(buf.last().is_none());

        buf.push(10);
        assert_eq!(buf.last(), Some(&10));

        buf.push(20);
        buf.push(30);
        assert_eq!(buf.last(), Some(&30));
    }

    #[test]
    fn test_ring_buffer_last_n() {
        let mut buf: RingBuffer<i32> = RingBuffer::new(5);

        for i in 1..=10 {
            buf.push(i);
        }

        let last_3: Vec<i32> = buf.last_n(3).copied().collect();
        assert_eq!(last_3, vec![8, 9, 10]);

        let last_10: Vec<i32> = buf.last_n(10).copied().collect();
        assert_eq!(last_10, vec![6, 7, 8, 9, 10]);
    }

    #[test]
    fn test_port_history_record() {
        let mut history = PortHistory::with_capacity(10);

        history.record(1000.0, 500.0, 10.0, 5.0, 0.0);
        history.record(2000.0, 1000.0, 20.0, 10.0, 0.1);

        assert_eq!(history.rx_bytes_per_sec.len(), 2);
        assert_eq!(history.tx_bytes_per_sec.len(), 2);
    }

    #[test]
    fn test_normalize_for_sparkline() {
        let values = vec![0.0, 50.0, 100.0, 25.0, 75.0];
        let normalized = normalize_for_sparkline(values.iter());

        assert_eq!(normalized.len(), 5);
        assert_eq!(normalized[0], 0); // 0%
        assert_eq!(normalized[2], 7); // 100%
    }

    #[test]
    fn test_normalize_empty() {
        let values: Vec<f64> = vec![];
        let normalized = normalize_for_sparkline(values.iter());
        assert!(normalized.is_empty());
    }

    #[test]
    fn test_normalize_all_zero() {
        let values = vec![0.0, 0.0, 0.0];
        let normalized = normalize_for_sparkline(values.iter());
        assert_eq!(normalized, vec![0, 0, 0]);
    }

    #[test]
    fn test_history_collector_basic() {
        let mut collector = HistoryCollector::new();

        collector.record("mlx5_0", 1, 1000.0, 500.0, 10.0, 5.0, 0.0);
        collector.record("mlx5_0", 2, 2000.0, 1000.0, 20.0, 10.0, 0.1);

        assert_eq!(collector.port_count(), 2);
        assert!(collector.get("mlx5_0", 1).is_some());
        assert!(collector.get("mlx5_0", 2).is_some());
        assert!(collector.get("mlx5_1", 1).is_none());
    }

    #[test]
    fn test_history_collector_retain() {
        let mut collector = HistoryCollector::new();

        collector.record("mlx5_0", 1, 1000.0, 500.0, 10.0, 5.0, 0.0);
        collector.record("mlx5_0", 2, 2000.0, 1000.0, 20.0, 10.0, 0.1);
        collector.record("mlx5_1", 1, 3000.0, 1500.0, 30.0, 15.0, 0.0);

        assert_eq!(collector.port_count(), 3);

        // Retain only mlx5_0:1 and mlx5_1:1
        collector.retain_ports(&[("mlx5_0".to_string(), 1), ("mlx5_1".to_string(), 1)]);

        assert_eq!(collector.port_count(), 2);
        assert!(collector.get("mlx5_0", 1).is_some());
        assert!(collector.get("mlx5_0", 2).is_none());
        assert!(collector.get("mlx5_1", 1).is_some());
    }

    #[test]
    fn test_port_history_peak_throughput() {
        let mut history = PortHistory::with_capacity(10);

        history.record(1000.0, 500.0, 10.0, 5.0, 0.0);
        history.record(2000.0, 1500.0, 20.0, 10.0, 0.0);
        history.record(500.0, 250.0, 5.0, 2.0, 0.0);

        // Peak is 2000 + 1500 = 3500
        assert!((history.peak_throughput() - 3500.0).abs() < 0.001);
    }

    #[test]
    fn test_port_history_avg_throughput() {
        let mut history = PortHistory::with_capacity(10);

        history.record(1000.0, 500.0, 10.0, 5.0, 0.0);
        history.record(2000.0, 1000.0, 20.0, 10.0, 0.0);

        // Avg is ((1000+500) + (2000+1000)) / 2 = 4500 / 2 = 2250
        assert!((history.avg_throughput() - 2250.0).abs() < 0.001);
    }
}
