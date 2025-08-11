use std::str::FromStr;

pub(crate) enum PortState {
    ACTIVE,
    DOWN,
}

impl FromStr for PortState {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ACTIVE" => Ok(PortState::ACTIVE),
            "DOWN" => Ok(PortState::DOWN),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub(crate) struct AdapterInfo {
    pub(crate) name: String,
    pub(crate) ports: Vec<PortInfo>,
}

#[derive(Debug, Default)]
pub(crate) struct PortInfo {
    pub(crate) port_number: u16,
    pub(crate) state: String,
    pub(crate) counters: PortCounters,
}

#[derive(Debug, Default)]
pub(crate) struct PortCounters {
    pub(crate) rx_bytes: u64,
    pub(crate) tx_bytes: u64,
    pub(crate) rx_packets: u64,
    pub(crate) tx_packets: u64,
    pub(crate) rx_errors: u64,
    pub(crate) tx_errors: u64,
    pub(crate) rx_dropped: u64,
}
