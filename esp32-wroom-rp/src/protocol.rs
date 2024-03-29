//! Defines functions, types and error definitions related to the WiFiNINA protocol communication specification.
//!

pub(crate) mod operation;

use core::cell::RefCell;

use defmt::{write, Format, Formatter};

use embedded_hal::blocking::delay::DelayMs;

use heapless::{String, Vec};

use super::network::{ConnectionState, IpAddress, Port, Socket, TransportMode};
use super::wifi::ConnectionStatus;
use super::{Error, FirmwareVersion};

// The maximum number of NINA param u8 bytes in a command send/receive byte stream
pub(crate) const MAX_NINA_PARAMS: usize = 8;

pub(crate) const MAX_NINA_BYTE_PARAM_BUFFER_LENGTH: usize = 1;
pub(crate) const MAX_NINA_WORD_PARAM_BUFFER_LENGTH: usize = 2;
pub(crate) const MAX_NINA_SMALL_ARRAY_PARAM_BUFFER_LENGTH: usize = 255;
pub(crate) const MAX_NINA_LARGE_ARRAY_PARAM_BUFFER_LENGTH: usize = 1024;

// The maximum length that a 2-byte length NINA response can be
pub(crate) const MAX_NINA_RESPONSE_LENGTH: usize = 1024;

// TODO: unalias this type and turn into a full wrapper struct
/// Provides a byte buffer to hold responses returned from NINA-FW
pub type NinaResponseBuffer = [u8; MAX_NINA_RESPONSE_LENGTH];

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum NinaCommand {
    SetPassphrase = 0x11u8,
    SetDNSConfig = 0x15u8,
    GetConnStatus = 0x20u8,
    StartClientTcp = 0x2du8,
    StopClientTcp = 0x2eu8,
    GetClientStateTcp = 0x2fu8,
    Disconnect = 0x30u8,
    ReqHostByName = 0x34u8,
    GetHostByName = 0x35u8,
    GetFwVersion = 0x37u8,
    GetSocket = 0x3fu8,
    SendDataTcp = 0x44,
}

pub(crate) trait NinaConcreteParam
where
    Self: core::marker::Sized,
{
    type DataBuffer;
    // Length of parameter in bytes
    type LengthAsBytes: IntoIterator<Item = u8>;

    fn new(data: &str) -> Result<Self, Error>;

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error>;

    fn data(&self) -> &[u8];

    fn length_as_bytes(&self) -> Self::LengthAsBytes;

    fn length(&self) -> u16;
}

pub(crate) trait NinaParam {
    fn length_as_bytes(&self) -> [u8; 2];
    fn data(&self) -> &[u8];
    fn length(&self) -> u16;
    fn length_size(&self) -> u8;
}

// Used for single byte params
#[derive(PartialEq, Debug)]
pub(crate) struct NinaByteParam {
    length: u8,
    data: <NinaByteParam as NinaConcreteParam>::DataBuffer,
}

// Used for 2-byte params
#[derive(PartialEq, Debug)]
pub(crate) struct NinaWordParam {
    length: u8,
    data: <NinaWordParam as NinaConcreteParam>::DataBuffer,
}

// Used for params that are smaller than 255 bytes
#[derive(PartialEq, Debug)]
pub(crate) struct NinaSmallArrayParam {
    length: u8,
    data: <NinaSmallArrayParam as NinaConcreteParam>::DataBuffer,
}

// Used for params that can be larger than 255 bytes up to MAX_NINA_PARAM_LENGTH
#[derive(PartialEq, Debug)]
pub(crate) struct NinaLargeArrayParam {
    length: u16,
    data: <NinaLargeArrayParam as NinaConcreteParam>::DataBuffer,
}

#[derive(PartialEq, Debug)]
pub(crate) struct NinaAbstractParam {
    // Byte representation of length of data
    length_as_bytes: [u8; 2],
    // Data to be transferred over SPI bus
    data: <NinaLargeArrayParam as NinaConcreteParam>::DataBuffer,
    // Number of bytes in data
    length: u16,
    // The number of bytes needed to represent
    // length_as_bytes
    length_size: u8,
}

