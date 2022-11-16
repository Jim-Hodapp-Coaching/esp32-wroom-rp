use defmt::{write, Format, Formatter};

#[derive(PartialEq, Eq, Debug)]
pub enum NetworkError {
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
