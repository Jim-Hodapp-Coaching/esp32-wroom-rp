use embedded_hal::digital::blocking::{InputPin, OutputPin};

use rp2040_hal as hal;
use rp2040_hal::gpio::bank0::{Gpio10, Gpio11, Gpio2, Gpio7};
use rp2040_hal::gpio::Pin;

#[derive(Clone, Copy, Debug)]
pub enum IOError {
    Pin,
}

pub trait ESP32ControlInterface {
    fn init(&mut self);

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

impl<CS, GPIO0, RESETN, ACK> ESP32ControlInterface for EspControlPins<CS, GPIO0, RESETN, ACK>
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
