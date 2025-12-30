//! Sophisticated traffic simulation for demo and testing
//!
//! Generates realistic `InfiniBand` traffic patterns including:
//! - Burst patterns (MPI collective operations)
//! - Steady streaming (RDMA transfers)
//! - Wave patterns (periodic workloads)
//! - Idle with occasional spikes (interactive)
//! - Congestion patterns (network contention)

#![allow(dead_code)] // TrafficPattern methods are for extensibility
#![allow(clippy::similar_names)] // rx/tx pairs are intentionally similar
#![allow(clippy::cast_precision_loss)] // Acceptable for metrics

use crate::types::{AdapterInfo, PortCounters, PortInfo, PortState};
use std::f64::consts::PI;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

/// Traffic pattern types for simulation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrafficPattern {
    /// MPI collective operations - periodic bursts with gaps
    Burst,
    /// Steady RDMA transfers - consistent high throughput
    Steady,
    /// Periodic workload - sine wave pattern
    Wave,
    /// Interactive/idle - low baseline with random spikes
    Interactive,
    /// Network congestion - high with periodic drops
    Congestion,
}

impl TrafficPattern {
    /// Returns all available patterns for cycling
    pub const fn all() -> &'static [TrafficPattern] {
        &[
            TrafficPattern::Burst,
            TrafficPattern::Steady,
            TrafficPattern::Wave,
            TrafficPattern::Interactive,
            TrafficPattern::Congestion,
        ]
    }

    /// Human-readable name for the pattern
    pub const fn name(self) -> &'static str {
        match self {
            TrafficPattern::Burst => "MPI Collective",
            TrafficPattern::Steady => "RDMA Stream",
            TrafficPattern::Wave => "Periodic Load",
            TrafficPattern::Interactive => "Interactive",
            TrafficPattern::Congestion => "Congested",
        }
    }
}

/// Simulated port configuration
struct SimulatedPort {
    adapter_name: &'static str,
    port_number: u16,
    state: PortState,
    rate: &'static str,
    pattern: TrafficPattern,
    /// Base throughput in bytes/sec (for 100% utilization reference)
    max_throughput: u64,
    /// RX/TX ratio (0.5 = balanced, >0.5 = more RX)
    rx_tx_ratio: f64,
}

/// Configuration for simulation
const SIMULATED_PORTS: &[SimulatedPort] = &[
    SimulatedPort {
        adapter_name: "mlx5_0",
        port_number: 1,
        state: PortState::Active,
        rate: "100 Gb/sec (4X EDR)",
        pattern: TrafficPattern::Burst,
        max_throughput: 12_500_000_000, // 100 Gbps = 12.5 GB/s
        rx_tx_ratio: 0.55,
    },
    SimulatedPort {
        adapter_name: "mlx5_0",
        port_number: 2,
        state: PortState::Down,
        rate: "100 Gb/sec (4X EDR)",
        pattern: TrafficPattern::Steady,
        max_throughput: 12_500_000_000,
        rx_tx_ratio: 0.5,
    },
    SimulatedPort {
        adapter_name: "mlx5_1",
        port_number: 1,
        state: PortState::Active,
        rate: "200 Gb/sec (4X HDR)",
        pattern: TrafficPattern::Steady,
        max_throughput: 25_000_000_000, // 200 Gbps = 25 GB/s
        rx_tx_ratio: 0.48,
    },
    SimulatedPort {
        adapter_name: "mlx5_2",
        port_number: 1,
        state: PortState::Active,
        rate: "400 Gb/sec (4X NDR)",
        pattern: TrafficPattern::Wave,
        max_throughput: 50_000_000_000, // 400 Gbps = 50 GB/s
        rx_tx_ratio: 0.52,
    },
    SimulatedPort {
        adapter_name: "mlx5_bond0",
        port_number: 1,
        state: PortState::Active,
        rate: "200 Gb/sec (Bonded)",
        pattern: TrafficPattern::Interactive,
        max_throughput: 25_000_000_000,
        rx_tx_ratio: 0.7, // More RX (receiving results)
    },
    SimulatedPort {
        adapter_name: "mlx5_bond0",
        port_number: 2,
        state: PortState::Active,
        rate: "200 Gb/sec (Bonded)",
        pattern: TrafficPattern::Congestion,
        max_throughput: 25_000_000_000,
        rx_tx_ratio: 0.3, // More TX (sending data)
    },
];

/// Global simulation state
static SIM_START: AtomicU64 = AtomicU64::new(0);
static CALL_COUNT: AtomicU64 = AtomicU64::new(0);

// Cumulative counters for each port (indexed by port config index)
static COUNTERS: [PortCounterState; 6] = [
    PortCounterState::new(),
    PortCounterState::new(),
    PortCounterState::new(),
    PortCounterState::new(),
    PortCounterState::new(),
    PortCounterState::new(),
];

