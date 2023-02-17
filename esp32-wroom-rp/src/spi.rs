//! Serial Peripheral Interface (SPI)
//!
//! Contains all SPI bus related structs, types and errors. Also responsible for
//! implementing WifiNINA protocol communication over a selected SPI interface.
//!
//! Note: Currently everything in this file is private and considered internal to the crate.
//!
use core::convert::Infallible;

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use super::gpio::EspControlInterface;
use super::network::{ConnectionState, IpAddress, NetworkError, Port, Socket, TransportMode};
use super::protocol::operation::Operation;
use super::protocol::{
    NinaByteParam, NinaCommand, NinaConcreteParam, NinaLargeArrayParam, NinaParam,
    NinaProtocolHandler, NinaResponseBuffer, NinaSmallArrayParam, NinaWordParam, ProtocolError,
    ProtocolInterface, MAX_NINA_PARAMS, MAX_NINA_RESPONSE_LENGTH,
};
use super::wifi::ConnectionStatus;
use super::{Error, FirmwareVersion};

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
        self.control_pins.reset(delay);
    }

    fn get_fw_version(&mut self) -> Result<FirmwareVersion, Error> {
        let operation = Operation::new(NinaCommand::GetFwVersion);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;
        let (version, _) = result.split_at(5);

        Ok(FirmwareVersion::new(version)) // e.g. 1.7.4
    }

    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        let operation = Operation::new(NinaCommand::SetPassphrase)
            .param(NinaSmallArrayParam::new(ssid)?)
            .param(NinaSmallArrayParam::new(passphrase)?);

        self.execute(&operation)?;

        self.receive(&operation, 1)?;
        Ok(())
    }

    fn get_conn_status(&mut self) -> Result<ConnectionStatus, Error> {
        let operation = Operation::new(NinaCommand::GetConnStatus);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;

        Ok(ConnectionStatus::from(result[0]))
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        let dummy_param = NinaByteParam::from_bytes(&[ControlByte::Dummy as u8]);
        let operation =
            Operation::new(NinaCommand::Disconnect).param(dummy_param.unwrap_or_default());

        self.execute(&operation)?;

        self.receive(&operation, 1)?;

        Ok(())
    }

    fn set_dns_config(&mut self, ip1: IpAddress, ip2: Option<IpAddress>) -> Result<(), Error> {
        // FIXME: refactor Operation so it can take different NinaParam types
        let operation = Operation::new(NinaCommand::SetDNSConfig)
            // FIXME: first param should be able to be a NinaByteParam:
            .param(NinaByteParam::from_bytes(&[1])?)
            .param(NinaSmallArrayParam::from_bytes(&ip1)?)
            .param(NinaSmallArrayParam::from_bytes(&ip2.unwrap_or_default())?);

        self.execute(&operation)?;

        self.receive(&operation, 1)?;

        Ok(())
    }

    fn req_host_by_name(&mut self, hostname: &str) -> Result<u8, Error> {
        let operation =
            Operation::new(NinaCommand::ReqHostByName).param(NinaSmallArrayParam::new(hostname)?);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;

        if result[0] != 1u8 {
            return Err(NetworkError::DnsResolveFailed.into());
        }

        Ok(result[0])
    }

    fn get_host_by_name(&mut self) -> Result<NinaResponseBuffer, Error> {
        let operation = Operation::new(NinaCommand::GetHostByName);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;

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
        let operation = Operation::new(NinaCommand::GetSocket);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;

        Ok(result[0])
    }

    fn start_client_tcp(
        &mut self,
        socket: Socket,
        ip: IpAddress,
        port: Port,
        mode: &TransportMode,
    ) -> Result<(), Error> {
        let port_as_bytes = [((port & 0xff00) >> 8) as u8, (port & 0xff) as u8];
        let operation = Operation::new(NinaCommand::StartClientTcp)
            .param(NinaSmallArrayParam::from_bytes(&ip)?)
            .param(NinaWordParam::from_bytes(&port_as_bytes)?)
            .param(NinaByteParam::from_bytes(&[socket])?)
            .param(NinaByteParam::from_bytes(&[*mode as u8])?);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;
        if result[0] == 1 {
            Ok(())
        } else {
            Err(NetworkError::ConnectFailed.into())
        }
    }

    // TODO: passing in TransportMode but not using, for now. It will become a way
    // of stopping the right kind of client (e.g. TCP, vs UDP)
    fn stop_client_tcp(&mut self, socket: Socket, _mode: &TransportMode) -> Result<(), Error> {
        let operation =
            Operation::new(NinaCommand::StopClientTcp).param(NinaByteParam::from_bytes(&[socket])?);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;
        if result[0] == 1 {
            Ok(())
        } else {
            Err(NetworkError::DisconnectFailed.into())
        }
    }

    fn get_client_state_tcp(&mut self, socket: Socket) -> Result<ConnectionState, Error> {
        let operation = Operation::new(NinaCommand::GetClientStateTcp)
            .param(NinaByteParam::from_bytes(&[socket])?);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;
        // TODO: Determine whether or not any ConnectionState variants should be considered
        // an error.
        Ok(ConnectionState::from(result[0]))
    }

    fn send_data(&mut self, data: &str, socket: Socket) -> Result<[u8; 1], Error> {
        let operation = Operation::new(NinaCommand::SendDataTcp)
            .param(NinaLargeArrayParam::from_bytes(&[socket])?)
            .param(NinaLargeArrayParam::new(data)?);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1)?;

        Ok([result[0]])
    }

    fn avail_data_tcp(&mut self, socket: Socket) -> Result<usize, Error> {
        let operation = Operation::new(NinaCommand::AvailDataTcp)
            .param(NinaLargeArrayParam::from_bytes(&[socket])?);

        self.execute(&operation)?;

        let result = self.receive_data16(&operation, 1)?;

        Ok(result[0] as usize)
    }

    // dummy implementations for now

    fn get_data_buf_tcp(
        &mut self,
        socket: Socket,
        available_length: usize,
    ) -> Result<NinaResponseBuffer, Error> {
        let mut response_param_buffer: NinaResponseBuffer = [0; MAX_NINA_RESPONSE_LENGTH];

        Ok(response_param_buffer)
    }

    fn receive_data(&mut self, socket: Socket) -> Result<NinaResponseBuffer, Error> {
        self.avail_data_tcp(socket);
        let mut response_param_buffer: NinaResponseBuffer = [0; MAX_NINA_RESPONSE_LENGTH];

        Ok(response_param_buffer)
    }
}

