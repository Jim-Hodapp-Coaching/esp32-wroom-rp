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
    NinaProtocolHandler, NinaSmallArrayParam, NinaWordParam, ParamLengthSize, ProtocolError,
    ProtocolInterface, ResponseData, MAX_NINA_RESPONSE_LENGTH,
};
use super::wifi::ConnectionStatus;
use super::{Error, FirmwareVersion};

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

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        let (version, _) = result.split_at(5);

        Ok(FirmwareVersion::new(version)) // e.g. 1.7.3
    }

    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        let operation = Operation::new(NinaCommand::SetPassphrase)
            .param(NinaSmallArrayParam::new(ssid).into())
            .param(NinaSmallArrayParam::new(passphrase).into());

        self.execute(&operation)?;

        self.receive(&operation, 1, ParamLengthSize::OneByte)?;
        Ok(())
    }

    fn get_conn_status(&mut self) -> Result<ConnectionStatus, Error> {
        let operation = Operation::new(NinaCommand::GetConnStatus);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        Ok(ConnectionStatus::from(result[0]))
    }

    fn disconnect(&mut self) -> Result<(), Error> {
        let dummy_param = NinaByteParam::from_bytes(&[ControlByte::Dummy as u8]);
        let operation = Operation::new(NinaCommand::Disconnect).param(dummy_param.into());

        self.execute(&operation)?;

        self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        Ok(())
    }

    fn set_dns_config(&mut self, ip1: IpAddress, ip2: Option<IpAddress>) -> Result<(), Error> {
        // FIXME: refactor Operation so it can take different NinaParam types
        let operation = Operation::new(NinaCommand::SetDNSConfig)
            // FIXME: first param should be able to be a NinaByteParam:
            .param(NinaByteParam::from_bytes(&[1]).into())
            .param(NinaSmallArrayParam::from_bytes(&ip1).into())
            .param(NinaSmallArrayParam::from_bytes(&ip2.unwrap_or_default()).into());

        self.execute(&operation)?;

        self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        Ok(())
    }

    fn req_host_by_name(&mut self, hostname: &str) -> Result<u8, Error> {
        let operation = Operation::new(NinaCommand::ReqHostByName)
            .param(NinaSmallArrayParam::new(hostname).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;
        // defmt::debug!("Response: {:?}", result);
        if result[0] != 1u8 {
            return Err(NetworkError::DnsResolveFailed.into());
        }

        Ok(result[0])
    }

    fn get_host_by_name(&mut self) -> Result<ResponseData, Error> {
        let operation = Operation::new(NinaCommand::GetHostByName);

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

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

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        Ok(result[0])
    }

    fn start_client_tcp(
        &mut self,
        socket: Socket,
        ip: IpAddress,
        port: Port,
        mode: &TransportMode,
    ) -> Result<(), Error> {
        let port_as_bytes = Self::split_word_into_bytes(port);
        let operation = Operation::new(NinaCommand::StartClientTcp)
            .param(NinaSmallArrayParam::from_bytes(&ip).into())
            .param(NinaWordParam::from_bytes(&port_as_bytes).into())
            .param(NinaByteParam::from_bytes(&[socket]).into())
            .param(NinaByteParam::from_bytes(&[*mode as u8]).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;
        if result[0] == 1 {
            Ok(())
        } else {
            Err(NetworkError::ConnectFailed.into())
        }
    }

    // TODO: passing in TransportMode but not using, for now. It will become a way
    // of stopping the right kind of client (e.g. TCP, vs UDP)
    fn stop_client_tcp(&mut self, socket: Socket, _mode: &TransportMode) -> Result<(), Error> {
        let operation = Operation::new(NinaCommand::StopClientTcp)
            .param(NinaByteParam::from_bytes(&[socket]).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;
        if result[0] == 1 {
            Ok(())
        } else {
            Err(NetworkError::DisconnectFailed.into())
        }
    }

    fn get_client_state_tcp(&mut self, socket: Socket) -> Result<ConnectionState, Error> {
        let operation = Operation::new(NinaCommand::GetClientStateTcp)
            .param(NinaByteParam::from_bytes(&[socket]).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;
        // TODO: Determine whether or not any ConnectionState variants should be considered
        // an error.
        Ok(ConnectionState::from(result[0]))
    }

    fn send_data(&mut self, data: &str, socket: Socket) -> Result<ResponseData, Error> {
        let operation = Operation::new(NinaCommand::SendDataTcp)
            .param(NinaLargeArrayParam::from_bytes(&[socket]).into())
            .param(NinaLargeArrayParam::new(data).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        Ok(result)
    }

    fn receive_data(&mut self, socket: Socket) -> Result<ResponseData, Error> {
        let mut timeout: u16 = 10_000;
        let mut available_data_length: usize = 0;

        while timeout > 0 {
            available_data_length = self.avail_data_tcp(socket)?;
            if available_data_length > 0 {
                break;
            }

            timeout += 1;
        }

        self.get_data_buf_tcp(socket, available_data_length)
    }

    fn avail_data_tcp(&mut self, socket: Socket) -> Result<usize, Error> {
        let operation = Operation::new(NinaCommand::AvailDataTcp)
            .param(NinaByteParam::from_bytes(&[socket]).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::OneByte)?;

        let avail_data_length: usize = Self::combine_2_bytes(result[0], result[1]).into();

        Ok(avail_data_length)
    }

    fn get_data_buf_tcp(
        &mut self,
        socket: Socket,
        available_length: usize,
    ) -> Result<ResponseData, Error> {
        let available_length: [u8; 2] = Self::split_word_into_bytes(available_length as u16);
        let operation = Operation::new(NinaCommand::GetDataBufTcp)
            .param(NinaByteParam::from_bytes(&[socket]).into())
            .param(NinaWordParam::from_bytes(&available_length).into());

        self.execute(&operation)?;

        let result = self.receive(&operation, 1, ParamLengthSize::TwoByte)?;

        Ok(result)
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
        param_length_size: ParamLengthSize,
    ) -> Result<ResponseData, Error> {
        self.control_pins.wait_for_esp_select();

        let result =
            self.wait_response_cmd(&operation.command, expected_num_params, param_length_size);

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
            self.bus.borrow_mut().transfer(write_buf).ok();
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
        param_length_size: ParamLengthSize,
    ) -> Result<ResponseData, Error> {
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

        let response_length: usize = match param_length_size {
            ParamLengthSize::OneByte => self.get_one_byte_response_length().unwrap(),

            ParamLengthSize::TwoByte => self.get_two_byte_response_length().unwrap(),
        };

        let mut response_bytes: ResponseData = [0; MAX_NINA_RESPONSE_LENGTH];

        for i in 0..response_length {
            response_bytes[i] = self.get_byte().ok().unwrap()
        }
        let control_byte: u8 = ControlByte::End as u8;
        self.read_and_check_byte(&control_byte).ok();

        Ok(response_bytes)
    }

    fn get_one_byte_response_length(&mut self) -> Result<usize, Infallible> {
        Ok(self.get_byte().unwrap() as usize)
    }

    fn get_two_byte_response_length(&mut self) -> Result<usize, Infallible> {
        let bytes = (self.get_byte().unwrap(), self.get_byte().unwrap());
        Ok(Self::combine_2_bytes(bytes.0, bytes.1) as usize)
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
        let retry_limit: u32 = 1000000u32;

        for i in 0..retry_limit {
            if i % 6000 == 0 {
                let byte_read = self.get_byte().ok().unwrap();
                if byte_read == ControlByte::Error as u8 {
                    return Err(ProtocolError::NinaProtocolVersionMismatch.into());
                } else if byte_read == wait_byte {
                    return Ok(true);
                }
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

    fn split_word_into_bytes(word: u16) -> [u8; 2] {
        [((word & 0xff00) >> 8) as u8, (word & 0xff) as u8]
    }
}
