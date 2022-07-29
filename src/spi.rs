use super::io_interface::IoInterface;
use super::{Error, FirmwareVersion, Interface, Params, WifiCommon};
use embedded_hal::spi::blocking::Transfer;

#[derive(Debug, Default)]
pub struct Wifi<SPI, IO> {
    common: WifiCommon<SPIInterface<SPI, IO>>,
}

#[derive(Debug, Default)]
struct SPIInterface<SPI, IO> {
    spi: SPI,
    io: IO,
}

impl<SPI, IO> Interface for SPIInterface<SPI, IO>
where
    SPI: Transfer<u8>,
    IO: IoInterface,
{
    type Error = SPIError<SPI::Error, IO::Error>;

    // These may be elevated to the NinaCommandHandler abstraction layer
    fn get_firmware_version(&self) -> Result<FirmwareVersion, self::Error> {
        // self.io.esp_deselect().map_err(|e| SPIError::IO(e))?;
        Ok(FirmwareVersion::new([0x31, 0x2e, 0x37, 0x2e, 0x34])) // 1.7.4
    }

    fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, self::Error> {
        Ok(FirmwareVersion::new([0x31, 0x2e, 0x37, 0x2e, 0x34])) // 1.7.4
    }
}

/// Error which occurred during an SPI transaction
#[derive(Clone, Copy, Debug)]
pub enum SPIError<SPIE, IOE> {
    /// The SPI implementation returned an error
    SPI(SPIE),
    /// The GPIO implementation returned an error when changing the chip-select pin state
    IO(IOE),
}

//   impl<C, I, Io> Wifi<C, I, Io>
//   where
//   // This is a trait defined in embedded_hal and implemented in rp2040_hal https://docs.rs/embedded-hal/0.2.7/embedded_hal/blocking/spi/trait.Transfer.html
//   I: Transfer<Error = Error>,
//   Io: IoInterface,
//   C: NinaCommandHandler
//   {
//       fn new(bus_interface: I, io_interface: Io) -> Result<Wifi<C, I, Io>, Error> {
//         Ok(
//             Wifi {
//                 command_handler: SpiCommandHandler::new(bus_interface, io_interface)
//             }
//         )
//       }

//       fn init() {}
//       // fn init_with_config(config: Configuration) {}

//       fn get_firmware_version(&self) -> Result<FirmwareVersion, Error> {
//           self.command_handler.get_firmware_version()
//       }
//   }

// struct SpiCommandHandler<I: IoInterface, Spi> {
//     io_interface: I,
//     bus_interface: Spi
// }

// impl<I: IoInterface, Spi> SpiCommandHandler<I, Spi> {
//     fn new(bus_interface: Spi, io_interface: I) -> SpiCommandHandler<I, Spi> {
//       SpiCommandHandler {
//         bus_interface: bus_interface,
//         io_interface: io_interface
//       }
//     }
//     fn send_command(&self, command: NinaCommand, parameters: [u8; 5]) -> Result<FirmwareVersion, Error> {
//         // self.bus_interface.transfer()
//         Ok(FirmwareVersion::new([0x31,0x2e,0x37,0x2e,0x34])) // 1.7.4
//       }
// }

// impl<I: IoInterface, Spi> NinaCommandHandler for SpiCommandHandler<I, Spi> {

//     fn start_client_tcp(&self, params: Params) -> Result<FirmwareVersion, Error> {
//         self.send_command(NinaCommand::StartClientTcp, params)
//     }

//     fn get_firmware_version(&self) -> Result<FirmwareVersion, Error> {
//         self.send_command(NinaCommand::GetFirmwareVersion, [0; 5])
//     }
// }
