use defmt::{write, Format, Formatter};

/// A four byte array type alias representing an IP address.
pub type IpAddress = [u8; 4];

/// A TCP/UDP network port.
pub type Port = u16;

pub(crate) type Socket = u8;

/// Defines the mode types that the ESP32 firmware can be put into when starting
/// a new client or server instance
#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TransportMode {
    Tcp = 0,
    Udp = 1,
    Tls = 2,
    UdpMulticast = 3,
    TlsBearSsl = 4,
}

/// Defines all possible TCP connection states for a client or server instance.
#[repr(u8)]
#[derive(PartialEq, PartialOrd, Debug)]
pub enum ConnectionState {
    Closed = 0,
    Listening = 1,
    SynSent = 2,
    SynReceived = 3,
    Established = 4,
    FinWait1 = 5,
    FinWait2 = 6,
    CloseWait = 7,
    Closing = 8,
    LastAck = 9,
    TimeWait = 10,
}

impl From<u8> for ConnectionState {
    fn from(state: u8) -> ConnectionState {
        match state {
            0 => ConnectionState::Closed,
            1 => ConnectionState::Listening,
            2 => ConnectionState::SynSent,
            3 => ConnectionState::SynReceived,
            4 => ConnectionState::Established,
            5 => ConnectionState::FinWait1,
            6 => ConnectionState::FinWait2,
            7 => ConnectionState::CloseWait,
            8 => ConnectionState::Closing,
            9 => ConnectionState::LastAck,
            10 => ConnectionState::TimeWait,
            _ => ConnectionState::Closed,
        }
    }
}

impl Format for ConnectionState {
    fn format(&self, fmt: Formatter) {
        match self {
            ConnectionState::Closed => write!(fmt, "Connection Closed"),
            ConnectionState::Listening => write!(fmt, "Connection Listening"),
            ConnectionState::SynSent => write!(fmt, "Connection SynSent"),
            ConnectionState::SynReceived => write!(fmt, "Connection SynRecieved"),
            ConnectionState::Established => write!(fmt, "Connection Established"),
            ConnectionState::FinWait1 => write!(fmt, "Connection FinWait1"),
            ConnectionState::FinWait2 => write!(fmt, "Connection FinWait2"),
            ConnectionState::CloseWait => write!(fmt, "Connection CloseWait"),
            ConnectionState::Closing => write!(fmt, "Connection Closing"),
            ConnectionState::LastAck => write!(fmt, "Connection LastAck"),
            ConnectionState::TimeWait => write!(fmt, "Connection TimeWait"),
        }
    }
}

/// Errors that occur due to issues involving communication over
/// WiFi network.
#[derive(PartialEq, Eq, Debug)]
pub enum NetworkError {
    /// Failed to resolve a hostname for the provided IP address.
    DnsResolveFailed,
    /// Failed to start up a new TCP/UDP client instance.
    StartClientFailed,
    /// Failed to stop an existing TCP/UDP client instance
    StopClientFailed,
}

impl Format for NetworkError {
    fn format(&self, fmt: Formatter) {
        match self {
            NetworkError::DnsResolveFailed => {
                write!(
                    fmt,
                    "Failed to resolve a hostname for the provided IP address"
                )
            }
            NetworkError::StartClientFailed => {
                write!(fmt, "Failed to start up a new TCP/UDP client instance")
            }
            NetworkError::StopClientFailed => {
                write!(fmt, "Failed to stop an existing TCP/UDP client instance")
            }
        }
    }
}