impl<S, C> NinaProtocolHandler<S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    fn execute<P: NinaParam>(&mut self, operation: &Operation<P>) -> Result<(), Error> {
        let mut total_params_length: u16 = 0;
        let mut total_params_length_size: u16 = 0;

        self.control_pins.wait_for_esp_select();
        let number_of_params: u8 = if !operation.params.is_empty() {
            operation.params.len() as u8
        } else {
            0
        };
        let result = self.send_cmd(&operation.command, number_of_params);

        // Only send params if they are present
        if !operation.params.is_empty() {
            operation.params.iter().for_each(|param| {
                self.send_param(param).ok();

                total_params_length += param.length();
                total_params_length_size += param.length_size() as u16;
            });

            self.send_end_cmd().ok();

            // This is to make sure we align correctly
            // 4 (start byte, command byte, number of params as byte, end byte)
            // + the number of bytes to represent the param length (1 or 2)
            // + the sum of all param lengths
            // See https://github.com/arduino/nina-fw/blob/master/main/CommandHandler.cpp#L2153 for the actual equation.
            let command_size: u16 = 4u16 + total_params_length_size + total_params_length;
            self.pad_to_multiple_of_4(command_size);
        }
        self.control_pins.esp_deselect();

        result
    }

    fn receive<P: NinaParam>(
        &mut self,
        operation: &Operation<P>,
        expected_num_params: u8,
    ) -> Result<NinaResponseBuffer, Error> {
        self.control_pins.wait_for_esp_select();

        self.check_response_ready(&operation.command, expected_num_params)?;

        let result = self.read_response()?;

        self.control_pins.esp_deselect();

        Ok(result)
    }

    fn receive_data16<P: NinaParam>(
        &mut self,
        operation: &Operation<P>,
        expected_num_params: u8,
    ) -> Result<NinaResponseBuffer, Error> {
        self.control_pins.wait_for_esp_select();

        self.check_response_ready(&operation.command, expected_num_params)?;

        let result = self.read_response16()?;

        self.control_pins.esp_deselect();

        Ok(result)
    }

    fn send_cmd(&mut self, cmd: &NinaCommand, num_params: u8) -> Result<(), Error> {
        let buf: [u8; 3] = [
            ControlByte::Start as u8,
            (*cmd as u8) & !(ControlByte::Reply as u8),
            num_params,
        ];

        for byte in buf {
            let write_buf = &mut [byte];
            self.bus.borrow_mut().transfer(write_buf).ok();
        }

        if num_params == 0 {
            self.send_end_cmd().ok();
        }
        Ok(())
    }

    fn read_response(&mut self) -> Result<NinaResponseBuffer, Error> {
        let response_length_in_bytes = self.get_byte().ok().unwrap() as usize;

        if response_length_in_bytes > MAX_NINA_PARAMS {
            return Err(ProtocolError::TooManyParameters.into());
        }

        let mut response_param_buffer: NinaResponseBuffer = [0; MAX_NINA_RESPONSE_LENGTH];
        if response_length_in_bytes > 0 {
            response_param_buffer =
                self.read_response_bytes(response_param_buffer, response_length_in_bytes)?;
        }

        let control_byte: u8 = ControlByte::End as u8;
        self.read_and_check_byte(&control_byte).ok();

        Ok(response_param_buffer)
    }

    fn read_response16(&mut self) -> Result<NinaResponseBuffer, Error> {
        // let response_length_in_bytes = self.get_byte().ok().unwrap() as usize;

        let number_of_params = self.get_byte().unwrap() as usize;

        // if response_length_in_bytes > MAX_NINA_PARAMS {
        //     return Err(ProtocolError::TooManyParameters.into());
        // }

        let mut response_param_buffer: NinaResponseBuffer = [0; MAX_NINA_RESPONSE_LENGTH];
        if number_of_params > 0 {
            let bytes = (self.get_byte().unwrap(), self.get_byte().unwrap());
            defmt::debug!("avail_data_tcp bytes: {:?}", bytes);
            let response_length_as_u16: usize = Self::combine_2_bytes(bytes.0, bytes.1).into();
            defmt::debug!(
                "avail_data_tcp response_length_as_u16: {:?}",
                response_length_as_u16
            );
            response_param_buffer =
                self.read_response_bytes(response_param_buffer, response_length_as_u16)?;
        }

        let control_byte: u8 = ControlByte::End as u8;
        self.read_and_check_byte(&control_byte).ok();

        Ok(response_param_buffer)
    }

    fn check_response_ready(&mut self, cmd: &NinaCommand, num_params: u8) -> Result<(), Error> {
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
        Ok(())
    }

    fn read_response_bytes(
        &mut self,
        mut response_param_buffer: NinaResponseBuffer,
        response_length_in_bytes: usize,
    ) -> Result<NinaResponseBuffer, Error> {
        for byte in response_param_buffer
            .iter_mut()
            .take(response_length_in_bytes)
        {
            *byte = self.get_byte().ok().unwrap();
        }
        Ok(response_param_buffer)
    }

    fn send_end_cmd(&mut self) -> Result<(), Infallible> {
        let end_command: &mut [u8] = &mut [ControlByte::End as u8];
        self.bus.borrow_mut().transfer(end_command).ok();
        Ok(())
    }

    fn get_byte(&mut self) -> Result<u8, Infallible> {
        let word_out = &mut [ControlByte::Dummy as u8];
        let word = self.bus.borrow_mut().transfer(word_out).ok().unwrap();
        Ok(word[0])
    }

    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, Error> {
        let retry_limit: u16 = 1000u16;

        for _ in 0..retry_limit {
            let byte_read = self.get_byte().ok().unwrap();
            if byte_read == ControlByte::Error as u8 {
                // consume remaining bytes after error: 0x00, 0xEE
                self.get_byte().ok();
                self.get_byte().ok();
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
            self.bus.borrow_mut().transfer(&mut [*byte]).ok();
        }
        Ok(())
    }

    fn send_param_length<P: NinaParam>(&mut self, param: &P) -> Result<(), Infallible> {
        let bytes = param.length_as_bytes();
        for byte in bytes.iter().take(param.length_size() as usize) {
            self.bus.borrow_mut().transfer(&mut [*byte]).ok();
        }
        Ok(())
    }

    fn pad_to_multiple_of_4(&mut self, mut command_size: u16) {
        while command_size % 4 != 0 {
            self.get_byte().ok();
            command_size += 1;
        }
    }

    // Accepts two separate bytes and packs them into 2 combined bytes as a u16
    // byte 0 is the LSB, byte1 is the MSB
    // See: https://en.wikipedia.org/wiki/Bit_numbering#LSB_0_bit_numbering
    fn combine_2_bytes(byte0: u8, byte1: u8) -> u16 {
        let word0: u16 = byte0 as u16;
        let word1: u16 = byte1 as u16;
        (word1 << 8) | (word0 & 0xff)
    }
}

