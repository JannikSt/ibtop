pub(crate) mod fake;

use crate::types::*;

const MLX5_DATA_MULTIPLIER: u64 = 4; // mlx5 reports in 32-bit words

pub(crate) fn discover_adapters() -> Vec<AdapterInfo> {
    let mut adapters: Vec<AdapterInfo> = Vec::new();

    let path =
        std::env::var("INFINIBAND_PATH").unwrap_or_else(|_| "/sys/class/infiniband/".to_string());

    let Ok(entries) = std::fs::read_dir(path) else {
        return adapters;
    };

    for entry in entries.flatten() {
        if let Some(adapter_name) = entry.file_name().to_str().map(|s| s.to_string()) {
            let adapter = create_adapter_info(adapter_name, &entry.path());
            adapters.push(adapter);
        }
    }

    adapters.sort_by(|a, b| a.name.cmp(&b.name));

    adapters
}

fn create_adapter_info(adapter_name: String, adapter_path: &std::path::Path) -> AdapterInfo {
    let mut ports: Vec<PortInfo> = Vec::new();
    let ports_path = adapter_path.join("ports");

    if ports_path.exists() {
        if let Ok(ports_entries) = std::fs::read_dir(ports_path) {
            for port_entry in ports_entries.flatten() {
                if let Some(port_name) = port_entry.file_name().to_str() {
                    if let Ok(port_number) = port_name.parse::<u16>() {
                        let port_info = create_port_info(port_number, adapter_path);
                        ports.push(port_info);
                    }
                }
            }
        }
    }

    AdapterInfo {
        name: adapter_name,
        ports,
    }
}

fn create_port_info(port_number: u16, adapter_path: &std::path::Path) -> PortInfo {
    let port_path = adapter_path.join("ports").join(port_number.to_string());
    let state = read_port_state(&port_path);
    let rate = read_port_rate(&port_path);
    let counters = read_port_counters(&port_path);

    PortInfo {
        port_number,
        state,
        rate,
        counters,
    }
}

fn read_port_state(port_path: &std::path::Path) -> PortState {
    let state_path = port_path.join("state");
    let raw_state = std::fs::read_to_string(state_path)
        .unwrap_or_default()
        .trim()
        .to_string();

    // Handle format like "4: ACTIVE" or just "ACTIVE"
    let state_str = if let Some(colon_pos) = raw_state.find(':') {
        raw_state[colon_pos + 1..].trim()
    } else {
        raw_state.as_str()
    };

    state_str.parse::<PortState>().unwrap_or(PortState::Unknown)
}

fn read_port_rate(port_path: &std::path::Path) -> String {
    let rate_path = port_path.join("rate");
    let raw_rate = std::fs::read_to_string(rate_path)
        .unwrap_or_default()
        .trim()
        .to_string();

    // Just keeping the raw rate for now to prevent cluttering the UI
    // I know already that people will complain about this - sorry
    if let Some(paren_pos) = raw_rate.find('(') {
        raw_rate[..paren_pos].trim().to_string()
    } else {
        raw_rate
    }
}

fn read_port_counters(port_path: &std::path::Path) -> PortCounters {
    let counters_path = port_path.join("counters");
    let mut counters = PortCounters::default();

    if counters_path.exists() {
        counters.rx_bytes = read_counter_value(&counters_path, "port_rcv_data");
        counters.tx_bytes = read_counter_value(&counters_path, "port_xmit_data");
        counters.rx_packets = read_counter_value(&counters_path, "port_rcv_packets");
        counters.tx_packets = read_counter_value(&counters_path, "port_xmit_packets");
        counters.rx_errors = read_counter_value(&counters_path, "port_rcv_errors");
        counters.tx_errors = read_counter_value(&counters_path, "port_xmit_discards");
        counters.rx_dropped = read_counter_value(&counters_path, "port_rcv_constraint_errors");
    }

    counters
}

fn read_counter_value(counters_path: &std::path::Path, filename: &str) -> u64 {
    let value = std::fs::read_to_string(counters_path.join(filename))
        .map(|content| content.trim().parse().unwrap_or(0))
        .unwrap_or(0);

    if filename == "port_rcv_data" || filename == "port_xmit_data" {
        value * MLX5_DATA_MULTIPLIER
    } else {
        value
    }
}
