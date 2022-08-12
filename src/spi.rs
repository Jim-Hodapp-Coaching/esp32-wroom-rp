use super::pins::ESP32ControlInterface;
use super::{Error, FirmwareVersion, Interface, Params, WifiCommon};
use embedded_hal_02::blocking::spi::Transfer;

// TODO: this should eventually move into NinaCommandHandler
#[repr(u8)]
#[derive(Debug)]
enum NinaCommand {
    StartClientTcp = 0x2Du8,
    GetFwVersion = 0x37u8,
}

// TODO: this should eventually move into NinaCommandHandler
#[repr(u8)]
#[derive(Debug)]
enum ControlByte {
    Start = 0xE0u8,
    End = 0xEEu8,
    Reply = 1u8 << 7u8,
}

#[derive(Debug, Default)]
pub struct Wifi<SPI, PINS> {
    common: WifiCommon<SPIInterface<SPI, PINS>>,
}

impl<SPI, PINS> Wifi<SPI, PINS>
where
    SPI: Transfer<u8>,
    PINS: ESP32ControlInterface,
{
    pub fn init(spi: SPI, pins: PINS) -> Result<Wifi<SPI, PINS>, Error> {
        Ok(Wifi {
            common: WifiCommon {
                interface: SPIInterface {
                    spi: spi,
                    pins: pins,
                },
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
    SPI: Transfer<u8>,
    PINS: ESP32ControlInterface,
{
    //type Error = SPIError<SPI::Error, PINS::Error>;

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error> {
        self.pins.wait_for_esp_select();

        self.send_cmd(NinaCommand::GetFwVersion, 0).ok().unwrap();

        self.pins.esp_deselect();
        self.pins.wait_for_esp_select();

        let bytes = self
            .wait_response_cmd(NinaCommand::GetFwVersion, 1)
            .ok()
            .unwrap();

        self.pins.esp_deselect();
        Ok(FirmwareVersion::new(bytes)) // 1.7.4
    }
}

// TODO: Does this struct impl break out and become a generic NinaCommandHandler struct shared
// between SPI and I2C?
impl<SPI, PINS> SPIInterface<SPI, PINS>
where
    SPI: Transfer<u8>,
{
    fn send_cmd(&mut self, cmd: NinaCommand, num_params: u8) -> Result<(), SPIError<SPI, PINS>> {
        let buf: [u8; 3] = [
            ControlByte::Start as u8,
            (cmd as u8) & !(ControlByte::Reply as u8),
            num_params,
        ];
        for byte in buf {
            let write_buf = &mut [byte];

            self.spi.transfer(write_buf).ok().unwrap();
        }
        Ok(())
    }

    fn wait_response_cmd(
        &mut self,
        cmd: NinaCommand,
        num_params: u8,
    ) -> Result<[u8; 5], SPIError<SPI, PINS>> {
        Ok([0x31, 0x2e, 0x37, 0x2e, 0x34])
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
