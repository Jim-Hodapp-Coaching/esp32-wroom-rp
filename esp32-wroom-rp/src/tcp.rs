use super::Error;

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;

use crate::wifi::Wifi;

use super::network::{IpAddress, Socket};

use embedded_hal::blocking::spi::Transfer;

pub struct Tcp<'a, 'b, B, C> {
    pub(crate) inner: TcpInner<NinaProtocolHandler<'a, B, C>>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) server_hostname: Option<&'b str>,
}

impl<'a, 'b, B, C> Tcp<'a, 'b, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    fn server_ip_address(mut self, ip: IpAddress) -> Self {
        self.server_ip_address = Some(ip);
        self
    }

    fn server_hostname(mut self, hostname: &'b str) -> Self {
        self.server_hostname = Some(hostname);
        self
    }

    pub fn get_socket(&mut self) -> Result<Socket, Error> {
        self.inner.get_socket()
    }

    pub fn demote_to_network(self) -> Wifi<'a, B, C> {
        Wifi::build(self.inner.protocol_handler)
    }
}

pub(crate) struct TcpInner<PH> {
    pub(crate) protocol_handler: PH,
}

impl<'a, PH> TcpInner<PH>
where
    PH: ProtocolInterface,
{
    fn get_socket(&mut self) -> Result<Socket, Error> {
        self.protocol_handler.get_socket()
    }
}
