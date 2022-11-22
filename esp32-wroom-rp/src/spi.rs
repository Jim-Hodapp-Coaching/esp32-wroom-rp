//! Serial Peripheral Interface (SPI) for Wifi

use super::gpio::EspControlInterface;
use super::protocol::{
    NinaByteParam, NinaCommand, NinaNoParams, NinaParam, NinaProtocolHandler, NinaSmallArrayParam,
    ProtocolInterface,
};

use super::{Error, FirmwareVersion, ARRAY_LENGTH_PLACEHOLDER};
use super::network::{IpAddress, NetworkError, Socket};
use super::protocol::operation::Operation;
use super::protocol::ProtocolError;
use super::wifi::ConnectionStatus;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use core::convert::Infallible;

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

// All SPI-specific aspects of the NinaProtocolHandler go here in this struct impl
impl<'a, S, C> ProtocolInterface for NinaProtocolHandler<'a, S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    fn init(&mut self) {
        // Chip select is active-low, so we'll initialize it to a driven-high state
        self.control_pins.init();
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.control_pins.reset(delay);
    }

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, Error> {
        // TODO: improve the ergonomics around with_no_params()
        let operation =
            Operation::new(NinaCommand::GetFwVersion, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation)?;

        let result = self.receive(&operation)?;

        Ok(FirmwareVersion::new(result)) // e.g. 1.7.4
    }

    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        let operation = Operation::new(NinaCommand::SetPassphrase, 1)
            .param(NinaSmallArrayParam::new(ssid))
            .param(NinaSmallArrayParam::new(passphrase));

        self.execute(&operation)?;

        self.receive(&operation)?;
        Ok(())
    }

    fn get_conn_status(&mut self) -> Result<ConnectionStatus, Error> {
        let operation =
            Operation::new(NinaCommand::GetConnStatus, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation)?;

        let result = self.receive(&operation)?;

        // TODO:
        Ok(ConnectionStatus::Connected)
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        let dummy_param = NinaByteParam::from_bytes(&[ControlByte::Dummy as u8]);
        let operation = Operation::new(NinaCommand::Disconnect, 1).param(dummy_param);

        self.execute(&operation)?;

        self.receive(&operation)?;

        Ok(())
    }

    fn set_dns_config(&mut self, ip1: IpAddress, ip2: Option<IpAddress>) -> Result<(), Error> {
        // FIXME: refactor Operation so it can take different NinaParam types
        let operation = Operation::new(NinaCommand::SetDNSConfig, 1)
            // FIXME: first param should be able to be a NinaByteParam:
            .param(NinaSmallArrayParam::from_bytes(&[1]))
            .param(NinaSmallArrayParam::from_bytes(&ip1))
            .param(NinaSmallArrayParam::from_bytes(&ip2.unwrap_or_default()));

        self.execute(&operation)?;

        self.receive(&operation)?;

        Ok(())
    }

    fn req_host_by_name(&mut self, hostname: &str) -> Result<u8, Error> {
        let operation =
            Operation::new(NinaCommand::ReqHostByName, 1).param(NinaSmallArrayParam::new(hostname));

        self.execute(&operation)?;

        let result = self.receive(&operation)?;

        if result[0] != 1u8 {
            return Err(NetworkError::DnsResolveFailed.into());
        }

        Ok(result[0])
    }

    fn get_host_by_name(&mut self) -> Result<[u8; 8], Error> {
        let operation =
            Operation::new(NinaCommand::GetHostByName, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation)?;

        let result = self.receive(&operation)?;

        Ok(result)
    }

    fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error> {
        self.req_host_by_name(hostname)?;

        let dummy: IpAddress = [255, 255, 255, 255];

        let result = self.get_host_by_name()?;

        let (ip_slice, _) = result.split_at(4);
        let mut ip_address: IpAddress = [0; 4];
        ip_address.clone_from_slice(ip_slice);

        if ip_address != dummy {
            Ok(ip_address)
        } else {
            Err(NetworkError::DnsResolveFailed.into())
        }
    }

    fn get_socket(&mut self) -> Result<Socket, Error> {
        let operation =
            Operation::new(NinaCommand::GetSocket, 1).with_no_params(NinaNoParams::new(""));

        self.execute(&operation)?;

        let result = self.receive(&operation)?;

        Ok(result[0])
    }
}

