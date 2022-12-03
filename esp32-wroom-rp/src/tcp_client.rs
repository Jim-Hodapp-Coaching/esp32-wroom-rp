use super::Error;

use super::protocol::NinaProtocolHandler;
use super::protocol::ProtocolInterface;

use super::network::{IpAddress, Socket};

use crate::wifi::SPI_PROTOCOL_HANDLER;
use cortex_m::interrupt;

pub struct TcpClient<'b> {
    pub(crate) common: TcpClientCommon,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) server_hostname: Option<&'b str>,
}

impl<'b> TcpClient<'b>
{
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

pub(crate) struct TcpClientCommon {
    //pub(crate) protocol_handler: &'a mut PH,
}

impl TcpClientCommon
//where
//    PH: ProtocolInterface,
{
    fn get_socket(&mut self) -> Result<Socket, Error> {
        let mut result= Ok(0);
        interrupt::free(|cs| {
            let mut protocol_handler = SPI_PROTOCOL_HANDLER.borrow(cs).borrow_mut();
            result = protocol_handler.as_mut().unwrap().get_socket();
        });
        result
        //self.protocol_handler.get_socket()
    }
}
