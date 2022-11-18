use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use super::{Error, FirmwareVersion};

use super::gpio::EspControlInterface;
use super::protocol::{NinaProtocolHandler, ProtocolInterface};
use super::tcp_client::{TcpClient, TcpClientCommon};

use super::IpAddress;

/// Fundamental struct for controlling a connected ESP32-WROOM NINA firmware-based Wifi board.
#[derive(Debug)]
pub struct Wifi<'a, B, C> {
    common: WifiCommon<NinaProtocolHandler<'a, B, C>>,
}

impl<'a, S, C> Wifi<'a, S, C>
where
    S: Transfer<u8>,
    C: EspControlInterface,
{
    /// Initializes the ESP32-WROOM Wifi device.
    /// Calling this function puts the connected ESP32-WROOM device in a known good state to accept commands.
    pub fn init<D: DelayMs<u16>>(
        spi: &'a mut S,
        esp32_control_pins: &'a mut C,
        delay: &mut D,
    ) -> Result<Wifi<'a, S, C>, Error> {
        let mut wifi = Wifi {
            common: WifiCommon {
                protocol_handler: NinaProtocolHandler {
                    bus: spi,
                    control_pins: esp32_control_pins,
                },
            },
        };

        wifi.common.init(delay);
        Ok(wifi)
    }

    /// Retrieves the NINA firmware version contained on the connected ESP32-WROOM device (e.g. 1.7.4).
    pub fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        self.common.firmware_version()
    }

    /// Joins a WiFi network given an SSID and a Passphrase.
    pub fn join(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        self.common.join(ssid, passphrase)
    }

    /// Disconnects from a joined WiFi network.
    pub fn leave(&mut self) -> Result<(), Error> {
        self.common.leave()
    }

    /// Retrieves the current WiFi network connection status.
    ///
    /// NOTE: A future version will provide a enumerated type instead of the raw integer values
    /// from the NINA firmware.
    pub fn get_connection_status(&mut self) -> Result<u8, Error> {
        self.common.get_connection_status()
    }

    /// Sets 1 or 2 DNS servers that are used for network hostname resolution.
    pub fn set_dns(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error> {
        self.common.set_dns(dns1, dns2)
    }

    /// Queries the DNS server(s) provided via [set_dns] for the associated IP address to the provided hostname.
    pub fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error> {
        self.common.resolve(hostname)
    }

    pub fn build_tcp_client(&self) -> TcpClient<S, C> {
        TcpClient {
            common: TcpClientCommon {
                protocol_handler: &self.common.protocol_handler
            },
            server_ip_address: None,
            server_hostname: None
        }
    }
}

#[derive(Debug)]
struct WifiCommon<PH> {
    protocol_handler: PH,
}

impl<PH> WifiCommon<PH>
where
    PH: ProtocolInterface,
{
    fn init<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.protocol_handler.init();
        self.reset(delay);
    }

    fn reset<D: DelayMs<u16>>(&mut self, delay: &mut D) {
        self.protocol_handler.reset(delay)
    }

    fn firmware_version(&mut self) -> Result<FirmwareVersion, Error> {
        self.protocol_handler.get_fw_version()
    }

    fn join(&mut self, ssid: &str, passphrase: &str) -> Result<(), Error> {
        self.protocol_handler.set_passphrase(ssid, passphrase)
    }

    fn leave(&mut self) -> Result<(), Error> {
        self.protocol_handler.disconnect()
    }

    fn get_connection_status(&mut self) -> Result<u8, Error> {
        self.protocol_handler.get_conn_status()
    }

    fn set_dns(&mut self, dns1: IpAddress, dns2: Option<IpAddress>) -> Result<(), Error> {
        self.protocol_handler.set_dns_config(dns1, dns2)
    }

    fn resolve(&mut self, hostname: &str) -> Result<IpAddress, Error> {
        self.protocol_handler.resolve(hostname)
    }
}
