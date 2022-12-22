use super::{Error, ARRAY_LENGTH_PLACEHOLDER};
use crate::wifi::Wifi;

use super::protocol::NinaProtocolHandler;
use crate::gpio::EspControlInterface;
use crate::protocol::ProtocolInterface;

use super::network::{ConnectionState, Hostname, IpAddress, Port, Socket, TransportMode};

use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use heapless::String;

use defmt::{write, Format, Formatter};

// TODO: find a good max length
const MAX_DATA_LENGTH: usize = 512;
const MAX_HOSTNAME_LENGTH: usize = 255;

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

/// Connect trait that allows for a `TcpClient` instance to connect to a remote
/// server by providing either a `Hostname` or an `IpAddress`. This trait also
/// makes it possible to implement and support IPv6 addresses.
pub trait Connect<'a, S, B, C> {
    /// Connects to `server` on `port` using transport layer `mode`.
    fn connect<F: FnMut(&mut TcpClient<'a, B, C>), D: DelayMs<u16>>(
        &mut self,
        server: S,
        port: Port,
        mode: TransportMode,
        delay: &mut D,
        f: &mut F,
    ) -> Result<(), Error>;
}

/// A client that connects to and performs send/receive operations with a remote
/// server using the TCP protocol.
pub struct TcpClient<'a, B, C> {
    pub(crate) protocol_handler: &'a mut NinaProtocolHandler<B, C>,
    pub(crate) socket: Option<Socket>,
    pub(crate) server_ip_address: Option<IpAddress>,
    pub(crate) port: Port,
    pub(crate) mode: TransportMode,
    pub(crate) server_hostname: Option<String<MAX_HOSTNAME_LENGTH>>,
}

impl<'a, B, C> Connect<'a, IpAddress, B, C> for TcpClient<'a, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    fn connect<F: FnMut(&mut TcpClient<'a, B, C>), D: DelayMs<u16>>(
        &mut self,
        ip: IpAddress,
        port: Port,
        mode: TransportMode,
        delay: &mut D,
        f: &mut F,
    ) -> Result<(), Error> {
        let socket = self.get_socket()?;
        self.socket = Some(socket);
        self.server_ip_address = Some(ip);
        self.server_hostname = Some(String::new());
        self.port = port;
        self.mode = mode;

        self.connect_common(delay, f)
    }
}

impl<'a, B, C> Connect<'a, Hostname<'_>, B, C> for TcpClient<'a, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    fn connect<F: FnMut(&mut TcpClient<'a, B, C>), D: DelayMs<u16>>(
        &mut self,
        server_hostname: Hostname,
        port: Port,
        mode: TransportMode,
        delay: &mut D,
        f: &mut F,
    ) -> Result<(), Error> {
        let socket = self.get_socket()?;
        self.socket = Some(socket);
        self.server_hostname = Some(server_hostname.into()); // into() makes a copy of the &str slice
        self.port = port;
        self.mode = mode;

        self.connect_common(delay, f)
    }
}

impl<'a, B, C> TcpClient<'a, B, C>
where
    B: Transfer<u8>,
    C: EspControlInterface,
{
    /// Builds a new instance of a `TcpClient` provided a `Wifi` instance.
    pub fn build(wifi: &'a mut Wifi<B, C>) -> Self {
        Self {
            protocol_handler: wifi.protocol_handler.get_mut(),
            socket: None,
            server_ip_address: None,
            port: 0,
            mode: TransportMode::Tcp,
            server_hostname: Some(String::new()),
        }
    }

    /// Returns `IpAddress` of the remote server to communicate with that is
    /// set by calling `connect()`.
    pub fn server_ip_address(&self) -> Option<IpAddress> {
        self.server_ip_address
    }

    /// Returns `Hostname` of the remote server to communicate with that is
    /// set by calling `connect()`.
    pub fn server_hostname(&self) -> &str {
        self.server_hostname.as_ref().unwrap().as_str()
    }

    /// Returns `Port` of the remote server to communicate with that is
    /// set by calling `connect()`.
    pub fn port(&self) -> Port {
        self.port
    }

    /// Returns `TransportMode` used in communication with the remote server that is
    /// set by calling `connect()`.
    pub fn mode(&self) -> TransportMode {
        self.mode
    }

    // TODO: Make this non-public
    pub fn get_socket(&mut self) -> Result<Socket, Error> {
        self.protocol_handler.get_socket()
    }

    // TODO: Make this non-public
    pub fn socket(&self) -> Socket {
        self.socket.unwrap()
    }

    /// Sends a string slice of data to a connected server.
    pub fn send_data(&mut self, data: &str) -> Result<[u8; ARRAY_LENGTH_PLACEHOLDER], Error> {
        self.protocol_handler
            .send_data(data, self.socket.unwrap_or_default())
    }

    // Provides the in-common connect() functionality used by the public interface's
    // connect(ip_address) or connect(hostname) instances.
    fn connect_common<F: FnMut(&mut TcpClient<'a, B, C>), D: DelayMs<u16>>(
        &mut self,
        delay: &mut D,
        mut f: F,
    ) -> Result<(), Error> {
        let socket = self.socket.unwrap_or_default();
        let mode = self.mode;
        let mut ip = self.server_ip_address.unwrap_or_default();
        let hostname = self.server_hostname.as_ref().unwrap();
        let port = self.port;

        if !hostname.is_empty() {
            ip = self
                .protocol_handler
                .resolve(hostname.as_str())
                .ok()
                .unwrap_or_default();
        }

        self.protocol_handler
            .start_client_tcp(socket, ip, port, &mode)?;

        // FIXME: without this delay, we'll frequently see timing issues and receive
        // a CmdResponseErr. We may not be handling busy/ack flag handling properly
        // and needs further investigation. I suspect that the ESP32 isn't ready to
        // receive another command yet. (copied this from POC)
        delay.delay_ms(250);

        let mut retry_limit = 10_000;

        while retry_limit > 0 {
            match self.protocol_handler.get_client_state_tcp(socket) {
                Ok(ConnectionState::Established) => {
                    f(self);

                    self.protocol_handler.stop_client_tcp(socket, &mode)?;

                    return Ok(());
                }
                Ok(_status) => {
                    delay.delay_ms(100);
                    retry_limit -= 1;
                }
                Err(error) => {
                    // At this point any error will likely be a protocol level error.
                    // We do not currently consider any ConnectionState variants as errors.
                    self.protocol_handler.stop_client_tcp(socket, &mode)?;

                    return Err(error);
                }
            }
        }

        self.protocol_handler.stop_client_tcp(socket, &mode)?;

        Err(TcpError::Timeout.into())
    }
}
