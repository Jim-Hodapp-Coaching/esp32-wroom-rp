//! esp32-wroom-rp
//!
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//!
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0

#![allow(dead_code, unused_imports)]
#![no_std]
#![no_main]

pub mod pins;
pub mod spi;

use defmt::{write, Format, Formatter};

// This is just a placeholder for now.
type Params = [u8; 5];

#[derive(Debug)]
pub enum Error {
    // Placeholder variants
    Bus,
    TimeOut,
}

impl Format for Error {
    fn format(&self, fmt: Formatter) {
        write!(fmt, "Generic ESP32-WROOM-RP Error")
    }
}

#[derive(Debug, Default, PartialEq)]
pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl FirmwareVersion {
    fn new(version: [u8; 8]) -> FirmwareVersion {
        Self::parse(version)
    }

    // Takes in 5 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
    fn parse(version: [u8; 8]) -> FirmwareVersion {
        let major: u8;
        let minor: u8;
        let patch: u8;

        [major, _, minor, _, patch, _, _, _] = version;

        FirmwareVersion {
            major: major,
            minor: minor,
            patch: patch,
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

#[derive(Debug, Default)]
struct WifiCommon<I> {
    interface: I,
}

impl<I> WifiCommon<I>
where
    I: Interface,
{
    fn init() {}

    fn configure() {}

    fn firmware_version(&mut self) -> Result<FirmwareVersion, self::Error> {
        self.interface.get_fw_version()
    }
}

trait Interface {
    //type Error;

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn firmware_new_returns_a_populated_firmware_struct() {
        let firmware_version: FirmwareVersion =
            FirmwareVersion::new([0x31, 0x2e, 0x37, 0x2e, 0x34]);

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
