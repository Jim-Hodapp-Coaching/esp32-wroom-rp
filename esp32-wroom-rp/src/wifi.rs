use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::spi::Transfer;

use core::cell::RefCell;

use rp2040_hal as hal;

use super::{Error, FirmwareVersion};

use super::gpio::{EspControlInterface, EspControlPins};
use super::protocol::{NinaProtocolHandler, ProtocolInterface};
use super::tcp_client::{TcpClient, TcpClientCommon};

type Spi = hal::Spi<Enabled, pac::SPI0, 8>;
type Pins = EspControlPins<
    Pin<Gpio7, PushPullOutput>,
    Pin<Gpio2, PushPullOutput>,
    Pin<Gpio11, PushPullOutput>,
    Pin<Gpio10, FloatingInput>,
>;

use hal::gpio::{
    bank0::Gpio10, bank0::Gpio11, bank0::Gpio2, bank0::Gpio7, FloatingInput, Pin, PushPullOutput,
};

use hal::{pac, spi::Enabled};
use cortex_m::interrupt::{self, Mutex};

use super::IpAddress;

// This is where we set up the memory space but leave the internal struct as None.
// We have to define exactly what the types will be here so that the compiler knows
// how much memory we need. This means we can't have a generic PROTOCOL_HANDLER that would
// handle other types of data busses unless we figured out something fancy since we need
// to be explicite about Spi being a hal::Spi<Enabled, pac::SPI0, 8>.
pub static mut SPI_PROTOCOL_HANDLER: Mutex<RefCell<Option<NinaProtocolHandler<Spi, Pins>>>> = Mutex::new(RefCell::new(None));

/// Fundamental struct for controlling a connected ESP32-WROOM NINA firmware-based Wifi board.
pub struct Wifi<'a, B, C> {
    common: WifiCommon<NinaProtocolHandler<'a, B, C>>,
}

impl<'a> Wifi<'a, hal::Spi<Enabled, pac::SPI0, 8>, Pins>

{

    /// Initializes the ESP32-WROOM Wifi device.
    /// Calling this function puts the connected ESP32-WROOM device in a known good state to accept commands.
    pub fn init<D: DelayMs<u16>>(
        spi: &'a mut Spi,
        esp32_control_pins: &'a mut Pins,
        delay: &mut D,
    ) -> Result<Wifi<'a, Spi, Pins>, Error> {

        // This is where we replace the static memory space with what we actually want at runtime
         interrupt::free(|cs| SPI_PROTOCOL_HANDLER.borrow(cs).replace(Some(
            NinaProtocolHandler {
                bus: spi,
                control_pins: esp32_control_pins
            }
        )));
        let mut wifi = Wifi {
            common: WifiCommon {
                // This is where we take a mutable reference via Mutex/RefCell
                protocol_handler: interrupt::free(|cs| SPI_PROTOCOL_HANDLER.borrow(cs)).borrow_mut().unwrap()
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

    // pub fn build_tcp_client(&'a mut self) -> TcpClient<S, C> {
    //     TcpClient {
    //         common: TcpClientCommon {
    //             protocol_handler: &mut self.common.protocol_handler,
    //         },
    //         server_ip_address: None,
    //         server_hostname: None,
    //     }
    // }
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
