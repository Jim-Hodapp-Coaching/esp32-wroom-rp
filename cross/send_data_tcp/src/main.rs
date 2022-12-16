//! # ESP32-WROOM-RP Pico Wireless Example
//!
//! This application demonstrates how to use the ESP32-WROOM-RP crate to
//! send data to a remote server over TCP.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]

extern crate esp32_wroom_rp;

include!("../../secrets/secrets.rs");

// The macro for our start-up function
use cortex_m_rt::entry;

// Needed for debug output symbols to be linked in binary image
use defmt_rtt as _;

use panic_probe as _;

// Alias for our HAL crate
use rp2040_hal as hal;

use embedded_hal::spi::MODE_0;

use core::fmt::Write;

use fugit::RateExtU32;
use hal::gpio::{FloatingInput, PushPullOutput};
use hal::{clocks::Clock, pac};

use heapless::String;

use esp32_wroom_rp::{
    gpio::EspControlPins, network::IpAddress, network::Port, network::TransportMode,
    tcp_client::Connect, tcp_client::TcpClient, wifi::ConnectionStatus, wifi::Wifi,
};

const MAX_HTTP_DOC_LENGTH: usize = u8::MAX as usize;

/// The linker will place this boot block at the start of our program image. We
/// need this to help the ROM bootloader get our code up and running.
#[link_section = ".boot2"]
#[used]
pub static BOOT2: [u8; 256] = rp2040_boot2::BOOT_LOADER_W25Q080;

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
const XTAL_FREQ_HZ: u32 = 12_000_000u32;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialized.
#[entry]
fn main() -> ! {
    // Grab our singleton objects
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    // Set up the watchdog driver - needed by the clock setup code
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Configure the clocks
    let clocks = hal::clocks::init_clocks_and_plls(
        XTAL_FREQ_HZ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    defmt::info!("ESP32-WROOM-RP example to send data over TCP socket");

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialized SPI driver for an initialized one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8.MHz(),
        &MODE_0,
    );

    let esp_pins = EspControlPins {
        // CS on pin x (GPIO7)
        cs: pins.gpio7.into_mode::<PushPullOutput>(),
        // GPIO0 on pin x (GPIO2)
        gpio0: pins.gpio2.into_mode::<PushPullOutput>(),
        // RESETn on pin x (GPIO11)
        resetn: pins.gpio11.into_mode::<PushPullOutput>(),
        // ACK on pin x (GPIO10)
        ack: pins.gpio10.into_mode::<FloatingInput>(),
    };

    let mut wifi = Wifi::init(spi, esp_pins, &mut delay).unwrap();

    let result = wifi.join(SSID, PASSPHRASE);
    defmt::info!("Join Result: {:?}", result);

    defmt::info!("Entering main loop");

    let mut sleep: u32 = 1500;
    loop {
        match wifi.get_connection_status() {
            Ok(status) => {
                defmt::info!("Connection status: {:?}", status);
                delay.delay_ms(sleep);

                if status == ConnectionStatus::Connected {
                    defmt::info!("Connected to network: {:?}", SSID);

                    // The IPAddresses of two DNS servers to resolve hostnames with.
                    // Note that failover from ip1 to ip2 is fully functional.
                    let ip1: IpAddress = [9, 9, 9, 9];
                    let ip2: IpAddress = [8, 8, 8, 8];
                    let dns_result = wifi.set_dns(ip1, Some(ip2));

                    defmt::info!("set_dns result: {:?}", dns_result);

                    let _hostname = "github.com";

                    //let ip_address: IpAddress = [18, 195, 85, 27];
                    // let ip_address: IpAddress = [10, 0, 1, 3];
                    let ip_address: IpAddress = [140, 82, 114, 3]; // github.com
                    // let port: Port = 4000;
                    let port: Port = 443u16; // HTTPS
                    let mode: TransportMode = TransportMode::Tcp;

                    let mut http_document: String<MAX_HTTP_DOC_LENGTH> = String::from("");
                    write!(http_document, "GET / HTTP/1.1\r\nHost: {}.{}.{}.{}:{}\r\nAccept: */*\r\n\r\n",
                        ip_address[0],
                        ip_address[1],
                        ip_address[2],
                        ip_address[3],
                        port
                    );

                    if let Err(e) = TcpClient::build(&mut wifi).connect(
                        ip_address,
                        port,
                        mode,
                        &mut delay,
                        |tcp_client| {
                            defmt::info!(
                                "TCP connection to {:?}:{:?} successful",
                                ip_address,
                                port
                            );
                            defmt::info!("hostname: {:?}", tcp_client.server_hostname());
                            defmt::info!("Socket: {:?}", tcp_client.socket());
                            defmt::info!("Sending HTTP Document: {:?}", http_document.as_str());
                            tcp_client.send_data(&http_document);

                        },
                    ) {
                        defmt::error!(
                            "TCP connection to {:?}:{:?} failed: {:?}",
                            ip_address,
                            port,
                            e
                        );
                    }

                    defmt::info!("Leaving network: {:?}", SSID);
                    wifi.leave().ok();
                } else if status == ConnectionStatus::Disconnected {
                    sleep = 20000; // No need to loop as often after disconnecting
                }
            }
            Err(e) => {
                defmt::error!("Failed to get connection result: {:?}", e);
            }
        }
    }
}