impl NinaParam for NinaAbstractParam {
    fn length_as_bytes(&self) -> [u8; 2] {
        self.length_as_bytes
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn length(&self) -> u16 {
        self.length
    }

    fn length_size(&self) -> u8 {
        self.length_size
    }
}

impl From<NinaByteParam> for NinaAbstractParam {
    fn from(concrete_param: NinaByteParam) -> NinaAbstractParam {
        NinaAbstractParam {
            length_as_bytes: [concrete_param.length_as_bytes()[0], 0],
            data: Vec::from_slice(concrete_param.data()).unwrap(),
            length: concrete_param.length(),
            length_size: 1,
        }
    }
}

impl From<NinaWordParam> for NinaAbstractParam {
    fn from(concrete_param: NinaWordParam) -> NinaAbstractParam {
        NinaAbstractParam {
            length_as_bytes: [concrete_param.length_as_bytes()[0], 0],
            data: Vec::from_slice(concrete_param.data()).unwrap(),
            length: concrete_param.length(),
            length_size: 1,
        }
    }
}

impl From<NinaSmallArrayParam> for NinaAbstractParam {
    fn from(concrete_param: NinaSmallArrayParam) -> NinaAbstractParam {
        NinaAbstractParam {
            length_as_bytes: [concrete_param.length_as_bytes()[0], 0],
            data: Vec::from_slice(concrete_param.data()).unwrap(),
            length: concrete_param.length(),
            length_size: 1,
        }
    }
}

impl From<NinaLargeArrayParam> for NinaAbstractParam {
    fn from(concrete_param: NinaLargeArrayParam) -> NinaAbstractParam {
        NinaAbstractParam {
            length_as_bytes: concrete_param.length_as_bytes(),
            data: Vec::from_slice(concrete_param.data()).unwrap(),
            length: concrete_param.length(),
            length_size: 2,
        }
    }
}

impl NinaConcreteParam for NinaByteParam {
    type DataBuffer = Vec<u8, MAX_NINA_BYTE_PARAM_BUFFER_LENGTH>;
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Result<Self, Error> {
        if data.len() > MAX_NINA_BYTE_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let data_as_bytes: Self::DataBuffer = String::from(data).into_bytes();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_NINA_BYTE_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let mut data_as_bytes: Self::DataBuffer = Vec::new();
        data_as_bytes.extend_from_slice(bytes).unwrap_or_default();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn length(&self) -> u16 {
        self.length as u16
    }

    fn length_as_bytes(&self) -> Self::LengthAsBytes {
        [self.length]
    }
}

impl Default for NinaByteParam {
    fn default() -> Self {
        Self {
            length: 0,
            data: Vec::new(),
        }
    }
}

impl NinaConcreteParam for NinaWordParam {
    type DataBuffer = Vec<u8, MAX_NINA_WORD_PARAM_BUFFER_LENGTH>;
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Result<Self, Error> {
        if data.len() > MAX_NINA_WORD_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let data_as_bytes: Self::DataBuffer = String::from(data).into_bytes();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_NINA_WORD_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let mut data_as_bytes: Self::DataBuffer = Vec::new();
        data_as_bytes.extend_from_slice(bytes).unwrap_or_default();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn length(&self) -> u16 {
        self.length as u16
    }

    fn length_as_bytes(&self) -> Self::LengthAsBytes {
        [self.length]
    }
}

impl Default for NinaWordParam {
    fn default() -> Self {
        Self {
            length: 0,
            data: Vec::new(),
        }
    }
}

impl NinaConcreteParam for NinaSmallArrayParam {
    type DataBuffer = Vec<u8, MAX_NINA_SMALL_ARRAY_PARAM_BUFFER_LENGTH>;
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Result<Self, Error> {
        if data.len() > MAX_NINA_SMALL_ARRAY_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let data_as_bytes: Self::DataBuffer = String::from(data).into_bytes();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_NINA_SMALL_ARRAY_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let mut data_as_bytes: Self::DataBuffer = Vec::new();
        data_as_bytes.extend_from_slice(bytes).unwrap_or_default();
        Ok(Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        })
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn length(&self) -> u16 {
        self.length as u16
    }

    fn length_as_bytes(&self) -> Self::LengthAsBytes {
        [self.length]
    }
}

impl Default for NinaSmallArrayParam {
    fn default() -> Self {
        Self {
            length: 0,
            data: Vec::new(),
        }
    }
}

impl NinaConcreteParam for NinaLargeArrayParam {
    type DataBuffer = Vec<u8, MAX_NINA_LARGE_ARRAY_PARAM_BUFFER_LENGTH>;
    type LengthAsBytes = [u8; 2];

    fn new(data: &str) -> Result<Self, Error> {
        if data.len() > MAX_NINA_LARGE_ARRAY_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let data_as_bytes: Self::DataBuffer = String::from(data).into_bytes();
        Ok(Self {
            length: data_as_bytes.len() as u16,
            data: data_as_bytes,
        })
    }

    fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() > MAX_NINA_LARGE_ARRAY_PARAM_BUFFER_LENGTH {
            return Err(ProtocolError::PayloadTooLarge.into());
        }

        let mut data_as_bytes: Self::DataBuffer = Vec::new();
        data_as_bytes.extend_from_slice(bytes).unwrap_or_default();
        Ok(Self {
            length: data_as_bytes.len() as u16,
            data: data_as_bytes,
        })
    }

    fn data(&self) -> &[u8] {
        self.data.as_slice()
    }

    fn length(&self) -> u16 {
        self.length
    }

