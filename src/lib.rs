//! esp32-wroom-rp
//! 
//! Rust-based Espressif ESP32-WROOM WiFi hardware abstraction layer for RP2040 series microcontroller.
//! Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.
//! Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.
//! 
//! NOTE This crate is still under active development. This API will remain volatile until 1.0.0


pub struct Wifi<C, I> {
  command_handler: C,
}

pub struct FirmwareVersion {
    major: u8,
    minor: u8,
    patch: u8
}

impl FirmwareVersion {
    fn new(version: [u8]) -> FirmwareVersion {
        self.parse(version)
    }

    // Takes in 5 bytes (e.g. 1.7.4) and returns a FirmwareVersion instance
    fn parse(version: [u8) -> FirmwareVersion {
        // TODO: real implementation
        FirmwareVersion {
            major: 1,
            minor: 7,
            patch: 4
        }
    }
}

impl<C, I> Wifi<C, I> {
    fn connect() -> Result<T> (
        self.command_handler.start_client_tcp()
    );

    fn get_firmware_version() -> Result<FirmwareVersion, Error> {
      self.command_handler.get_fw_version()
    }
}

impl NinaCommandHandler for SpiCommandHandler {
    
    fn start_client_tcp (
        // TODO: implement a trait interface and set of structs for different parameter sets, e.g. SocketType
        self.io_interface.send_commmad(START_CLIENT_TCP, [ip, port])
    )

    fn get_fw_version -> Result<FirmwareVersion, Error> (
      self.io_interface.send_command(GET_FW_VERSION, [])
    )
}

trait NinaCommandHandler {
  const START_CLIENT_TCP: u8 = 0x2du8;
  const GET_FW_VERSION: u8 = 0x37u8;

  type Error;

  fn start_client_tcp(&self) -> Result<FirmwareVersion, Error>;

  fn get_fw_version(&self) -> Result<FirmwareVersion, Error>;
}

struct SpiCommandHandler<I> {
    pins: 
    io_interface: I
}

// struct I2C {}

// impl NinaCommandHandler for I2C{
//     fn send_cmd()
// }

trait IoHandler {
  fn send_command (
    //   wait_for_esp_select()
  ) ;
}


// impl Deref for IoHandler {
//     fn deref {
//         esp_deselect()
//         super()
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]

    fn firmware_parse_returns_a_populated_firmware_struct() {
        
    } 
}