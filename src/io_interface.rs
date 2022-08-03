use super::EspPins;
use embedded_hal::digital::blocking::{InputPin, OutputPin};

#[derive(Clone, Copy, Debug)]
pub enum IOError {
    Pin,
}

pub trait IoInterface {
    type Error;

    fn esp_select(&mut self);

    fn esp_deselect(&mut self);

    fn get_esp_ready(&self) -> bool;

    fn get_esp_ack(&self) -> bool;

    fn wait_for_esp_ready(&self);

    fn wait_for_esp_ack(&self);

    fn wait_for_esp_select(&mut self);
}

pub struct IoInterfaceImpl {
    esp_pins: EspPins,
}

impl IoInterface for IoInterfaceImpl {
    type Error = IOError;
    // TODO: add error handling
    fn esp_select(&mut self) {
        self.esp_pins.cs.set_low().unwrap();
    }

    //   fn esp_deselect(&mut self) -> Result<(), Error<Self::Error>> {
    //     //   self.esp_pins.cs.set_high().unwrap();
    //     self.esp_pins.cs.set_high().map_err(|e| IOError::Pin(e))?;
    //   }

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