struct PortCounterState {
    rx_bytes: AtomicU64,
    tx_bytes: AtomicU64,
    rx_packets: AtomicU64,
    tx_packets: AtomicU64,
    rx_errors: AtomicU64,
    tx_errors: AtomicU64,
    rx_dropped: AtomicU64,
}

impl PortCounterState {
    const fn new() -> Self {
        Self {
            rx_bytes: AtomicU64::new(0),
            tx_bytes: AtomicU64::new(0),
            rx_packets: AtomicU64::new(0),
            tx_packets: AtomicU64::new(0),
            rx_errors: AtomicU64::new(0),
            tx_errors: AtomicU64::new(0),
            rx_dropped: AtomicU64::new(0),
        }
    }
}

/// Initialize simulation with starting timestamp
#[allow(clippy::cast_possible_truncation)]
fn ensure_initialized() -> f64 {
    let now_nanos = Instant::now().elapsed().as_nanos() as u64;
    let _ = SIM_START.compare_exchange(0, now_nanos.max(1), Ordering::SeqCst, Ordering::SeqCst);

    let count = CALL_COUNT.fetch_add(1, Ordering::Relaxed);
    // Use call count as time proxy (each call is ~250ms in real app)
    count as f64 * 0.25
}

/// Calculate traffic multiplier based on pattern and time
fn calculate_utilization(pattern: TrafficPattern, time_secs: f64) -> f64 {
    match pattern {
        TrafficPattern::Burst => {
            // MPI collective pattern: high bursts with gaps
            // 2 second cycle: 0.5s burst at 90%, 1.5s at 10%
            let cycle_pos = time_secs % 2.0;
            if cycle_pos < 0.5 {
                0.85 + random_noise() * 0.1
            } else {
                0.05 + random_noise() * 0.1
            }
        }
        TrafficPattern::Steady => {
            // Consistent high throughput with minor variations
            0.75 + random_noise() * 0.15
        }
        TrafficPattern::Wave => {
            // Sine wave pattern with 10 second period
            let base = 0.5 + 0.4 * (time_secs * 2.0 * PI / 10.0).sin();
            base + random_noise() * 0.1
        }
        TrafficPattern::Interactive => {
            // Low baseline with occasional spikes
            let spike = if random_noise() > 0.92 { 0.7 } else { 0.0 };
            0.05 + random_noise() * 0.08 + spike
        }
        TrafficPattern::Congestion => {
            // High utilization with periodic drops (packet loss)
            let drop = if random_noise() > 0.85 { -0.3 } else { 0.0 };
            (0.9 + random_noise() * 0.1 + drop).max(0.3)
        }
    }
}

/// Generate random noise in [0, 1)
fn random_noise() -> f64 {
    rand::random::<f64>()
}

/// Average packet size based on pattern (affects packet/byte ratio)
fn avg_packet_size(pattern: TrafficPattern) -> u64 {
    match pattern {
        TrafficPattern::Burst => 4096,       // Large MPI messages
        TrafficPattern::Steady => 65536,     // Max MTU RDMA
        TrafficPattern::Wave => 8192,        // Mixed workload
        TrafficPattern::Interactive => 512,  // Small messages
        TrafficPattern::Congestion => 32768, // Large but congested
    }
}

/// Calculate error rate based on pattern
fn error_probability(pattern: TrafficPattern) -> f64 {
    match pattern {
        TrafficPattern::Burst | TrafficPattern::Wave => 0.0001,
        TrafficPattern::Steady => 0.00005,
        TrafficPattern::Interactive => 0.0002,
        TrafficPattern::Congestion => 0.002, // Higher errors due to congestion
    }
}

/// Generate fake adapters with sophisticated traffic simulation
pub fn generate_fake_adapters() -> Vec<AdapterInfo> {
    let time_secs = ensure_initialized();

    // Group ports by adapter
    let mut adapter_map: std::collections::HashMap<&str, Vec<PortInfo>> =
        std::collections::HashMap::new();

    for (idx, port_config) in SIMULATED_PORTS.iter().enumerate() {
        let counters = if port_config.state == PortState::Down {
            PortCounters::default()
        } else {
            generate_counters(idx, port_config, time_secs)
        };

        let port_info = PortInfo {
            port_number: port_config.port_number,
            state: port_config.state,
            rate: port_config.rate.to_string(),
            counters,
        };

        adapter_map
            .entry(port_config.adapter_name)
            .or_default()
            .push(port_info);
    }

    // Convert to sorted vector of adapters
    let mut adapters: Vec<AdapterInfo> = adapter_map
        .into_iter()
        .map(|(name, ports)| AdapterInfo {
            name: name.to_string(),
            ports,
        })
        .collect();

    adapters.sort_by(|a, b| a.name.cmp(&b.name));
    adapters
}

