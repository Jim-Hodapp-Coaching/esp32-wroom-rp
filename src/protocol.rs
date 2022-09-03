use super::*;

use embedded_hal::delay::blocking::DelayUs;

use heapless::{String, Vec};

pub const MAX_NINA_PARAM_LENGTH: usize = 255;

#[repr(u8)]
#[derive(Debug)]
pub enum NinaCommand {
    GetFwVersion = 0x37u8,
    SetPassphrase = 0x11u8,
    GetConnStatus = 0x20u8,
    Disconnect = 0x30u8,
}

pub trait NinaParam {
    // Length of parameter in bytes
    type LengthAsBytes: IntoIterator<Item = u8>;

    fn new(data: &str) -> Self;
    fn from_bytes(bytes: &[u8]) -> Self;

    fn data(&mut self) -> &[u8];

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes;
}

// Used for single byte params
pub struct NinaByteParam {
    length: u8,
    data: Vec<u8, 1>,
}

// Used for 2-byte params
pub struct NinaWordParam {
    length: u8,
    data: Vec<u8, 2>,
}

// Used for params that are smaller than 255 bytes
pub struct NinaSmallArrayParam {
    length: u8,
    data: Vec<u8, MAX_NINA_PARAM_LENGTH>,
}

// Used for params that can be larger than 255 bytes up to MAX_NINA_PARAM_LENGTH
pub struct NinaLargeArrayParam {
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
        data_as_bytes.extend_from_slice(bytes);
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn data(&mut self) -> &[u8] {
        self.data.as_slice()
    }

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes {
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
        data_as_bytes.extend_from_slice(bytes);
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn data(&mut self) -> &[u8] {
        self.data.as_slice()
    }

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes {
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
        data_as_bytes.extend_from_slice(bytes);
        Self {
            length: data_as_bytes.len() as u8,
            data: data_as_bytes,
        }
    }

    fn data(&mut self) -> &[u8] {
        self.data.as_slice()
    }

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes {
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
        data_as_bytes.extend_from_slice(bytes);
        Self {
            length: data_as_bytes.len() as u16,
            data: data_as_bytes,
        }
    }

    fn data(&mut self) -> &[u8] {
        self.data.as_slice()
    }

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes {
        [
            ((self.length & 0xff00) >> 8) as u8,
            (self.length & 0xff) as u8,
        ]
    }
}

pub trait ProtocolInterface {
    fn init(&mut self);
    fn reset<D: DelayUs>(&mut self, delay: &mut D);
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>;
    fn set_passphrase(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error>;
    fn disconnect(&mut self) -> Result<(), self::Error>;
    fn get_conn_status(&mut self) -> Result<u8, self::Error>;

    fn send_cmd(&mut self, cmd: NinaCommand, num_params: u8) -> Result<(), self::Error>;
    fn wait_response_cmd(
        &mut self,
        cmd: NinaCommand,
        num_params: u8,
    ) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], self::Error>;
    fn send_end_cmd(&mut self) -> Result<(), self::Error>;

    fn get_param(&mut self) -> Result<u8, self::Error>;
    fn read_byte(&mut self) -> Result<u8, self::Error>;
    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, self::Error>;
    fn check_start_cmd(&mut self) -> Result<bool, self::Error>;
    fn read_and_check_byte(&mut self, check_byte: u8) -> Result<bool, self::Error>;
    fn send_param<P: NinaParam>(&mut self, param: P) -> Result<(), self::Error>;
    fn send_param_length<P: NinaParam>(&mut self, param: &mut P) -> Result<(), self::Error>;
    fn pad_to_multiple_of_4(&mut self, command_size: u16);
}

#[derive(Debug, Default)]
pub struct NinaProtocolHandler<B, C> {
    /// A Spi or I2c instance
    pub bus: B,
    /// A EspControlPins instance
    pub control_pins: C,
}
