//! Serial Peripheral Interface (SPI) for Wifi

use super::pins::ESP32ControlInterface;
use super::{Error, FirmwareVersion, Interface, Params, WifiCommon};
use embedded_hal_02::blocking::spi::Transfer;

const PARAMS_ARRAY_LEN: usize = 8;

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
    Dummy = 0xFFu8,
    Error = 0xEFu8,
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
    SPI: Transfer<u8>,
    PINS: ESP32ControlInterface,
{
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error> {
        // Chip select is active-low, so we'll initialize it to a driven-high state
        self.pins.init();

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

        if num_params == 0 {
            self.send_end_cmd().ok().unwrap();
        }
        Ok(())
    }

    fn wait_response_cmd(
        &mut self,
        cmd: NinaCommand,
        num_params: u8,
    ) -> Result<[u8; PARAMS_ARRAY_LEN], SPIError<SPI, PINS>> {
        self.check_start_cmd().ok().unwrap();

        let result = self.read_and_check_byte(cmd as u8 | ControlByte::Reply as u8)?;
        // Ensure we see a cmd byte
        if !result {
            return Err(SPIError::Misc);
        }

        let result = self.read_and_check_byte(num_params)?;
        // Ensure we see the number of params we expected to receive back
        if !result {
            return Err(SPIError::Misc);
        }

        let num_params_to_read = self.get_param()? as usize;

        if num_params_to_read > PARAMS_ARRAY_LEN {
            return Err(SPIError::Misc);
        }

        let mut params: [u8; PARAMS_ARRAY_LEN] = [0; PARAMS_ARRAY_LEN];
        for i in 0..num_params_to_read {
            params[i] = self.get_param().ok().unwrap()
        }

        self.read_and_check_byte(ControlByte::End as u8)?;

        Ok(params)
    }

    fn send_end_cmd(&mut self) -> Result<(), SPIError<SPI, PINS>> {
        let end_command: &mut [u8] = &mut [ControlByte::End as u8];
        self.spi.transfer(end_command).ok().unwrap();
        Ok(())
    }

    fn get_param(&mut self) -> Result<u8, SPIError<SPI, PINS>> {
        // Blocking read, don't return until we've read a byte successfully
        loop {
            let word_out = &mut [ControlByte::Dummy as u8];
            match self.spi.transfer(word_out) {
                Ok(word) => {
                    let byte: u8 = word[0] as u8;
                    return Ok(byte);
                }
                Err(_e) => {
                    continue;
                }
            }
        }
    }

    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, SPIError<SPI, PINS>> {
        let mut timeout: u16 = 1000u16;

        loop {
            match self.get_param() {
                Ok(byte_read) => {
                    if byte_read == ControlByte::Error as u8 {
                        return Err(SPIError::Misc);
                    } else if byte_read == wait_byte {
                        return Ok(true);
                    } else if timeout == 0 {
                        return Err(SPIError::Timeout);
                    }
                    timeout -= 1;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    fn check_start_cmd(&mut self) -> Result<bool, SPIError<SPI, PINS>> {
        self.wait_for_byte(ControlByte::Start as u8)
    }

    fn read_and_check_byte(&mut self, check_byte: u8) -> Result<bool, SPIError<SPI, PINS>> {
        match self.get_param() {
            Ok(byte_out) => {
                return Ok(byte_out == check_byte);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }
}

/// Error which occurred during an SPI transaction
#[derive(Clone, Copy, Debug)]
pub enum SPIError<SPIE, IOE> {
    /// The SPI implementation returned an error
    SPI(SPIE),
    /// The GPIO implementation returned an error when changing the chip-select pin state
    IO(IOE),
    /// Timeout
    Timeout,
    Misc,
}
