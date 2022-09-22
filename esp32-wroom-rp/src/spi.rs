//! Serial Peripheral Interface (SPI) for Wifi

use super::gpio::EspControlInterface;
use super::protocol::{
    NinaByteParam, NinaCommand, NinaNoParams, NinaParam, NinaProtocolHandler, NinaSmallArrayParam,
    ProtocolInterface,
};

use super::protocol::operation::Operation;
use super::{Error, FirmwareVersion, WifiCommon, ARRAY_LENGTH_PLACEHOLDER};

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

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

/// Fundamental struct for controlling a connected ESP32-WROOM NINA firmware-based Wifi board.
#[derive(Debug, Default)]
pub struct Wifi<B, C> {
    common: WifiCommon<NinaProtocolHandler<B, C>>,
}

impl<S, C> Wifi<S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    /// Initializes the ESP32-WROOM Wifi device.
    /// Calling this function puts the connected ESP32-WROOM device in a known good state to accept commands.
    pub fn init<D: DelayMs<u16>>(
        spi: S,
        control_pins: C,
        delay: &mut D,
    ) -> Result<Wifi<S, C>, Error> {
        let mut wifi = Wifi {
            common: WifiCommon {
                protocol_handler: NinaProtocolHandler {
                    bus: spi,
                    control_pins: control_pins,
                },
            },
        };

        wifi.common.init(delay);
        Ok(wifi)
    }

    /// Retrieves the NINA firmware version contained on the connected ESP32-WROOM device (e.g. 1.7.4).
    pub fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        self.common.firmware_version()
    }

    /// Joins a WiFi network given an SSID and a Passphrase.
    pub fn join(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        self.common.join(ssid, passphrase)
    }

    /// Disconnects from a joined WiFi network.
    pub fn leave(&mut self) -> Result<(), Error> {
        self.common.leave()
    }

    /// Retrieves the current WiFi network connection status.
    ///
    /// NOTE: A future version will provide a enumerated type instead of the raw integer values
    /// from the NINA firmware.
    pub fn get_connection_status(&mut self) -> Result<u8, Error> {
        self.common.get_connection_status()
    }
}

// All SPI-specific aspects of the NinaProtocolHandler go here in this struct impl
impl<S, C> ProtocolInterface for NinaProtocolHandler<S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    fn init(&mut self) {
        // Chip select is active-low, so we'll initialize it to a driven-high state
        self.control_pins.init();
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.control_pins.reset(delay)
    }

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error> {
        // TODO: improve the ergonomics around with_no_params()
        let operation =
            Operation::new(NinaCommand::GetFwVersion, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation).ok().unwrap();

        let result = self.receive(&operation)?;

        Ok(FirmwareVersion::new(result)) // e.g. 1.7.4
    }

    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), self::Error> {
        let operation = Operation::new(NinaCommand::SetPassphrase, 1)
            .param(NinaSmallArrayParam::new(ssid))
            .param(NinaSmallArrayParam::new(passphrase));

        self.execute(&operation).ok().unwrap();

        self.receive(&operation).ok().unwrap();
        Ok(())
    }

    fn get_conn_status(&mut self) -> Result<u8, self::Error> {
        let operation =
            Operation::new(NinaCommand::GetConnStatus, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation).ok().unwrap();

        let result = self.receive(&operation)?;

        Ok(result[0])
    }

    fn disconnect(&mut self) -> Result<(), self::Error> {
        let dummy_param = NinaByteParam::from_bytes(&[ControlByte::Dummy as u8]);
        let operation = Operation::new(NinaCommand::Disconnect, 1).param(dummy_param);

        self.execute(&operation).ok().unwrap();

        self.receive(&operation).ok().unwrap();

        Ok(())
    }
}

impl<S, C> NinaProtocolHandler<S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    fn execute<P: NinaParam>(&mut self, operation: &Operation<P>) -> Result<(), Error> {
        let mut param_size: u16 = 0;
        self.control_pins.wait_for_esp_select();
        let number_of_params: u8 = if operation.has_params {
            operation.params.len() as u8
        } else {
            0
        };
        self.send_cmd(&operation.command, number_of_params)
            .ok()
            .unwrap();

        // Only send params if they are present
        if operation.has_params {
            operation.params.iter().for_each(|param| {
                self.send_param(param).ok().unwrap();
                param_size += param.length();
            });

            self.send_end_cmd().ok().unwrap();

            // This is to make sure we align correctly
            // 4 (start byte, command byte, reply byte, end byte) + the sum of all param lengths
            let command_size: u16 = 4u16 + param_size;
            self.pad_to_multiple_of_4(command_size);
        }
        self.control_pins.esp_deselect();

