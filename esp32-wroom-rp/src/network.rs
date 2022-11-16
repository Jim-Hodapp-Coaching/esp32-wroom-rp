use defmt::{write, Format, Formatter};

/// Errors that occur due to issues involving communication over
/// WiFi network.
#[derive(PartialEq, Eq, Debug)]
pub enum NetworkError {
    /// Failure to resolve a hostname to an IP address.
    DnsResolveFailed,
}

impl Format for NetworkError {
    fn format(&self, fmt: Formatter) {
        match self {
            NetworkError::DnsResolveFailed => {
                write!(fmt, "Failed when trying to resolve a DNS hostname")
            }
        }
    }
}
