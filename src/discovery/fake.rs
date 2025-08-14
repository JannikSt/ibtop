use crate::types::{AdapterInfo, PortCounters, PortInfo};
use std::sync::atomic::{AtomicU64, Ordering};

static BASE_RX_BYTES_0: AtomicU64 = AtomicU64::new(1_234_567_890);
static BASE_TX_BYTES_0: AtomicU64 = AtomicU64::new(987_654_321);
static BASE_RX_PACKETS_0: AtomicU64 = AtomicU64::new(1_000_000);
static BASE_TX_PACKETS_0: AtomicU64 = AtomicU64::new(950_000);
static BASE_RX_ERRORS_0: AtomicU64 = AtomicU64::new(12);
static BASE_TX_ERRORS_0: AtomicU64 = AtomicU64::new(5);

static BASE_RX_BYTES_1: AtomicU64 = AtomicU64::new(5_555_555_555);
static BASE_TX_BYTES_1: AtomicU64 = AtomicU64::new(4_444_444_444);
static BASE_RX_PACKETS_1: AtomicU64 = AtomicU64::new(2_500_000);
static BASE_TX_PACKETS_1: AtomicU64 = AtomicU64::new(2_400_000);
static BASE_RX_ERRORS_1: AtomicU64 = AtomicU64::new(8);
static BASE_TX_ERRORS_1: AtomicU64 = AtomicU64::new(3);
static BASE_RX_DROPPED_1: AtomicU64 = AtomicU64::new(1);

pub fn generate_fake_adapters() -> Vec<AdapterInfo> {
    let rx_bytes_0 = BASE_RX_BYTES_0.fetch_add(rand::random::<u64>() % 100_000, Ordering::Relaxed);
    let tx_bytes_0 = BASE_TX_BYTES_0.fetch_add(rand::random::<u64>() % 80_000, Ordering::Relaxed);
    let rx_packets_0 = BASE_RX_PACKETS_0.fetch_add(rand::random::<u64>() % 1000, Ordering::Relaxed);
    let tx_packets_0 = BASE_TX_PACKETS_0.fetch_add(rand::random::<u64>() % 900, Ordering::Relaxed);
    let rx_errors_0 =
        BASE_RX_ERRORS_0.fetch_add(u64::from(rand::random::<u8>() % 100 < 5), Ordering::Relaxed);
    let tx_errors_0 =
        BASE_TX_ERRORS_0.fetch_add(u64::from(rand::random::<u8>() % 100 < 3), Ordering::Relaxed);

    let rx_bytes_1 = BASE_RX_BYTES_1.fetch_add(rand::random::<u64>() % 150_000, Ordering::Relaxed);
    let tx_bytes_1 = BASE_TX_BYTES_1.fetch_add(rand::random::<u64>() % 120_000, Ordering::Relaxed);
    let rx_packets_1 = BASE_RX_PACKETS_1.fetch_add(rand::random::<u64>() % 1500, Ordering::Relaxed);
    let tx_packets_1 = BASE_TX_PACKETS_1.fetch_add(rand::random::<u64>() % 1400, Ordering::Relaxed);
    let rx_errors_1 =
        BASE_RX_ERRORS_1.fetch_add(u64::from(rand::random::<u8>() % 100 < 4), Ordering::Relaxed);
    let tx_errors_1 =
        BASE_TX_ERRORS_1.fetch_add(u64::from(rand::random::<u8>() % 100 < 2), Ordering::Relaxed);
    let rx_dropped_1 =
        BASE_RX_DROPPED_1.fetch_add(u64::from(rand::random::<u8>() % 100 < 1), Ordering::Relaxed);

    vec![
        AdapterInfo {
            name: "mlx5_0".to_string(),
            ports: vec![
                PortInfo {
                    port_number: 1,
                    state: crate::types::PortState::Active,
                    rate: "100 Gb/sec (4X EDR)".to_string(),
                    counters: PortCounters {
                        rx_bytes: rx_bytes_0,
                        tx_bytes: tx_bytes_0,
                        rx_packets: rx_packets_0,
                        tx_packets: tx_packets_0,
                        rx_errors: rx_errors_0,
                        tx_errors: tx_errors_0,
                        rx_dropped: 0,
                    },
                },
                PortInfo {
                    port_number: 2,
                    state: crate::types::PortState::Down,
                    rate: "100 Gb/sec (4X EDR)".to_string(),
                    counters: PortCounters::default(),
                },
            ],
        },
        AdapterInfo {
            name: "mlx5_1".to_string(),
            ports: vec![PortInfo {
                port_number: 1,
                state: crate::types::PortState::Active,
                rate: "200 Gb/sec (4X HDR)".to_string(),
                counters: PortCounters {
                    rx_bytes: rx_bytes_1,
                    tx_bytes: tx_bytes_1,
                    rx_packets: rx_packets_1,
                    tx_packets: tx_packets_1,
                    rx_errors: rx_errors_1,
                    tx_errors: tx_errors_1,
                    rx_dropped: rx_dropped_1,
                },
            }],
        },
    ]
}