impl<'a, S, C> NinaProtocolHandler<'a, S, C>
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
        let result = self.send_cmd(&operation.command, number_of_params);

        // Only send params if they are present
        if operation.has_params {
            operation.params.iter().for_each(|param| {
                self.send_param(param).ok();
                param_size += param.length();
            });

            self.send_end_cmd().ok();

            // This is to make sure we align correctly
            // 4 (start byte, command byte, number of params, end byte) + 1 byte for each param + the sum of all param lengths
            // See https://github.com/arduino/nina-fw/blob/master/main/CommandHandler.cpp#L2153 for the actual equation.
            let command_size: u16 = 4u16 + number_of_params as u16 + param_size;
            self.pad_to_multiple_of_4(command_size);
        }
        self.control_pins.esp_deselect();

        result
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

    fn send_cmd(&mut self, cmd: &NinaCommand, num_params: u8) -> Result<(), Error> {
        let buf: [u8; 3] = [
            ControlByte::Start as u8,
            (*cmd as u8) & !(ControlByte::Reply as u8),
            num_params,
        ];

        for byte in buf {
            let write_buf = &mut [byte];
            self.bus.transfer(write_buf).ok();
        }

        if num_params == 0 {
            self.send_end_cmd().ok();
        }
        Ok(())
    }

    fn wait_response_cmd(
        &mut self,
        cmd: &NinaCommand,
        num_params: u8,
    ) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], Error> {
        self.check_start_cmd()?;
        let byte_to_check: u8 = *cmd as u8 | ControlByte::Reply as u8;
        let result = self.read_and_check_byte(&byte_to_check).ok().unwrap();
        // Ensure we see a cmd byte
        if !result {
            return Err(ProtocolError::InvalidCommand.into());
        }

        let result = self.read_and_check_byte(&num_params).unwrap();
        // Ensure we see the number of params we expected to receive back
        if !result {
            return Err(ProtocolError::InvalidNumberOfParameters.into());
        }

        let num_params_to_read = self.get_byte().ok().unwrap() as usize;

        // TODO: use a constant instead of inline params max == 8
        if num_params_to_read > 8 {
            return Err(ProtocolError::TooManyParameters.into());
        }

        let mut params: [u8; ARRAY_LENGTH_PLACEHOLDER] = [0; 8];
        for (index, _param) in params.into_iter().enumerate() {
            params[index] = self.get_byte().ok().unwrap()
        }
        let control_byte: u8 = ControlByte::End as u8;
        self.read_and_check_byte(&control_byte).ok();

        Ok(params)
    }

    fn send_end_cmd(&mut self) -> Result<(), Infallible> {
        let end_command: &mut [u8] = &mut [ControlByte::End as u8];
        self.bus.transfer(end_command).ok();
        Ok(())
    }

    fn get_byte(&mut self) -> Result<u8, Infallible> {
        let word_out = &mut [ControlByte::Dummy as u8];
        let word = self.bus.transfer(word_out).ok().unwrap();
        Ok(word[0] as u8)
    }

    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, Error> {
        let retry_limit: u16 = 1000u16;

        for _ in 0..retry_limit {
            let byte_read = self.get_byte().ok().unwrap();
            if byte_read == ControlByte::Error as u8 {
                return Err(ProtocolError::NinaProtocolVersionMismatch.into());
            } else if byte_read == wait_byte {
                return Ok(true);
            }
        }
        Err(ProtocolError::CommunicationTimeout.into())
    }

    fn check_start_cmd(&mut self) -> Result<bool, Error> {
        self.wait_for_byte(ControlByte::Start as u8)
    }

    fn read_and_check_byte(&mut self, check_byte: &u8) -> Result<bool, Infallible> {
        let byte = self.get_byte().ok().unwrap();
        Ok(&byte == check_byte)
    }

    fn send_param<P: NinaParam>(&mut self, param: &P) -> Result<(), Infallible> {
        self.send_param_length(param)?;

        for byte in param.data().iter() {
            self.bus.transfer(&mut [*byte]).ok();
        }
        Ok(())
    }

    fn send_param_length<P: NinaParam>(&mut self, param: &P) -> Result<(), Infallible> {
        for byte in param.length_as_bytes().into_iter() {
            self.bus.transfer(&mut [byte]).ok();
        }
        Ok(())
    }

    fn pad_to_multiple_of_4(&mut self, mut command_size: u16) {
        while command_size % 4 != 0 {
            self.get_byte().ok();
            command_size += 1;
        }
    }
}
