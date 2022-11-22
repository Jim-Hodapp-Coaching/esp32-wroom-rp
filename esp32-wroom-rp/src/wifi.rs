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