use super::*;

use eh_02::blocking::spi::Transfer;
use embedded_hal::delay::blocking::DelayUs;

pub const PARAMS_ARRAY_LEN: usize = 8;

#[repr(u8)]
#[derive(Debug)]
pub enum NinaCommand {
    StartClientTcp = 0x2Du8,
    GetFwVersion = 0x37u8,
}

pub trait ProtocolInterface {
    fn init(&mut self);
    fn reset<D: DelayUs>(&mut self, delay: &mut D);
    fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>;

    fn send_cmd(&mut self, cmd: NinaCommand, num_params: u8) -> Result<(), self::Error>;
    fn wait_response_cmd(
        &mut self,
        cmd: NinaCommand,
        num_params: u8,
    ) -> Result<[u8; PARAMS_ARRAY_LEN], self::Error>;
    fn send_end_cmd(&mut self) -> Result<(), self::Error>;

    fn get_param(&mut self) -> Result<u8, self::Error>;
    fn wait_for_byte(&mut self, wait_byte: u8) -> Result<bool, self::Error>;
    fn check_start_cmd(&mut self) -> Result<bool, self::Error>;
    fn read_and_check_byte(&mut self, check_byte: u8) -> Result<bool, self::Error>;
}

#[derive(Debug, Default)]
pub struct NinaProtocolHandler<B, C> {
    /// A Spi or I2c instance
    pub bus: B,
    /// A EspControlPins instance
    pub control_pins: C,
}

// TODO:
// 1. Research default implementation for structs that spi/i2c impls can call as a parent

// Default impl (type of bus doesn't matter whether SPI or I2C)
// impl<BUS, CONTROL_PINS> ProtocolInterface for NinaProtocolHandler<BUS, CONTROL_PINS>
// {
//     fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>
//     {
//         Ok(FirmwareVersion::new([0x31, 0x2e, 0x37, 0x2e, 0x34, 0x0, 0x0, 0x0]))
//     }

//     fn send_cmd(&mut self, cmd: NinaCommand, num_params: u8) -> Result<(), self::Error>
//     {
//         Ok(())
//     }

//     fn send_end_cmd(&mut self) -> Result<(), self::Error> {
//         Ok(())
//     }
// }

// TODO: implement one of these for I2C in i2c.rs
// impl<SPI> ProtocolInterface for NinaProtocolHandler<SPI>
// where
//     SPI: Transfer<u8>,
// {
//     fn get_fw_version(&mut self) -> Result<FirmwareVersion, self::Error>
//     {
//         Ok(FirmwareVersion::new([0x31, 0x2e, 0x37, 0x2e, 0x34, 0x0, 0x0, 0x0]))
//     }
// }
