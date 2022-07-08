//! esp32-wroom-rp
//! 
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//! 
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0

#![no_std]
#![no_main]

pub mod spi;

use rp2040_hal as hal;

use rp2040_hal::gpio::bank0::{Gpio2, Gpio7, Gpio10, Gpio11};
use rp2040_hal::gpio::Pin;
use embedded_hal::digital::blocking::InputPin;
use embedded_hal::digital::blocking::OutputPin;

// This is just a placeholder for now. 
type Params = [u8; 5];

#[repr(u8)]
#[derive(Debug)]
enum NinaCommand {
  StartClientTcp = 0x2du8,
  GetFirmwareVersion = 0x37u8

}

#[derive(Debug)]
pub enum Error {
    // Placeholder variants
    Bus,
    TimeOut
}

// Wifi::new();
// Wifi::init();
// Wifi::get_firmware_version();
pub struct Wifi<C> {
  common: C,
}

struct WifiCommon<N> {
    command_handler: N
}

impl<N> WifiCommon<N>
where
    N: NinaCommandHandler,
{
  fn get_firmware_version() -> Result<FirmwareVersion, Error> {
    self.command_handler.get_firmware_version
  }
}

pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8
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
            patch: 4
        }
    }
}

impl<C> Wifi<C> {
    fn new() {}
    fn init() {}
    // fn init_with_config(config: Configuration) {}

    // fn connect(&self, params: Params) -> Result<T, E> {
    //     self.command_handler.start_client_tcp(params: Params)
    // }

    fn get_firmware_version(&self) -> Result<FirmwareVersion, Error> {
      self.common.get_fw_version()
    }
}

trait NinaCommandHandler {

  fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, Error>;

  fn get_fw_version(&self) -> Result<FirmwareVersion, Error>;
}

trait IoInterface {

  fn esp_select(&mut self);

  fn esp_deselect(&mut self);

  fn get_esp_ready(&self) -> bool;

  fn get_esp_ack(&self) -> bool;

  fn wait_for_esp_ready(&self);

  fn wait_for_esp_ack(&self);

  fn wait_for_esp_select(&mut self);
  
}

struct IoInterfaceImpl {
  esp_pins: EspPins
}

impl IoInterface for IoInterfaceImpl {
    // TODO: add error handling
    fn esp_select(&mut self) {
        self.esp_pins.cs.set_low().unwrap();
    }

    fn esp_deselect(&mut self) {
        self.esp_pins.cs.set_high().unwrap();
    }

    fn get_esp_ready(&self) -> bool {
        self.esp_pins.ack.is_low().unwrap()
    }

    fn get_esp_ack(&self) -> bool {
        self.esp_pins.ack.is_high().unwrap()
    }

    fn wait_for_esp_ready(&self) {
        while self.get_esp_ready() != true {
            cortex_m::asm::nop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_ack(&self) {
        while self.get_esp_ack() == false {
            cortex_m::asm::nop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_select(&mut self) {
        self.wait_for_esp_ready();
        self.esp_select();
        self.wait_for_esp_ack();
    }

}

struct EspPins {
    cs: Pin<Gpio7, hal::gpio::PushPullOutput>,
    gpio0: Pin<Gpio2, hal::gpio::PushPullOutput>,
    resetn: Pin<Gpio11, hal::gpio::PushPullOutput>,
    ack: Pin<Gpio10, hal::gpio::FloatingInput>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn firmware_parse_returns_a_populated_firmware_struct() {
        
    } 
}