//! Perform common core WiFi functions such as join a network, resolve a DNS hostname, etc.
//!
//! ## Usage
//!
//! ```no_run
//! let mut wifi = Wifi::init(spi, esp_pins, &mut delay).unwrap();
//!
//! let result = wifi.join(ssid, passphrase);
//! defmt::info!("Join Result: {:?}", result);
//!
//! defmt::info!("Entering main loop");
//!
//! let mut sleep: u32 = 1500;
//! loop {
//!     match wifi.get_connection_status() {
//!         Ok(status) => {
//!             defmt::info!("Connection status: {:?}", status);
//!             delay.delay_ms(sleep);
//!
//!             if status == ConnectionStatus::Connected {
//!                 defmt::info!("Connected to network: {:?}", SSID);
//!
//!                 // The IPAddresses of two DNS servers to resolve hostnames with.
//!                 // Note that failover from ip1 to ip2 is fully functional.
//!                 let ip1: IpAddress = [9, 9, 9, 9];
//!                 let ip2: IpAddress = [8, 8, 8, 8];
//!                 let dns_result = wifi.set_dns(ip1, Some(ip2));
//!
//!                 defmt::info!("set_dns result: {:?}", dns_result);
//!
//!                 match wifi.resolve(hostname) {
//!                     Ok(ip) => {
//!                         defmt::info!("Server IP: {:?}", ip);
//!                     }
//!                     Err(e) => {
//!                         defmt::error!("Failed to resolve hostname {}", hostname);
//!                         defmt::error!("Err: {}", e);
//!                     }
//!                 }
//!
//!                 defmt::info!("Leaving network: {:?}", ssid);
//!                 wifi.leave().ok();
//!             } else if status == ConnectionStatus::Disconnected {
//!                 sleep = 20000; // No need to loop as often after disconnecting
//!             }
//!         }
//!         Err(e) => {
//!             defmt::error!("Failed to get connection result: {:?}", e);
//!         }
//!     }
//! }
//! ```
//!

use core::cell::RefCell;

use defmt::{write, Format, Formatter};

use embedded_hal::blocking::{delay::DelayMs, spi::Transfer};

use super::gpio::EspControlInterface;
use super::network::IpAddress;
use super::protocol::{NinaProtocolHandler, ProtocolInterface};
use super::{Error, FirmwareVersion};

/// An enumerated type that represents the current WiFi network connection status.
#[repr(u8)]
#[derive(Eq, PartialEq, PartialOrd, Debug)]
pub enum ConnectionStatus {
    /// No device is connected to hardware
    NoEsp32 = 255,
    /// No SSID is available
    NoActiveSsid = 1,
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
            _ => ConnectionStatus::Invalid,
        }
    }
}

impl Format for ConnectionStatus {
    fn format(&self, fmt: Formatter) {
        match self {
            ConnectionStatus::NoEsp32 => write!(fmt, "No device is connected to hardware"),
            ConnectionStatus::NoActiveSsid => write!(fmt, "No SSID is available"),
            ConnectionStatus::ScanCompleted => write!(fmt, "WiFi network scan has finished"),
            ConnectionStatus::Connected => write!(fmt, "Device is connected to WiFi network"),
            ConnectionStatus::Failed => write!(fmt, "Device failed to connect to WiFi network"),
            ConnectionStatus::Lost => write!(fmt, "Device lost connection to WiFi network"),
            ConnectionStatus::Disconnected => write!(fmt, "Device disconnected from WiFi network"),
            ConnectionStatus::ApListening => write!(
                fmt,
                "Device is listening for connections in Access Point mode"
            ),
            ConnectionStatus::ApConnected => {
                write!(fmt, "Device is connected in Access Point mode")
            }
            ConnectionStatus::ApFailed => {
                write!(fmt, "Device failed to make connection in Access Point mode")
            }
            ConnectionStatus::Invalid => write!(
                fmt,
                "Unexpected value returned from device, reset may be required"
            ),
        }
    }
}

/// Base type for controlling an ESP32-WROOM NINA firmware-based WiFi board.
#[derive(Debug)]
pub struct Wifi<B, C> {
    pub(crate) protocol_handler: RefCell<NinaProtocolHandler<B, C>>,
}

impl<S, C> Wifi<S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    /// Initialize the ESP32-WROOM WiFi device.
    /// Call this function to put the connected ESP32-WROOM device in a known good state to accept commands.
    pub fn init<D: DelayMs<u16>>(
        spi: S,
        esp32_control_pins: C,
        delay: &mut D,
    ) -> Result<Wifi<S, C>, Error> {
        let wifi = Wifi {
            protocol_handler: RefCell::new(NinaProtocolHandler {
                bus: RefCell::new(spi),
                control_pins: esp32_control_pins,
            }),
        };

        wifi.protocol_handler.borrow_mut().init();
        wifi.protocol_handler.borrow_mut().reset(delay);
        Ok(wifi)
    }

    /// Retrieve the NINA firmware version contained on the connected ESP32-WROOM device (e.g. 1.7.4).
    pub fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        self.protocol_handler.borrow_mut().get_fw_version()
    }

    /// Join a WiFi network given an SSID and a Passphrase.
    pub fn join(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        self.protocol_handler
            .borrow_mut()
            .set_passphrase(ssid, passphrase)
    }

    /// Disconnect from a previously joined WiFi network.
    pub fn leave(&mut self) -> Result<(), Error> {
        self.protocol_handler.borrow_mut().disconnect()
    }

    /// Retrieve the current WiFi network [`ConnectionStatus`].
    pub fn get_connection_status(&mut self) -> Result<ConnectionStatus, Error> {
        self.protocol_handler.borrow_mut().get_conn_status()
    }

    /// Set 1 or 2 DNS servers that are used for network hostname resolution.
    pub fn set_dns(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error> {
        self.protocol_handler
            .borrow_mut()
            .set_dns_config(dns1, dns2)
    }

    /// Query the DNS server(s) provided via `set_dns` for the associated IP address to the provided hostname.
    pub fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error> {
        self.protocol_handler.borrow_mut().resolve(hostname)
    }

    /// Return a reference to the `Spi` bus instance typically used when cleaning up
    /// an instance of [`Wifi`].
    pub fn destroy(self) -> S {
        self.protocol_handler.into_inner().bus.into_inner()
    }
}
