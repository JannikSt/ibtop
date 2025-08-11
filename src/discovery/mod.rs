pub(crate) mod fake;

use crate::types::*;

pub(crate) fn discover_adapters() -> Vec<AdapterInfo> {
    let mut adapters: Vec<AdapterInfo> = Vec::new();

    let path =
        std::env::var("INFINIBAND_PATH").unwrap_or_else(|_| "/sys/class/infiniband/".to_string());

    let Ok(entries) = std::fs::read_dir(path) else {
        return adapters;
    };

    for entry in entries {
        if let Ok(entry) = entry {
            if let Some(adapter_name) = entry.file_name().to_str().map(|s| s.to_string()) {
                let adapter = create_adapter_info(adapter_name, &entry.path());
                adapters.push(adapter);
            }
        }
    }
    adapters
}

fn create_adapter_info(adapter_name: String, adapter_path: &std::path::Path) -> AdapterInfo {
    let mut ports: Vec<PortInfo> = Vec::new();
    let ports_path = adapter_path.join("ports");

    if ports_path.exists() {
        if let Ok(ports_entries) = std::fs::read_dir(ports_path) {
            for port_entry in ports_entries {
                if let Ok(port_entry) = port_entry {
                    if let Some(port_name) = port_entry.file_name().to_str() {
                        if let Ok(port_number) = port_name.parse::<u16>() {
                            let port_info = create_port_info(port_number, adapter_path);
                            ports.push(port_info);
                        }
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
    let counters = read_port_counters(&port_path);

    PortInfo {
        port_number,
        state,
        counters,
    }
}

fn read_port_state(port_path: &std::path::Path) -> String {
    let state_path = port_path.join("state");
    std::fs::read_to_string(state_path)
        .unwrap_or_default()
        .trim()
        .to_string()
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
    std::fs::read_to_string(counters_path.join(filename))
        .map(|content| content.trim().parse().unwrap_or(0))
        .unwrap_or(0)
}
