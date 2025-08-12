use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(crate) enum PortState {
    Active,
    Down,
    #[default]
    Unknown,
}
impl Display for PortState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortState::Active => write!(f, "ACTIVE"),
            PortState::Down => write!(f, "DOWN"),
            PortState::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl FromStr for PortState {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim() {
            "ACTIVE" => Ok(PortState::Active),
            "DOWN" => Ok(PortState::Down),
            _ => Ok(PortState::Unknown),
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
    pub(crate) state: PortState,
    pub(crate) rate: String,
    pub(crate) counters: PortCounters,
}

#[derive(Debug, Default, Clone)]
pub(crate) struct PortCounters {
    pub(crate) rx_bytes: u64,
    pub(crate) tx_bytes: u64,
    pub(crate) rx_packets: u64,
    pub(crate) tx_packets: u64,
    pub(crate) rx_errors: u64,
    pub(crate) tx_errors: u64,
    pub(crate) rx_dropped: u64,
}
