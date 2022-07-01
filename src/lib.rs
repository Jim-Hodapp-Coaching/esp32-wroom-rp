//! esp32-wroom-rp
//! 
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//! 
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0


struct Wifi<C, I> {
  command_handler: C,
}

impl<C, I> Wifi<C, I> {
    fn connect() -> Result<T> (
        command_handler.start_client_tcp()
    );
}

struct NinaCommandHandler<I> {
    pins: 
    io_handler: I
}

impl NinaCommandHandler for Spi {
    fn start_client_tcp (
        self.io_interface.send_cmd(START_CLIENT_TCP, [ip, port])
    )
}

trait NinaCommandHandler {
  fn send_cmd(&self) -> (
  );
}

// struct I2C {}

// impl NinaCommandHandler for I2C{
//     fn send_cmd()
// }

trait IoHandler {
  fn send_cmd (
    //   wait_for_esp_select()
  ) 
}


// impl Deref for IoHandler {
//     fn deref {
//         esp_deselect()
//         super()
//     }
// }

