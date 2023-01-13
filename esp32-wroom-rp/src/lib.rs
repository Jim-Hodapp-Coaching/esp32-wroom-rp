//! esp32-wroom-rp
//!
//! This crate is an Espressif ESP32-WROOM WiFi module communications driver for RP2040 series microcontroller implemented in Rust.
//! It currently supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf)
//! and [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations intend to add support for the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//!
//! It's intended to communicate with recent versions of most [Arduino-derived WiFiNINA firmwares](https://www.arduino.cc/reference/en/libraries/wifinina/)
//! that run on an ESP32-WROOM-XX WiFi module. For example, Adafruit makes such WiFi hardware, referred to as the [Airlift](https://www.adafruit.com/product/4201), and maintains its [firmware](https://github.com/adafruit/nina-fw).
//!
//! This driver is implemented on top of [embedded-hal](https://github.com/rust-embedded/embedded-hal/), which makes it platform-independent, but is currently only intended
//! to be used with [rp2040-hal](https://github.com/rp-rs/rp-hal/tree/main/rp2040-hal) for your application.
//!
//! Please see the README.md for details on where to obtain and how to connect your RP2040-based device (e.g. Pico) to your ESP32-WROOM-XX WiFi board.
//!
//! Once connected, note that all communication with the WiFi board occurs via a SPI bus. As the example below (and all examples under the directory `cross/`)
//! show, you first need to create an `embedded_hal::spi::Spi` instance. See the [rp2040-hal documentation](https://docs.rs/rp2040-hal/0.6.0/rp2040_hal/spi/index.html) along
//! with the datasheet for your device on what specific SPI ports are available to you.
//!
//! You'll also need to reserve 4 important [GPIO pins](https://docs.rs/rp2040-hal/0.6.0/rp2040_hal/gpio/index.html) (3 output, 1 input) that are used to mediate communication between the two boards. The examples
//! also demonstrate how to do this through instantiating an instance of `esp32_wroom_rp::gpio::EspControlPins`.
//!
//! **NOTE:** This crate is still under active development. This API will remain volatile until 1.0.0.
//!
//! ## Usage
//!
//! First add this to your Cargo.toml
//!
//! ```toml
//! [dependencies]
//! esp32_wroom_rp = 0.3.0
//! ...
//! ```
//!
//! Next:
//!
//! ```
//! // The macro for our start-up function
//! use cortex_m_rt::entry;
//!
//! // Needed for debug output symbols to be linked in binary image
//! use defmt_rtt as _;
//!
//! use panic_probe as _;
//!
//! // Alias for our HAL crate
//! use rp2040_hal as hal;
//!
//! use embedded_hal::spi::MODE_0;
//! use fugit::RateExtU32;
//! use hal::clocks::Clock;
//! use hal::pac;
//!
//! use esp32_wroom_rp::gpio::EspControlPins;
//! use esp32_wroom_rp::wifi::Wifi;
//!
//! // The linker will place this boot block at the start of our program image. We
//! // need this to help the ROM bootloader get our code up and running.
//! #[link_section = ".boot2"]
//! #[used]
//! pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;
//!
//! // External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
//! // if your board has a different frequency
//! const XTAL_FREQ_HZ: u32 = 12_000_000u32;
//!
//! // Entry point to our bare-metal application.
//! //
//! // The `#[entry]` macro ensures the Cortex-M start-up code calls this function
//! // as soon as all global variables are initialized.
//! #[entry]
//! fn main() -> ! {
//!     // Grab our singleton objects
//!     let mut pac = pac::Peripherals::take().unwrap();
//!     let core = pac::CorePeripherals::take().unwrap();
//!
//!     // Set up the watchdog driver - needed by the clock setup code
//!     let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);
//!
//!     // Configure the clocks
//!     let clocks = hal::clocks::init_clocks_and_plls(
//!         XTAL_FREQ_HZ,
//!         pac.XOSC,
//!         pac.CLOCKS,
//!         pac.PLL_SYS,
//!         pac.PLL_USB,
//!         &mut pac.RESETS,
//!         &mut watchdog,
//!     )
//!     .ok()
//!     .unwrap();
//!
//!     let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
//!
//!     // The single-cycle I/O block controls our GPIO pins
//!     let sio = hal::Sio::new(pac.SIO);
//!
//!     // Set the pins to their default state
//!     let pins = hal::gpio::Pins::new(
//!         pac.IO_BANK0,
//!         pac.PADS_BANK0,
//!         sio.gpio_bank0,
//!         &mut pac.RESETS,
//!     );
//!
//!     defmt::info!("ESP32-WROOM-RP get NINA firmware version example");
//!
//!     // These are implicitly used by the spi driver if they are in the correct mode
//!     let _spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
//!     let _spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
//!     let _spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();
//!
//!     let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);
//!
//!     // Exchange the uninitialized SPI driver for an initialized one
//!     let spi = spi.init(
//!         &mut pac.RESETS,
//!         clocks.peripheral_clock.freq(),
//!         8.MHz(),
//!         &MODE_0,
//!     );
//!
//!     let esp_pins = EspControlPins {
//!         // CS on pin x (GPIO7)
//!         cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
//!         // GPIO0 on pin x (GPIO2)
//!         gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
//!         // RESETn on pin x (GPIO11)
//!         resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
//!         // ACK on pin x (GPIO10)
//!         ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
//!     };
//!     let mut wifi = Wifi::init(spi, esp_pins, &mut delay).unwrap();
//!     let firmware_version = wifi.firmware_version();
//!     defmt::info!("NINA firmware version: {:?}", firmware_version);
//!
//!     // Infinitely sit in a main loop
//!     loop {}
//! }
//! ```
//!
//! ## More examples
//!
//! Please refer to the `cross/` directory in this crate's source for examples that demonstrate how to use every part of its public API.
//!

#![doc(html_root_url = "https://docs.rs/esp32-wroom-rp")]
#![doc(issue_tracker_base_url = "https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/issues")]
#![warn(missing_docs)]
#![cfg_attr(not(test), no_std)]

pub mod gpio;
pub mod network;
pub mod protocol;
pub mod tcp_client;
pub mod wifi;

mod spi;

use defmt::{write, Format, Formatter};

use network::NetworkError;

use protocol::ProtocolError;

const ARRAY_LENGTH_PLACEHOLDER: usize = 8;

/// Highest level error types for this crate.
#[derive(Debug, Eq, PartialEq)]
pub enum Error {
    /// SPI/I2C related communications error with the ESP32 WiFi target
    Bus,
    /// Protocol error in communicating with the ESP32 WiFi target
    Protocol(ProtocolError),

    /// Network related error
    Network(NetworkError),
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
            Error::Network(e) => write!(fmt, "Network error: {}", e),
        }
    }
}

impl From<protocol::ProtocolError> for Error {
    fn from(err: protocol::ProtocolError) -> Self {
        Error::Protocol(err)
    }
}

impl From<network::NetworkError> for Error {
    fn from(err: network::NetworkError) -> Self {
        Error::Network(err)
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
