//! Control interface for ESP32
//!
//! ## Usage
//!
//! ```no_run
//! use rp2040_hal as hal;
//! use rp2040_hal::pac;
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
//! ```
use embedded_hal::delay::blocking::DelayUs;
use embedded_hal::digital::blocking::{InputPin, OutputPin};

use rp2040_hal as hal;
use rp2040_hal::gpio::bank0::{Gpio10, Gpio11, Gpio2, Gpio7};
use rp2040_hal::gpio::Pin;

#[derive(Clone, Copy, Debug)]
pub enum IOError {
    Pin,
}

pub trait EspControlInterface {
    fn init(&mut self);

    fn reset<D: DelayUs>(&mut self, delay: &mut D);

    fn esp_select(&mut self);

    fn esp_deselect(&mut self);

    fn get_esp_ready(&self) -> bool;

    fn get_esp_ack(&self) -> bool;

    fn wait_for_esp_ready(&self);

    fn wait_for_esp_ack(&self);

    fn wait_for_esp_select(&mut self);
}

pub struct EspControlPins<CS, GPIO0, RESETN, ACK> {
    pub cs: CS,
    pub gpio0: GPIO0,
    pub resetn: RESETN,
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
        self.cs.set_high().unwrap();
    }

    fn reset<D: DelayUs>(&mut self, delay: &mut D) {
        self.gpio0.set_high().unwrap();
        self.cs.set_high().unwrap();
        self.resetn.set_low().unwrap();
        delay.delay_ms(10).ok().unwrap();
        self.resetn.set_high().unwrap();
        delay.delay_ms(750).ok().unwrap();
    }

    fn esp_select(&mut self) {
        self.cs.set_low().unwrap();
    }

    fn esp_deselect(&mut self) {
        self.cs.set_high().unwrap();
    }

    fn get_esp_ready(&self) -> bool {
        self.ack.is_low().unwrap()
    }

    fn get_esp_ack(&self) -> bool {
        self.ack.is_high().unwrap()
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
