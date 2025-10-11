use std::fmt;

/// Estados posibles de una conexión de red
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Established,
    Listen,
    TimeWait,
    CloseWait,
    SynSent,
    SynReceived,
    FinWait1,
    FinWait2,
    Closing,
    LastAck,
    Closed,
    Unknown,
}

impl ConnectionState {
    pub fn from_str(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "ESTABLISHED" => Self::Established,
            "LISTEN" => Self::Listen,
            "TIME_WAIT" => Self::TimeWait,
            "CLOSE_WAIT" => Self::CloseWait,
            "SYN_SENT" => Self::SynSent,
            "SYN_RECV" | "SYN_RECEIVED" => Self::SynReceived,
            "FIN_WAIT1" => Self::FinWait1,
            "FIN_WAIT2" => Self::FinWait2,
            "CLOSING" => Self::Closing,
            "LAST_ACK" => Self::LastAck,
            "CLOSED" => Self::Closed,
            _ => Self::Unknown,
        }
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Established => write!(f, "ESTABLISHED"),
            Self::Listen => write!(f, "LISTEN"),
            Self::TimeWait => write!(f, "TIME_WAIT"),
            Self::CloseWait => write!(f, "CLOSE_WAIT"),
            Self::SynSent => write!(f, "SYN_SENT"),
            Self::SynReceived => write!(f, "SYN_RECV"),
            Self::FinWait1 => write!(f, "FIN_WAIT1"),
            Self::FinWait2 => write!(f, "FIN_WAIT2"),
            Self::Closing => write!(f, "CLOSING"),
            Self::LastAck => write!(f, "LAST_ACK"),
            Self::Closed => write!(f, "CLOSED"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}
