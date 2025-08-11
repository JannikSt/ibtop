use crate::types::{AdapterInfo, PortCounters, PortInfo};
use std::sync::atomic::{AtomicU64, Ordering};

static BASE_RX_BYTES_0: AtomicU64 = AtomicU64::new(1234567890);
static BASE_TX_BYTES_0: AtomicU64 = AtomicU64::new(987654321);
static BASE_RX_PACKETS_0: AtomicU64 = AtomicU64::new(1000000);
static BASE_TX_PACKETS_0: AtomicU64 = AtomicU64::new(950000);
static BASE_RX_ERRORS_0: AtomicU64 = AtomicU64::new(12);
static BASE_TX_ERRORS_0: AtomicU64 = AtomicU64::new(5);

static BASE_RX_BYTES_1: AtomicU64 = AtomicU64::new(5555555555);
static BASE_TX_BYTES_1: AtomicU64 = AtomicU64::new(4444444444);
static BASE_RX_PACKETS_1: AtomicU64 = AtomicU64::new(2500000);
static BASE_TX_PACKETS_1: AtomicU64 = AtomicU64::new(2400000);
static BASE_RX_ERRORS_1: AtomicU64 = AtomicU64::new(8);
static BASE_TX_ERRORS_1: AtomicU64 = AtomicU64::new(3);
static BASE_RX_DROPPED_1: AtomicU64 = AtomicU64::new(1);

pub fn generate_fake_adapters() -> Vec<AdapterInfo> {
    let rx_bytes_0 = BASE_RX_BYTES_0.fetch_add(rand::random::<u64>() % 100000, Ordering::Relaxed);
    let tx_bytes_0 = BASE_TX_BYTES_0.fetch_add(rand::random::<u64>() % 80000, Ordering::Relaxed);
    let rx_packets_0 = BASE_RX_PACKETS_0.fetch_add(rand::random::<u64>() % 1000, Ordering::Relaxed);
    let tx_packets_0 = BASE_TX_PACKETS_0.fetch_add(rand::random::<u64>() % 900, Ordering::Relaxed);
    let rx_errors_0 = BASE_RX_ERRORS_0.fetch_add(
        if rand::random::<u8>() % 100 < 5 { 1 } else { 0 },
        Ordering::Relaxed,
    );
    let tx_errors_0 = BASE_TX_ERRORS_0.fetch_add(
        if rand::random::<u8>() % 100 < 3 { 1 } else { 0 },
        Ordering::Relaxed,
    );

    let rx_bytes_1 = BASE_RX_BYTES_1.fetch_add(rand::random::<u64>() % 150000, Ordering::Relaxed);
    let tx_bytes_1 = BASE_TX_BYTES_1.fetch_add(rand::random::<u64>() % 120000, Ordering::Relaxed);
    let rx_packets_1 = BASE_RX_PACKETS_1.fetch_add(rand::random::<u64>() % 1500, Ordering::Relaxed);
    let tx_packets_1 = BASE_TX_PACKETS_1.fetch_add(rand::random::<u64>() % 1400, Ordering::Relaxed);
    let rx_errors_1 = BASE_RX_ERRORS_1.fetch_add(
        if rand::random::<u8>() % 100 < 4 { 1 } else { 0 },
        Ordering::Relaxed,
    );
    let tx_errors_1 = BASE_TX_ERRORS_1.fetch_add(
        if rand::random::<u8>() % 100 < 2 { 1 } else { 0 },
        Ordering::Relaxed,
    );
    let rx_dropped_1 = BASE_RX_DROPPED_1.fetch_add(
        if rand::random::<u8>() % 100 < 1 { 1 } else { 0 },
        Ordering::Relaxed,
    );

    vec![
        AdapterInfo {
            name: "mlx5_0".to_string(),
            ports: vec![
                PortInfo {
                    port_number: 1,
                    state: "ACTIVE".to_string(),
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
                    state: "DOWN".to_string(),
                    counters: PortCounters::default(),
                },
            ],
        },
        AdapterInfo {
            name: "mlx5_1".to_string(),
            ports: vec![PortInfo {
                port_number: 1,
                state: "ACTIVE".to_string(),
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
