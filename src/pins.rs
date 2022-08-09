use embedded_hal::digital::blocking::{InputPin, OutputPin};

use rp2040_hal as hal;
use rp2040_hal::gpio::bank0::{Gpio10, Gpio11, Gpio2, Gpio7};
use rp2040_hal::gpio::Pin;

#[derive(Clone, Copy, Debug)]
enum IOError {
    Pin,
}

pub trait ESP32ControlInterface {
    //type Error;

    fn esp_select(&mut self);

    fn esp_deselect(&mut self);

    fn get_esp_ready(&self) -> bool;

    fn get_esp_ack(&self) -> bool;

    fn wait_for_esp_ready(&self);

    fn wait_for_esp_ack(&self);

    fn wait_for_esp_select(&mut self);
}

// struct EspControlPins {
//     esp_pins: EspPins<Pin<Gpio7, hal::gpio::PushPullOutput>,
//                       Pin<Gpio2, hal::gpio::PushPullOutput>,
//                       Pin<Gpio11, hal::gpio::PushPullOutput>,
//                       Pin<Gpio10, hal::gpio::FloatingInput>>
// }

impl ESP32ControlInterface for EspControlPins {
    //type Error = IOError;
    // TODO: add error handling
    fn esp_select(&mut self) {
        self.cs.set_low().unwrap();
    }

    //   fn esp_deselect(&mut self) -> Result<(), Error<Self::Error>> {
    //     //   self.esp_pins.cs.set_high().unwrap();
    //     self.esp_pins.cs.set_high().map_err(|e| IOError::Pin(e))?;
    //   }

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

pub struct EspControlPins {
    pub cs: Pin<Gpio7, hal::gpio::PushPullOutput>,
    pub gpio0: Pin<Gpio2, hal::gpio::PushPullOutput>,
    pub resetn: Pin<Gpio11, hal::gpio::PushPullOutput>,
    pub ack: Pin<Gpio10, hal::gpio::FloatingInput>,
}
