pub use crate::spi::Wifi;

use defmt::{write, Format, Formatter};

/// An enumerated type that represents the current WiFi network connection status.
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd, Debug)]
pub enum ConnectionStatus {
    /// No device is connected to hardware
    NoEsp32 = 255,
    /// Temporary status while attempting to connect to WiFi network
    Idle = 0,
    /// No SSID is available
    NoActiveSsid,
    /// WiFi network scan has finished
    ScanCompleted,
    /// Device is connected to WiFi network
    Connected,
    /// Device failed to connect to WiFi network
    Failed,
    /// Device lost connection to WiFi network
    Lost,
    /// Device disconnected from WiFi network
    Disconnected,
    /// Device is listening for connections in Access Point mode
    ApListening,
    /// Device is connected in Access Point mode
    ApConnected,
    /// Device failed to make connection in Access Point mode
    ApFailed,
    /// Unexpected value returned from device, reset may be required
    Invalid,
}

impl From<u8> for ConnectionStatus {
    fn from(status: u8) -> ConnectionStatus {
        match status {
            0   => ConnectionStatus::Idle,
            1   => ConnectionStatus::NoActiveSsid,
            2   => ConnectionStatus::ScanCompleted,
            3   => ConnectionStatus::Connected,
            4   => ConnectionStatus::Failed,
            5   => ConnectionStatus::Lost,
            6   => ConnectionStatus::Disconnected,
            7   => ConnectionStatus::ApListening,
            8   => ConnectionStatus::ApConnected,
            9   => ConnectionStatus::ApFailed,
            255 => ConnectionStatus::NoEsp32,
            _   => ConnectionStatus::Invalid,
        }
    }
}

impl Format for ConnectionStatus {
    fn format(&self, fmt: Formatter) {
        match self {
            ConnectionStatus::NoEsp32 => write!(
                fmt,"No device is connected to hardware"
                ),
            ConnectionStatus::Idle => write!(
                fmt,
                "Temporary status while attempting to connect to WiFi network"
                ),
            ConnectionStatus::NoActiveSsid => write!(
                fmt,
                "No SSID is available"
                ),
            ConnectionStatus::ScanCompleted => write!(
                fmt,
                "WiFi network scan has finished"
                ),
            ConnectionStatus::Connected => write!(
                fmt,
                "Device is connected to WiFi network"
                ),
            ConnectionStatus::Failed => write!(
                fmt,
                "Device failed to connect to WiFi network"
                ),
            ConnectionStatus::Lost => write!(
                fmt,
                "Device lost connection to WiFi network"
                ),
            ConnectionStatus::Disconnected => write!(
                fmt,
                "Device disconnected from WiFi network"
                ),
            ConnectionStatus::ApListening => write!(
                fmt,
                "Device is lstening for connections in Access Point mode"
                ),
            ConnectionStatus::ApConnected => write!(
                fmt,
                "Device is connected in Access Point mode"
                ),
            ConnectionStatus::ApFailed => write!(
                fmt,
                "Device failed to make connection in Access Point mode"
                ),
            ConnectionStatus::Invalid => write!(
                fmt,
                "Unexpected value returned from device, reset may be required"
                ),
        }
    }
}
