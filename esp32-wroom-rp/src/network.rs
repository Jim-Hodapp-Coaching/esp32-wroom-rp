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

/// Errors that occur due to issues involving communication over
/// WiFi network.
#[derive(PartialEq, Eq, Debug)]
pub enum NetworkError {
    /// Failed to resolve a hostname for the provided IP address.
    DnsResolveFailed,
    /// Failed to start up a new TCP/UDP client instancwe.
    StartClientFailed,
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
                write!(
                    fmt,
                    "Failed to start up a new TCP/UDP client instance"
                )
            }
        }
    }
}