#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn generate_counters(idx: usize, config: &SimulatedPort, time_secs: f64) -> PortCounters {
    let utilization = calculate_utilization(config.pattern, time_secs);

    // Calculate bytes transferred in this interval (~250ms)
    let interval_secs = 0.25;
    let total_bytes = (config.max_throughput as f64 * utilization * interval_secs) as u64;

    let rx_bytes = (total_bytes as f64 * config.rx_tx_ratio) as u64;
    let tx_bytes = total_bytes - rx_bytes;

    let packet_size = avg_packet_size(config.pattern);
    let rx_packets = rx_bytes / packet_size;
    let tx_packets = tx_bytes / packet_size;

    // Error generation
    let error_prob = error_probability(config.pattern);
    let rx_errors = if random_noise() < error_prob {
        (random_noise() * 3.0) as u64
    } else {
        0
    };
    let tx_errors = if random_noise() < error_prob {
        (random_noise() * 2.0) as u64
    } else {
        0
    };
    let rx_dropped = if config.pattern == TrafficPattern::Congestion && random_noise() < 0.01 {
        (random_noise() * 5.0) as u64
    } else {
        0
    };

    // Update cumulative counters
    let counter = &COUNTERS[idx];
    let total_rx = counter.rx_bytes.fetch_add(rx_bytes, Ordering::Relaxed) + rx_bytes;
    let total_tx = counter.tx_bytes.fetch_add(tx_bytes, Ordering::Relaxed) + tx_bytes;
    let total_rx_pkt = counter.rx_packets.fetch_add(rx_packets, Ordering::Relaxed) + rx_packets;
    let total_tx_pkt = counter.tx_packets.fetch_add(tx_packets, Ordering::Relaxed) + tx_packets;
    let total_rx_err = counter.rx_errors.fetch_add(rx_errors, Ordering::Relaxed) + rx_errors;
    let total_tx_err = counter.tx_errors.fetch_add(tx_errors, Ordering::Relaxed) + tx_errors;
    let total_dropped = counter.rx_dropped.fetch_add(rx_dropped, Ordering::Relaxed) + rx_dropped;

    PortCounters {
        rx_bytes: total_rx,
        tx_bytes: total_tx,
        rx_packets: total_rx_pkt,
        tx_packets: total_tx_pkt,
        rx_errors: total_rx_err,
        tx_errors: total_tx_err,
        rx_dropped: total_dropped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_traffic_patterns_all() {
        let patterns = TrafficPattern::all();
        assert_eq!(patterns.len(), 5);
    }

    #[test]
    fn test_pattern_names() {
        assert_eq!(TrafficPattern::Burst.name(), "MPI Collective");
        assert_eq!(TrafficPattern::Steady.name(), "RDMA Stream");
        assert_eq!(TrafficPattern::Wave.name(), "Periodic Load");
        assert_eq!(TrafficPattern::Interactive.name(), "Interactive");
        assert_eq!(TrafficPattern::Congestion.name(), "Congested");
    }

    #[test]
    fn test_utilization_bounds() {
        for pattern in TrafficPattern::all() {
            for t in 0..100 {
                let util = calculate_utilization(*pattern, f64::from(t) * 0.1);
                assert!(
                    util >= 0.0 && util <= 1.0,
                    "Pattern {pattern:?} at t={t}: util={util}"
                );
            }
        }
    }

    #[test]
    fn test_generate_fake_adapters() {
        let adapters = generate_fake_adapters();
        assert!(!adapters.is_empty());

        // Should have multiple adapters
        assert!(adapters.len() >= 2);

        // Check that active ports have non-zero counters after a few calls
        generate_fake_adapters();
        generate_fake_adapters();
        let adapters = generate_fake_adapters();

        for adapter in &adapters {
            for port in &adapter.ports {
                if port.state == PortState::Active {
                    assert!(port.counters.rx_bytes > 0 || port.counters.tx_bytes > 0);
                }
            }
        }
    }

    #[test]
    fn test_down_port_has_zero_counters() {
        // Find a down port
        let adapters = generate_fake_adapters();
        for adapter in adapters {
            for port in adapter.ports {
                if port.state == PortState::Down {
                    assert_eq!(port.counters.rx_bytes, 0);
                    assert_eq!(port.counters.tx_bytes, 0);
                }
            }
        }
    }

    #[test]
    fn test_avg_packet_sizes() {
        // Verify packet sizes are reasonable
        assert!(
            avg_packet_size(TrafficPattern::Interactive) < avg_packet_size(TrafficPattern::Steady)
        );
        assert!(avg_packet_size(TrafficPattern::Burst) < avg_packet_size(TrafficPattern::Steady));
    }
}
