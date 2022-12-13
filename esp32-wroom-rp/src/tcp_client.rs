use super::Error;
use crate::wifi::Wifi;

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;

use super::network::{ConnectionState, IpAddress, Port, Socket, TransportMode};

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use defmt::{write, Format, Formatter};

#[derive(Debug, Eq, PartialEq)]
pub enum TcpError {
    Timeout,
}

impl Format for TcpError {
    fn format(&self, fmt: Formatter) {
        match self {
            TcpError::Timeout => write!(fmt, "Timeout Connecting to TCP Server"),
        }
    }
}

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

    pub fn connect<F, D: DelayMs<u16>>(
        mut self,
        ip: IpAddress,
        port: Port,
        mode: TransportMode,
        delay: &mut D,
        f: F,
    ) -> Result<(), Error>
    where
        F: Fn(&TcpClient<'a, B, C>),
    {
        let socket = self.get_socket().unwrap_or_default();
        self.socket = Some(socket);
        self.server_ip_address = Some(ip);
        self.port = port;
        self.mode = mode;

        self.protocol_handler
            .start_client_tcp(socket, ip, port, &mode)
            .ok()
            .unwrap();

        // FIXME: without this delay, we'll frequently see timing issues and receive
        // a CmdResponseErr. We may not be handling busy/ack flag handling properly
        // and needs further investigation. I suspect that the ESP32 isn't ready to
        // receive another command yet. (copied this from POC)
        delay.delay_ms(250);

        let mut retry_limit = 10_000;

        while retry_limit > 0 {
            match self.protocol_handler.get_client_state_tcp(socket) {
                Ok(ConnectionState::Established) => {
                    f(&self);

                    self.protocol_handler
                        .stop_client_tcp(socket, &mode)
                        .ok()
                        .unwrap();
                    return Ok(());
                }
                Ok(status) => {
                    defmt::debug!("TCP Client Connection Status: {:?}", status);
                }
                Err(error) => {
                    // At this point any error will likely be a protocol level error.
                    // We do not currently consider any ConnectionState variants as errors.
                    defmt::debug!("TCP Client Connection Error: {:?}", error);
                    self.protocol_handler
                        .stop_client_tcp(socket, &mode)
                        .ok()
                        .unwrap();

                    return Err(error);
                }
            }
            delay.delay_ms(100);
            retry_limit -= 1;
        }

        self.protocol_handler
            .stop_client_tcp(socket, &mode)
            .ok()
            .unwrap();

        Err(TcpError::Timeout.into())
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