#[cfg(test)]
mod spi_tests {
    use super::*;

    use crate::gpio::EspControlPins;
    use crate::Error;
    use core::cell::RefCell;
    use core::str;
    use embedded_hal::blocking::spi::Transfer;
    use embedded_hal::digital::v2::{InputPin, OutputPin, PinState};

    struct TransferMock {}

    impl Transfer<u8> for TransferMock {
        type Error = Error;
        fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
            Ok(words)
        }
    }

    struct OutputPinMock {}

    impl OutputPin for OutputPinMock {
        type Error = Error;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        fn set_state(&mut self, _state: PinState) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    struct InputPinMock {}

    impl InputPin for InputPinMock {
        type Error = Error;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(true)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    #[test]
    fn too_large_of_a_nina_param_throws_error() {
        let bytes = [0xA; 256];
        let str_slice: &str = str::from_utf8(&bytes).unwrap();

        let control_pins = EspControlPins {
            cs: OutputPinMock {},
            gpio0: OutputPinMock {},
            resetn: OutputPinMock {},
            ack: InputPinMock {},
        };

        let transfer_mock = TransferMock {};

        let mut protocol_handler = NinaProtocolHandler {
            bus: RefCell::new(transfer_mock),
            control_pins: control_pins,
        };

        let result = protocol_handler.set_passphrase(str_slice, "");

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }
}
