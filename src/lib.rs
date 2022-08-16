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
//! {add our crate here}
//! ```
//!
//! Next:
//!
//! ```no_run
//! use rp2040_hal as hal;
//! use rp2040_hal::pac;
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
//! let uart_pins = (
//!     // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
//!     pins.gpio0.into_mode::<hal::gpio::FunctionUart>(),
//!     // UART RX (characters reveived by RP2040) on pin 2 (GPIO1)
//!     pins.gpio1.into_mode::<hal::gpio::FunctionUart>(),
//! );
//!
//! let uart = hal::uart::UartPeripheral::<_, _, _>::new(pac.UART0, uart_pins, &mut pac.RESETS)
//!     .enable(
//!         hal::uart::common_configs::_115200_8_N_1,
//!         clocks.peripheral_clock.freq(),
//!     )
//!     .unwrap();
//!
//! defmt::info!("ESP32-WROOM-RP get NINA firmware version example");
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
//!     8_000_000u32.Hz(),
//!     &MODE_0,
//! );
//!
//! let esp_pins = esp32_wroom_rp::pins::EspControlPins {
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
//! let mut wifi = esp32_wroom_rp::spi::Wifi::init(spi, esp_pins).unwrap();
//! wifi.firmware_version();
//! ```

#![allow(dead_code, unused_imports)]
#![no_std]
#![no_main]

pub mod gpio;
pub mod spi;
pub mod wifi;

mod protocol;

use protocol::ProtocolInterface;

use defmt::{write, Format, Formatter};
use embedded_hal::delay::blocking::DelayUs;

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

    // Takes in 8 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
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
struct WifiCommon<PH> {
    protocol_handler: PH,
}

impl<PH> WifiCommon<PH>
where
    PH: ProtocolInterface,
{
    fn init<D: DelayUs>(&mut self, delay: &mut D) {
        self.reset(delay);
    }

    fn configure() {}

    fn reset<D: DelayUs>(&mut self, delay: &mut D) {
        self.protocol_handler.reset(delay)
    }

    fn firmware_version(&mut self) -> Result<FirmwareVersion, self::Error> {
        self.protocol_handler.get_fw_version()
    }
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
