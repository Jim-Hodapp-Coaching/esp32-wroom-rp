pub mod operation;

use super::*;

use embedded_hal::blocking::delay::DelayMs;

use heapless::{String, Vec};

pub const MAX_NINA_PARAM_LENGTH: usize = 255;

#[repr(u8)]
#[derive(Copy, Clone, Debug)]
pub(crate) enum NinaCommand {
    GetFwVersion = 0x37u8,
    SetPassphrase = 0x11u8,
    GetConnStatus = 0x20u8,
    Disconnect = 0x30u8,
}

pub(crate) trait NinaParam {
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

impl NinaParam for NinaNoParams {
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

impl NinaParam for NinaByteParam {
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
        [self.length as u8]
    }
}

impl NinaParam for NinaWordParam {
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
        [self.length as u8]
    }
}

impl NinaParam for NinaSmallArrayParam {
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
        [self.length as u8]
    }
}

impl NinaParam for NinaLargeArrayParam {
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
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>;
    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error>;
    fn disconnect(&mut self) -> Result<(), self::Error>;
    fn get_conn_status(&mut self) -> Result<u8, self::Error>;
}

#[derive(Debug, Default)]
pub(crate) struct NinaProtocolHandler<B, C> {
    /// A Spi or I2c instance
    pub bus: B,
    /// An EspControlPins instance
    pub control_pins: C,
}

pub(crate) enum Error {
    NinaProtocolVersionMismatch,
    Timeout,
}