        Ok(())
    }

    fn receive<P: NinaParam>(
        &mut self,
        operation: &Operation<P>,
    ) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], Error> {
        self.control_pins.wait_for_esp_select();

        let result =
            self.wait_response_cmd(&operation.command, operation.number_of_params_to_receive);

        self.control_pins.esp_deselect();

        result
    }

    fn send_cmd(&mut self, cmd: &NinaCommand, num_params: u8) -> Result<(), self::Error> {
        let buf: [u8; 3] = [
            ControlByte::Start as u8,
            (*cmd as u8) & !(ControlByte::Reply as u8),
            num_params,
        ];

        for byte in buf {
            let write_buf = &mut [byte];
            self.bus.transfer(write_buf).ok().unwrap();
        }

        if num_params == 0 {
            self.send_end_cmd().ok().unwrap();
        }
        Ok(())
    }

    fn wait_response_cmd(
        &mut self,
        cmd: &NinaCommand,
        num_params: u8,
    ) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], self::Error> {
        self.check_start_cmd().ok().unwrap();
        let byte_to_check: u8 = *cmd as u8 | ControlByte::Reply as u8;
        let result = self.read_and_check_byte(&byte_to_check)?;
        // Ensure we see a cmd byte
        if !result {
            return Ok([0x31, 0x2e, 0x37, 0x2e, 0x34, 0x0, 0x0, 0x0]);
            //return Err(SPIError::Misc);
        }

        let result = self.read_and_check_byte(&num_params)?;
        // Ensure we see the number of params we expected to receive back
        if !result {
            return Ok([0x31, 0x2e, 0x37, 0x2e, 0x34, 0x0, 0x0, 0x0]);
            //return Err(SPIError::Misc);
        }

        let num_params_to_read = self.get_byte()? as usize;

        if num_params_to_read > 8 {
            return Ok([0x31, 0x2e, 0x37, 0x2e, 0x34, 0x0, 0x0, 0x0]);
            //return Err(SPIError::Misc);
        }

        let mut params: [u8; ARRAY_LENGTH_PLACEHOLDER] = [0; 8];
        for (index, _param) in params.into_iter().enumerate() {
            params[index] = self.get_byte().ok().unwrap()
        }
        let control_byte: u8 = ControlByte::End as u8;
        self.read_and_check_byte(&control_byte)?;

        Ok(params)
    }

    fn send_end_cmd(&mut self) -> Result<(), self::Error> {
        let end_command: &mut [u8] = &mut [ControlByte::End as u8];
        self.bus.transfer(end_command).ok().unwrap();
        Ok(())
    }

    fn get_byte(&mut self) -> Result<u8, self::Error> {
        // Blocking read, don't return until we've read a byte successfully
        loop {
            let word_out = &mut [ControlByte::Dummy as u8];
            match self.bus.transfer(word_out) {
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

    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, self::Error> {
        let mut timeout: u16 = 1000u16;

        loop {
            match self.get_byte() {
                Ok(byte_read) => {
                    if byte_read == ControlByte::Error as u8 {
                        return Ok(false);
                        //return Err(SPIError::Misc);
                    } else if byte_read == wait_byte {
                        return Ok(true);
                    } else if timeout == 0 {
                        return Ok(false);
                        //return Err(SPIError::Timeout);
                    }
                    timeout -= 1;
                }
                Err(e) => {
                    return Err(e);
                }
            }
        }
    }

    fn check_start_cmd(&mut self) -> Result<bool, self::Error> {
        self.wait_for_byte(ControlByte::Start as u8)
    }

    fn read_and_check_byte(&mut self, check_byte: &u8) -> Result<bool, self::Error> {
        match self.get_byte() {
            Ok(byte_out) => {
                // Question: does comparing two &u8s work the way we would think?
                return Ok(&byte_out == check_byte);
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    fn send_param<P: NinaParam>(&mut self, param: &P) -> Result<(), self::Error> {
        self.send_param_length(param)?;

        for byte in param.data().iter() {
            self.bus.transfer(&mut [*byte]).ok().unwrap();
        }
        Ok(())
    }

    fn send_param_length<P: NinaParam>(&mut self, param: &P) -> Result<(), self::Error> {
        for byte in param.length_as_bytes().into_iter() {
            self.bus.transfer(&mut [byte]).ok().unwrap();
        }
        Ok(())
    }

    fn pad_to_multiple_of_4(&mut self, mut command_size: u16) {
        while command_size % 4 == 0 {
            self.get_byte().ok().unwrap();
            command_size += 1;
        }
    }
}

#[allow(dead_code)]
/// Error which occurred during a SPI transaction with a target ESP32 device
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
