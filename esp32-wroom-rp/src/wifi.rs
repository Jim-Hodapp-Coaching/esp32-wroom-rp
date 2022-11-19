pub use crate::spi::Wifi;

/// Defines the WiFi network connection status
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd, Debug)]
pub enum ConnectionStatus {
    NoEsp32 = 255,
    Idle = 0, // Assigned by WiFi.begin() in the Reference Library, is this relevant?
    NoActiveSsid,
    ScanCompleted,
    Connected,
    Failed,
    Lost,
    Disconnected,
    ApListening,
    ApConnected,
    ApFailed, // Not in the Reference Library
}

impl From<u8> for ConnectionStatus {
    fn from(status: u8) -> ConnectionStatus {
        match status {
            0 => ConnectionStatus::Idle,
            1 => ConnectionStatus::NoActiveSsid,
            2 => ConnectionStatus::ScanCompleted,
            3 => ConnectionStatus::Connected,
            4 => ConnectionStatus::Failed,
            5 => ConnectionStatus::Lost,
            6 => ConnectionStatus::Disconnected,
            7 => ConnectionStatus::ApListening,
            8 => ConnectionStatus::ApConnected,
            9 => ConnectionStatus::ApFailed,
            255 => ConnectionStatus::NoEsp32,
            _ => panic!("Unexpected value: {}", status),
        }
    }
}
