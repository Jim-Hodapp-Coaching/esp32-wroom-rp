//! GPIO pin control interface of a connected ESP32-WROOM target WiFi board.
//!
//! ## Usage
//!
//! ```no_run
//! use esp32_wroom_rp::gpio::*;
//!
//! let mut pac = pac::Peripherals::take().unwrap();
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
//! ```

use core::hint;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{InputPin, OutputPin};

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum IOError {
    Pin,
}

/// Provides an internal pin interface that abstracts the extra control lines that
/// are separate from a data bus (e.g. SPI/I2C).
///
/// Not meant to be used outside of the crate.
pub trait EspControlInterface {
    /// Initializes all controls pins to set ready communication with the NINA firmware.
    fn init(&mut self);

    /// Resets communication with the NINA firmware.
    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D);

    /// Tells the NINA firmware we're about to send it a protocol command.
    fn esp_select(&mut self);

    /// Tells the NINA firmware we're done sending it a protocol command.
    fn esp_deselect(&mut self);

    /// Is the NINA firmware ready to send it a protocol command?
    fn get_esp_ready(&self) -> bool;

    /// Is the NINA firmware ready to receive more commands? Also referred to as BUSY.
    fn get_esp_ack(&self) -> bool;

    /// Blocking waits for the NINA firmware to be ready to send it a protocol command.
    fn wait_for_esp_ready(&self);

    /// Blocking waits for the NINA firmware to acknowledge it's ready to receive more commands.
    fn wait_for_esp_ack(&self);

    /// Blocking waits for the NINA firmware to be ready to send it a protocol command.
    fn wait_for_esp_select(&mut self);
}

/// A structured representation of all GPIO pins that control a ESP32-WROOM NINA firmware-based
/// device outside of commands sent over the SPI/I²C bus. Pass a single instance of this struct
/// into `Wifi::init()`.
pub struct EspControlPins<CS, GPIO0, RESETN, ACK> {
    /// Chip select pin to let the NINA firmware know we're going to send it a command over
    /// the SPI bus.
    pub cs: CS,
    /// Puts the ESP32 WiFi target into bootloading mode. Or if acting as a server, provides
    /// a status line for when data is ready to be read.
    pub gpio0: GPIO0,
    /// Places the ESP32 WiFi target into reset mode. Useful for when the target gets into
    /// a stuck state.
    pub resetn: RESETN,
    /// Is the ESP32 WiFi target busy?
    pub ack: ACK,
}

impl<CS, GPIO0, RESETN, ACK> EspControlInterface for EspControlPins<CS, GPIO0, RESETN, ACK>
where
    CS: OutputPin,
    GPIO0: OutputPin,
    RESETN: OutputPin,
    ACK: InputPin,
{
    fn init(&mut self) {
        // Chip select is active-low, so we'll initialize it to a driven-high state
        self.cs.set_high().ok();
        self.gpio0.set_high().ok();
        self.resetn.set_high().ok();
        self.get_esp_ready();
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.gpio0.set_high().ok();
        self.cs.set_high().ok();
        self.resetn.set_low().ok();
        delay.delay_ms(10);
        self.resetn.set_high().ok();
        delay.delay_ms(750);
    }

    fn esp_select(&mut self) {
        self.cs.set_low().ok();
    }

    fn esp_deselect(&mut self) {
        self.cs.set_high().ok();
    }

    fn get_esp_ready(&self) -> bool {
        self.ack.is_low().ok().unwrap()
    }

    fn get_esp_ack(&self) -> bool {
        self.ack.is_high().ok().unwrap()
    }

    fn wait_for_esp_ready(&self) {
        while !self.get_esp_ready() {
            hint::spin_loop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_ack(&self) {
        while !self.get_esp_ack() {
            hint::spin_loop(); // Make sure rustc doesn't optimize this loop out
        }
    }

    fn wait_for_esp_select(&mut self) {
        self.wait_for_esp_ready();
        self.esp_select();
        self.wait_for_esp_ack();
    }
}

impl Default for EspControlPins<(), (), (), ()> {
    fn default() -> Self {
        Self {
            cs: (),
            gpio0: (),
            resetn: (),
            ack: (),
        }
    }
}

#[cfg(test)]
mod gpio_tests {
    use super::EspControlPins;
    use crate::gpio::EspControlInterface;
    use embedded_hal_mock::pin::{
        Mock as PinMock, State as PinState, Transaction as PinTransaction,
    };

    #[test]
    fn gpio_init_sets_correct_state() {
        let cs_expectations = [PinTransaction::set(PinState::High)];

        let gpio0_expectations = [PinTransaction::set(PinState::High)];

        let resetn_expectations = [PinTransaction::set(PinState::High)];

        let ack_expectations = [PinTransaction::get(PinState::Low)];

        let cs_mock = PinMock::new(&cs_expectations);
        let gpio0_mock = PinMock::new(&gpio0_expectations);
        let resetn_mock = PinMock::new(&resetn_expectations);
        let ack_mock = PinMock::new(&ack_expectations);
        let mut pins = EspControlPins {
            cs: cs_mock,
            gpio0: gpio0_mock,
            resetn: resetn_mock,
            ack: ack_mock,
        };

        pins.init();

        pins.cs.done();
        pins.gpio0.done();
        pins.resetn.done();
        pins.ack.done();
    }
}
