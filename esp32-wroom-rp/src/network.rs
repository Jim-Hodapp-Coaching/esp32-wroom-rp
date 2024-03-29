//! Defines common network functions, types and error definitions.
//!

use defmt::{write, Format, Formatter};

/// A four byte array type alias representing an IP address.
pub type IpAddress = [u8; 4];

/// A named string slice type representing a network hostname.
pub type Hostname<'a> = &'a str;

/// A TCP/UDP network port.
pub type Port = u16;

pub(crate) type Socket = u8;

/// Defines the mode types that the ESP32 firmware can be put into when starting
/// a new client or server instance
#[repr(u8)]
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum TransportMode {
    /// TCP mode
    Tcp = 0,
    /// UDP mode
    Udp = 1,
    /// TLS mode
    Tls = 2,
    /// UDP multicast mode
    UdpMulticast = 3,
    /// TLS BearSSL mode
    TlsBearSsl = 4,
}

/// Defines all possible TCP connection states for a client or server instance.
#[repr(u8)]
#[derive(PartialEq, PartialOrd, Debug)]
pub enum ConnectionState {
    /// Closed
    Closed = 0,
    /// Listening
    Listening = 1,
    /// SynSent
    SynSent = 2,
    /// SynReceived
    SynReceived = 3,
    /// Established
    Established = 4,
    /// FinWait1
    FinWait1 = 5,
    /// Finwait2
    FinWait2 = 6,
    /// CloseWait
    CloseWait = 7,
    /// Closing
    Closing = 8,
    /// LastAck
    LastAck = 9,
    /// TimeWait
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
    /// Timed out while trying to connect to remote TCP server.
    ConnectionTimeout,
    /// Failed to connect to remote TCP server.
    ConnectFailed,
    /// Failed to disconnect from remote TCP server.
    DisconnectFailed,
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
            NetworkError::ConnectionTimeout => {
                write!(fmt, "Timed out while trying connect the remote TCP server")
            }
            NetworkError::ConnectFailed => {
                write!(fmt, "Failed to connect to remote TCP server")
            }
            NetworkError::DisconnectFailed => {
                write!(fmt, "Failed to start up a new TCP/UDP client instance")
            }
        }
    }
}
