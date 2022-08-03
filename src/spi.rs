use super::io_interface::IoInterface;
use super::{Error, FirmwareVersion, Interface, Params, WifiCommon};
use embedded_hal::spi::blocking::Transfer;

#[derive(Debug, Default)]
pub struct Wifi<SPI, IO> {
    common: WifiCommon<SPIInterface<SPI, IO>>,
}

impl<SPI, IO> Wifi<SPI, IO>
where
    SPI: Transfer<u8>,
    IO: IoInterface,
{
    pub fn init(spi: SPI, io: IO) -> Result<Wifi<SPI, IO>, Error> {
        Ok(Wifi {
            common: WifiCommon {
                interface: SPIInterface { spi: spi, io: io },
            },
        })
    }
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

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error> {
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