    fn length_as_bytes(&self) -> Self::LengthAsBytes {
        [
            ((self.length & 0xff00) >> 8) as u8,
            (self.length & 0xff) as u8,
        ]
    }
}

impl Default for NinaLargeArrayParam {
    fn default() -> Self {
        Self {
            length: 0,
            data: Vec::new(),
        }
    }
}

pub(crate) trait ProtocolInterface {
    fn init(&mut self);
    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D);
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, Error>;
    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error>;
    fn disconnect(&mut self) -> Result<(), Error>;
    fn get_conn_status(&mut self) -> Result<ConnectionStatus, Error>;
    fn set_dns_config(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error>;
    fn req_host_by_name(&mut self, hostname: &str) -> Result<u8, Error>;
    fn get_host_by_name(&mut self) -> Result<[u8; MAX_NINA_RESPONSE_LENGTH], Error>;
    fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error>;
    fn get_socket(&mut self) -> Result<Socket, Error>;
    fn start_client_tcp(
        &mut self,
        socket: Socket,
        ip: IpAddress,
        port: Port,
        mode: &TransportMode,
    ) -> Result<(), Error>;
    fn stop_client_tcp(&mut self, socket: Socket, _mode: &TransportMode) -> Result<(), Error>;
    fn get_client_state_tcp(&mut self, socket: Socket) -> Result<ConnectionState, Error>;
    fn send_data(&mut self, data: &str, socket: Socket) -> Result<[u8; 1], Error>;
}

#[derive(Debug)]
pub(crate) struct NinaProtocolHandler<B, C> {
    /// A Spi or I2c instance
    pub bus: RefCell<B>,
    /// An EspControlPins instance
    pub control_pins: C,
}

// TODO: look at Nina Firmware code to understand conditions
// that lead to NinaProtocolVersionMismatch
/// Errors related to communication with NINA firmware
#[derive(Debug, Eq, PartialEq)]
pub enum ProtocolError {
    /// TODO: look at Nina Firmware code to understand conditions
    /// that lead to NinaProtocolVersionMismatch
    NinaProtocolVersionMismatch,
    /// A timeout occurred.
    CommunicationTimeout,
    /// An invalid NINA command has been sent over the data bus.
    InvalidCommand,
    /// An invalid number of parameters sent over the data bus.
    InvalidNumberOfParameters,
    /// Too many parameters sent over the data bus.
    TooManyParameters,
    /// Payload is larger than the maximum buffer size allowed for transmission over
    /// the data bus.
    PayloadTooLarge,
}

impl Format for ProtocolError {
    fn format(&self, fmt: Formatter) {
        match self {
            ProtocolError::NinaProtocolVersionMismatch => write!(fmt, "Encountered an unsupported version of the NINA protocol."),
            ProtocolError::CommunicationTimeout => write!(fmt, "Communication with ESP32 target timed out."),
            ProtocolError::InvalidCommand => write!(fmt, "Encountered an invalid command while communicating with ESP32 target."),
            ProtocolError::InvalidNumberOfParameters => write!(fmt, "Encountered an unexpected number of parameters for a NINA command while communicating with ESP32 target."),
            ProtocolError::TooManyParameters => write!(fmt, "Encountered too many parameters for a NINA command while communicating with ESP32 target."),
            ProtocolError::PayloadTooLarge => write!(fmt, "The payload is larger than the max buffer size allowed for a NINA parameter while communicating with ESP32 target."),
        }
    }
}

#[cfg(test)]
mod protocol_tests {
    use super::*;
    use core::str;

    #[test]
    fn nina_byte_param_new_returns_payload_too_large_error_when_given_too_many_bytes() {
        let str_slice: &str = "too many bytes";
        let result = NinaByteParam::new(str_slice);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_byte_param_from_bytes_returns_payload_too_large_error_when_given_too_many_bytes() {
        let bytes: [u8; 2] = [0; 2];
        let result = NinaByteParam::from_bytes(&bytes);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_word_param_new_returns_payload_too_large_error_when_given_too_many_bytes() {
        let str_slice: &str = "too many bytes";
        let result = NinaWordParam::new(str_slice);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_word_param_from_bytes_returns_payload_too_large_error_when_given_too_many_bytes() {
        let bytes: [u8; 3] = [0; 3];
        let result = NinaWordParam::from_bytes(&bytes);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_small_array_param_new_returns_payload_too_large_error_when_given_too_many_bytes() {
        let bytes = [0xA; 256];
        let str_slice: &str = str::from_utf8(&bytes).unwrap();
        let result = NinaSmallArrayParam::new(str_slice);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_small_array_param_from_bytes_returns_payload_too_large_error_when_given_too_many_bytes()
    {
        let bytes: [u8; 256] = [0xA; 256];
        let result = NinaSmallArrayParam::from_bytes(&bytes);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_large_array_param_new_returns_payload_too_large_error_when_given_too_many_bytes() {
        let bytes = [0xA; 1025];
        let str_slice: &str = str::from_utf8(&bytes).unwrap();
        let result = NinaLargeArrayParam::new(str_slice);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }

    #[test]
    fn nina_large_array_param_from_bytes_returns_payload_too_large_error_when_given_too_many_bytes()
    {
        let bytes: [u8; 1025] = [0xA; 1025];
        let result = NinaLargeArrayParam::from_bytes(&bytes);

        assert_eq!(
            result.unwrap_err(),
            Error::Protocol(ProtocolError::PayloadTooLarge)
        )
    }
}
