use super::pins::ESP32ControlInterface;
use super::{Error, FirmwareVersion, Interface, Params, WifiCommon};
use embedded_hal::spi::blocking::Transfer;

#[derive(Debug, Default)]
pub struct Wifi<SPI, PINS> {
    common: WifiCommon<SPIInterface<SPI, PINS>>,
}

impl<SPI, PINS> Wifi<SPI, PINS>
where
    // FIXME: figure the proper trait bound to use that allows this to compile
    //SPI: Transfer<u8>,
    PINS: ESP32ControlInterface,
{
    pub fn init(spi: SPI, pins: PINS) -> Result<Wifi<SPI, PINS>, Error> {
        Ok(Wifi {
            common: WifiCommon {
                interface: SPIInterface { spi: spi, pins: pins },
            },
        })
    }

    pub fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        self.common.firmware_version()
    }
}

#[derive(Debug, Default)]
struct SPIInterface<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<SPI, PINS> Interface for SPIInterface<SPI, PINS>
where
    // FIXME: figure the proper trait bound to use that allows this to compile
    //SPI: Transfer<u8>,
    PINS: ESP32ControlInterface,
{
    //type Error = SPIError<SPI::Error, PINS::Error>;

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error> {
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
