use super::Error;

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;
use crate::wifi::SPI_PROTOCOL_HANDLER;

use super::network::{IpAddress, Socket};

use embedded_hal::blocking::spi::Transfer;

use cortex_m::interrupt;

pub struct TcpClient<'a, 'b, B, C> {
    pub(crate) common: TcpClientCommon<'a, NinaProtocolHandler<'a, B, C>>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) server_hostname: Option<&'b str>,
}

impl<'a, 'b, B, C> TcpClient<'a, 'b, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    fn new() -> Self {
        Self {
            common: TcpClientCommon {
                // This is where we take a mutable reference via Mutex/RefCell
                protocol_handler: interrupt::free(|cs| SPI_PROTOCOL_HANDLER.borrow(cs)).borrow_mut().unwrap()
            },
            server_ip_address: None,
            server_hostname: None,
        }
    }

    fn server_ip_address(mut self, ip: IpAddress) -> Self {
        self.server_ip_address = Some(ip);
        self
    }

    fn server_hostname(mut self, hostname: &'b str) -> Self {
        self.server_hostname = Some(hostname);
        self
    }

    fn get_socket(&mut self) -> Result<Socket, Error> {
        self.common.get_socket()
    }
}

pub(crate) struct TcpClientCommon<'a, PH> {
    pub(crate) protocol_handler: &'a mut PH,
}

impl<'a, PH> TcpClientCommon<'a, PH>
where
    PH: ProtocolInterface,
{
    fn get_socket(&mut self) -> Result<Socket, Error> {
        self.protocol_handler.get_socket()
    }
}
