//! Defines functions, types and error definitions related to the WiFiNINA protocol communication specification.
//!

pub(crate) mod operation;

use core::cell::RefCell;

use defmt::{write, Format, Formatter};

use embedded_hal::blocking::delay::DelayMs;

use heapless::{String, Vec};

use super::network::{ConnectionState, IpAddress, Port, Socket, TransportMode};
use super::wifi::ConnectionStatus;
use super::{Error, FirmwareVersion, ARRAY_LENGTH_PLACEHOLDER};

pub(crate) const MAX_NINA_PARAM_LENGTH: usize = 4096;

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

pub(crate) trait NinaConcreteParam {
    // Length of parameter in bytes
    type LengthAsBytes: IntoIterator<Item = u8>;

    fn new(data: &str) -> Self;

    fn from_bytes(bytes: &[u8]) -> Self;

    fn data(&self) -> &[u8];

    fn length_as_bytes(&self) -> Self::LengthAsBytes;

    fn length(&self) -> u16;
}

// Used for Nina protocol commands with no parameters
pub(crate) struct NinaNoParams {
    _placeholder: u8,
}

impl NinaConcreteParam for NinaNoParams {
    type LengthAsBytes = [u8; 0];

    fn new(_data: &str) -> Self {
        Self { _placeholder: 0 }
    }

    fn from_bytes(_bytes: &[u8]) -> Self {
        Self { _placeholder: 0 }
    }

    fn data(&self) -> &[u8] {
        &[0u8]
    }

    fn length_as_bytes(&self) -> Self::LengthAsBytes {
        []
    }

    fn length(&self) -> u16 {
        0u16
    }
}

pub(crate) trait NinaParam {
    fn length_as_bytes(&self) -> [u8; 2];
    fn data(&self) -> &[u8];
    fn length(&self) -> u16;
    fn length_size(&self) -> u8;
}

// Used for single byte params
pub(crate) struct NinaByteParam {
    length: u8,
    data: Vec<u8, 1>,
}

// Used for 2-byte params
pub(crate) struct NinaWordParam {
    length: u8,
    data: Vec<u8, 2>,
}

// Used for params that are smaller than 255 bytes
pub(crate) struct NinaSmallArrayParam {
    length: u8,
    data: Vec<u8, MAX_NINA_PARAM_LENGTH>,
}

// Used for params that can be larger than 255 bytes up to MAX_NINA_PARAM_LENGTH
pub(crate) struct NinaLargeArrayParam {
    length: u16,
    data: Vec<u8, MAX_NINA_PARAM_LENGTH>,
}

pub(crate) struct NinaAbstractParam {
    // Byte representation of length of data
    length_as_bytes: [u8; 2],
    // Data to be transfered over SPI bus
    data: Vec<u8, MAX_NINA_PARAM_LENGTH>,
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

impl From<NinaNoParams> for NinaAbstractParam {
    fn from(concrete_param: NinaNoParams) -> NinaAbstractParam {
        NinaAbstractParam {
            length_as_bytes: [0, 0],
            data: Vec::from_slice(concrete_param.data()).unwrap(),
            length: concrete_param.length(),
            length_size: 0,
        }
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
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Self {
        let data_as_bytes: Vec<u8, 1> = String::from(data).into_bytes();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut data_as_bytes: Vec<u8, 1> = Vec::new();
        data_as_bytes.extend_from_slice(bytes).ok().unwrap();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
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

impl NinaConcreteParam for NinaWordParam {
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Self {
        let data_as_bytes: Vec<u8, 2> = String::from(data).into_bytes();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut data_as_bytes: Vec<u8, 2> = Vec::new();
        data_as_bytes.extend_from_slice(bytes).ok().unwrap();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
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

impl NinaConcreteParam for NinaSmallArrayParam {
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> Self {
        let data_as_bytes: Vec<u8, MAX_NINA_PARAM_LENGTH> = String::from(data).into_bytes();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut data_as_bytes: Vec<u8, MAX_NINA_PARAM_LENGTH> = Vec::new();
        data_as_bytes.extend_from_slice(bytes).ok().unwrap();
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
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

impl NinaConcreteParam for NinaLargeArrayParam {
    type LengthAsBytes = [u8; 2];

    fn new(data: &str) -> Self {
        let data_as_bytes: Vec<u8, MAX_NINA_PARAM_LENGTH> = String::from(data).into_bytes();
        Self {
            length: data_as_bytes.len() as u16,
            data: data_as_bytes,
        }
    }

    fn from_bytes(bytes: &[u8]) -> Self {
        let mut data_as_bytes: Vec<u8, MAX_NINA_PARAM_LENGTH> = Vec::new();
        data_as_bytes.extend_from_slice(bytes).ok().unwrap();
        Self {
            length: data_as_bytes.len() as u16,
            data: data_as_bytes,
        }
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

pub(crate) trait ProtocolInterface {
    fn init(&mut self);
    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D);
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, Error>;
    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error>;
    fn disconnect(&mut self) -> Result<(), Error>;
    fn get_conn_status(&mut self) -> Result<ConnectionStatus, Error>;
    fn set_dns_config(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error>;
    fn req_host_by_name(&mut self, hostname: &str) -> Result<u8, Error>;
    fn get_host_by_name(&mut self) -> Result<[u8; 8], Error>;
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
    fn send_data(
        &mut self,
        data: &str,
        socket: Socket,
    ) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], Error>;
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
}

impl Format for ProtocolError {
    fn format(&self, fmt: Formatter) {
        match self {
            ProtocolError::NinaProtocolVersionMismatch => write!(fmt, "Encountered an unsupported version of the NINA protocol."),
            ProtocolError::CommunicationTimeout => write!(fmt, "Communication with ESP32 target timed out."),
            ProtocolError::InvalidCommand => write!(fmt, "Encountered an invalid command while communicating with ESP32 target."),
            ProtocolError::InvalidNumberOfParameters => write!(fmt, "Encountered an unexpected number of parameters for a NINA command while communicating with ESP32 target."),
            ProtocolError::TooManyParameters => write!(fmt, "Encountered too many parameters for a NINA command while communicating with ESP32 target.")
        }
    }
}
