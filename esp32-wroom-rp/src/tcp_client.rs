use super::Error;

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;

use super::network::{IpAddress, Socket};

use embedded_hal::blocking::spi::Transfer;

pub struct TcpClient<'a, B, C> {
    pub(crate) protocol_handler: &'a mut NinaProtocolHandler<B, C>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) server_hostname: Option<&'a str>,
}

impl<'a, B, C> TcpClient<'a, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    fn server_ip_address(mut self, ip: IpAddress) -> Self {
        self.server_ip_address = Some(ip);
        self
    }

    fn server_hostname(mut self, hostname: &'a str) -> Self {
        self.server_hostname = Some(hostname);
        self
    }

    pub fn get_socket(&mut self) -> Result<Socket, Error> {
        self.protocol_handler.get_socket()
    }
}
