use super::Error;
use crate::{wifi::Wifi};

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;

use super::network::{IpAddress, Port, Socket, TransportMode};

use embedded_hal::blocking::spi::Transfer;

pub struct TcpClient<'a, B, C> {
    pub(crate) protocol_handler: &'a mut NinaProtocolHandler<B, C>,
    pub(crate) socket: Option<Socket>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) port: Port,
    pub(crate) mode: TransportMode,
    pub(crate) server_hostname: Option<&'a str>,
}

impl<'a, B, C> TcpClient<'a, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    pub fn build(wifi: &'a mut Wifi<B, C>) -> Self {
        Self {
            protocol_handler: wifi.protocol_handler.get_mut(),
            socket: None,
            server_ip_address: None,
            port: 0,
            mode: TransportMode::Tcp,
            server_hostname: None,
        }
    }

    pub fn connect<F>(mut self, ip: IpAddress, port: Port, mode: TransportMode, f: F) -> Self
    where
        F: Fn(&TcpClient<'a, B, C>)
    {
        let socket = self.get_socket().unwrap_or_default();
        self.socket = Some(socket);
        self.server_ip_address = Some(ip);
        self.port = port;
        self.mode = mode;

        self.protocol_handler.start_client(socket, ip, port, &mode).ok().unwrap();

        // TODO: utilize get_state() here to determine when we're connected to the remote TCP server

        f(&self);

        self.protocol_handler.stop_client(socket, &mode).ok().unwrap();

        self
    }

    pub fn server_ip_address(&self) -> Option<IpAddress> {
        self.server_ip_address
    }

    pub fn server_hostname(&self) -> Option<&'a str> {
        self.server_hostname
    }

    pub fn port(&self) -> Port {
        self.port
    }

    pub fn mode(&self) -> TransportMode {
        self.mode
    }

    pub fn get_socket(&mut self) -> Result<Socket, Error> {
        self.protocol_handler.get_socket()
    }

    pub fn socket(&self) -> Socket {
        self.socket.unwrap()
    }
}
