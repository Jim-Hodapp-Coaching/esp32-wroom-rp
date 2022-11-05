//! esp32-wroom-rp
//!
//! Rust-based Espressif ESP32-WROOM WiFi driver for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//!
//! **NOTE:** This crate is still under active development. This API will remain volatile until 1.0.0
//!
//! ## Usage
//!
//! First add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! esp32_wroom_rp = 0.3
//! ```
//!
//! Next:
//!
//! ```no_run
//! use esp32_wroom_rp::spi::*;
//! use embedded_hal::blocking::delay::DelayMs;
//!
//! let mut pac = pac::Peripherals::take().unwrap();
//! let core = pac::CorePeripherals::take().unwrap();
//!
//! let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
//!
//! // Configure the clocks
//! let clocks = hal::clocks::init_clocks_and_plls(
//!     XTAL_FREQ_HZ,
//!     pac.XOSC,
//!     pac.CLOCKS,
//!     pac.PLL_SYS,
//!     pac.PLL_USB,
//!     &mut pac.RESETS,
//!     &mut watchdog,
//! )
//! .ok()
//! .unwrap();
//!
//! // The single-cycle I/O block controls our GPIO pins
//! let sio = hal::Sio::new(pac.SIO);
//!
//! // Set the pins to their default state
//! let pins = hal::gpio::Pins::new(
//!     pac.IO_BANK0,
//!     pac.PADS_BANK0,
//!     sio.gpio_bank0,
//!     &mut pac.RESETS,
//! );
//!
//! let spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
//! let spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
//! let spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();
//!
//! let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);
//!
//! // Exchange the uninitialized SPI driver for an initialized one
//! let spi = spi.init(
//!     &mut pac.RESETS,
//!     clocks.peripheral_clock.freq(),
//!     8.MHz(),
//!     &MODE_0,
//! );
//!
//! let esp_pins = esp32_wroom_rp::gpio::EspControlPins {
//!     // CS on pin x (GPIO7)
//!     cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
//!     // GPIO0 on pin x (GPIO2)
//!     gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
//!     // RESETn on pin x (GPIO11)
//!     resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
//!     // ACK on pin x (GPIO10)
//!     ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
//! };
//!
//! let mut wifi = esp32_wroom_rp::spi::Wifi::init(&mut spi, &mut esp_pins, &mut delay).unwrap();
//! let version = wifi.firmware_version();
//! ```

#![doc(html_root_url = "https://docs.rs/esp32-wroom-rp")]
#![doc(issue_tracker_base_url = "https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/issues")]
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

pub mod gpio;
/// Fundamental interface for controlling a connected ESP32-WROOM NINA firmware-based Wifi board.
pub mod wifi;

pub mod protocol;
mod spi;

use protocol::{ProtocolError, ProtocolInterface};

use defmt::{write, Format, Formatter};
use embedded_hal::blocking::delay::DelayMs;

const ARRAY_LENGTH_PLACEHOLDER: usize = 8;

pub type IpAddress = [u8; 4];

/// Highest level error types for this crate.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// SPI/I2C related communications error with the ESP32 WiFi target
    Bus,
    /// Protocol error in communicating with the ESP32 WiFi target
    Protocol(ProtocolError),
}

impl Format for Error {
    fn format(&self, fmt: Formatter) {
        match self {
            Error::Bus => write!(fmt, "Bus error"),
            Error::Protocol(e) => write!(
                fmt,
                "Communication protocol error with ESP32 WiFi target: {}",
                e
            ),
        }
    }
}

impl From<protocol::ProtocolError> for Error {
    fn from(err: protocol::ProtocolError) -> Self {
        Error::Protocol(err)
    }
}

/// A structured representation of a connected NINA firmware device's version number (e.g. 1.7.4).
#[derive(Debug, Default, Eq, PartialEq)]
pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl FirmwareVersion {
    fn new(version: [u8; ARRAY_LENGTH_PLACEHOLDER]) -> FirmwareVersion {
        Self::parse(version)
    }

    // Takes in 8 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
    fn parse(version: [u8; ARRAY_LENGTH_PLACEHOLDER]) -> FirmwareVersion {
        let major_version: u8;
        let minor_version: u8;
        let patch_version: u8;

        [major_version, _, minor_version, _, patch_version, _, _, _] = version;

        FirmwareVersion {
            major: major_version,
            minor: minor_version,
            patch: patch_version,
        }
    }
}

impl Format for FirmwareVersion {
    fn format(&self, fmt: Formatter) {
        write!(
            fmt,
            "Major: {:?}, Minor: {:?}, Patch: {:?}",
            self.major as char, self.minor as char, self.patch as char
        );
    }
}

#[derive(Debug)]
struct WifiCommon<PH> {
    protocol_handler: PH,
}

impl<PH> WifiCommon<PH>
where
    PH: ProtocolInterface,
{
    fn init<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.protocol_handler.init();
        self.reset(delay);
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.protocol_handler.reset(delay)
    }

    fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        Ok(self.protocol_handler.get_fw_version()?)
    }

    fn join(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        Ok(self.protocol_handler.set_passphrase(ssid, passphrase)?)
    }

    fn leave(&mut self) -> Result<(), Error> {
        Ok(self.protocol_handler.disconnect()?)
    }

    fn get_connection_status(&mut self) -> Result<u8, Error> {
        Ok(self.protocol_handler.get_conn_status()?)
    }

    fn set_dns(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error> {
        Ok(self.protocol_handler.set_dns_config(dns1, dns2)?)
    }

    fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error> {
        Ok(self.protocol_handler.resolve(hostname)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firmware_new_returns_a_populated_firmware_struct() {
        let firmware_version: FirmwareVersion =
            FirmwareVersion::new([0x1, 0x2e, 0x7, 0x2e, 0x4, 0x0, 0x0, 0x0]);

        assert_eq!(
            firmware_version,
            FirmwareVersion {
                major: 1,
                minor: 7,
                patch: 4
            }
        )
    }
}
