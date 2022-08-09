//! esp32-wroom-rp
//!
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//!
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0

#![no_std]
#![no_main]

pub mod pins;
pub mod spi;

// This is just a placeholder for now.
type Params = [u8; 5];

#[repr(u8)]
#[derive(Debug)]
enum NinaCommand {
    StartClientTcp = 0x2du8,
    GetFwVersion = 0x37u8,
}

#[derive(Debug)]
pub enum Error {
    // Placeholder variants
    Bus,
    TimeOut,
}

pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8,
}

impl FirmwareVersion {
    fn new(version: [u8; 5]) -> FirmwareVersion {
        Self::parse(version)
    }

    // Takes in 5 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
    fn parse(version: [u8; 5]) -> FirmwareVersion {
        // TODO: real implementation
        FirmwareVersion {
            major: 1,
            minor: 7,
            patch: 4,
        }
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

// NinaCommandHandler?
trait Interface {
    //type Error;

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>;

    // This will not return FirmwareVersion
    fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn firmware_parse_returns_a_populated_firmware_struct() {}
}
