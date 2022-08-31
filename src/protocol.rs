use super::*;

use eh_02::blocking::spi::Transfer;
use embedded_hal::delay::blocking::DelayUs;

use heapless::{String, Vec};

//pub const PARAMS_ARRAY_LEN: usize = 8;
pub const MAX_NINA_PARAM_LENGTH: usize = 255;

#[repr(u8)]
#[derive(Debug)]
pub enum NinaCommand {
    StartClientTcp = 0x2Du8,
    GetFwVersion = 0x37u8,
    SetPassphrase = 0x11u8,
}

pub trait NinaParam {
    // The actual parameter data to send over the data bus
    type Data: IntoIterator<Item = u8>;

    // Length of parameter in bytes
    type LengthAsBytes: IntoIterator<Item = u8>;

    fn new(data: &str) -> Self
    where
        Self:;

    fn data(&mut self) -> &[u8];

    fn length_as_bytes(&mut self) -> Self::LengthAsBytes;
}

pub struct NinaByteParam {
    // length_size: u8,
    length: u8,
    // last_param: bool,
    data: Vec<u8, 1>,
}

pub struct NinaWordParam {
    // length_size: u8,
    length: u8,
    // last_param: bool,
    data: Vec<u8, 2>,
}

pub struct NinaArrayParam {
    // length_size: u8,
    length: u16,
    // last_param: bool,
    data: Vec<u8, MAX_NINA_PARAM_LENGTH>,
}

impl NinaParam for NinaByteParam {
    type Data = Vec<u8, 1>;
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> NinaByteParam {
        let data_as_bytes: Vec<u8, 1> = String::from(data).into_bytes();
        NinaByteParam {
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
    type Data = Vec<u8, 2>;
    type LengthAsBytes = [u8; 1];

    fn new(data: &str) -> NinaWordParam {
        let data_as_bytes: Vec<u8, 2> = String::from(data).into_bytes();
        NinaWordParam {
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

impl NinaParam for NinaArrayParam {
    type Data = Vec<u8, MAX_NINA_PARAM_LENGTH>;
    type LengthAsBytes = [u8; 2];

    fn new(data: &str) -> NinaArrayParam {
        let data_as_bytes: Vec<u8, MAX_NINA_PARAM_LENGTH> = String::from(data).into_bytes();
        NinaArrayParam {
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

    fn send_cmd(&mut self, cmd: NinaCommand, num_params: u8) -> Result<(), self::Error>;
    fn wait_response_cmd(
        &mut self,
        cmd: NinaCommand,
        num_params: u8,
    ) -> Result<[u8; 8], self::Error>;
    fn send_end_cmd(&mut self) -> Result<(), self::Error>;

    fn get_param(&mut self) -> Result<u8, self::Error>;
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
