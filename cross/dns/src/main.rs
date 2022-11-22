//! # ESP32-WROOM-RP Pico Wireless Example
//!
//! This application demonstrates how to use the ESP32-WROOM-RP crate to perform
//! a DNS hostname lookup after setting what DNS server to use.
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
use fugit::RateExtU32;
use hal::clocks::Clock;
use hal::pac;

use esp32_wroom_rp::network::IpAddress;
use esp32_wroom_rp::wifi::ConnectionStatus;

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

    defmt::info!("ESP32-WROOM-RP DNS resolve example");

    // These are implicitly used by the spi driver if they are in the correct mode
    let _spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
    let _spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialized SPI driver for an initialized one
    let mut spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8.MHz(),
        &MODE_0,
    );

    let mut esp_pins = esp32_wroom_rp::gpio::EspControlPins {
        // CS on pin x (GPIO7)
        cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
        // GPIO0 on pin x (GPIO2)
        gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
        // RESETn on pin x (GPIO11)
        resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
        // ACK on pin x (GPIO10)
        ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
    };

    let mut wifi = esp32_wroom_rp::wifi::Wifi::init(&mut spi, &mut esp_pins, &mut delay).unwrap();

    let result = wifi.join(SSID, PASSPHRASE);
    defmt::info!("Join Result: {:?}", result);

    defmt::info!("Entering main loop");

    let mut sleep: u32 = 1500;
    loop {
        match wifi.get_connection_status() {
            Ok(status) => {
                defmt::info!("Get Connection Result: {:?}", status);
                delay.delay_ms(sleep);

                if status == ConnectionStatus::Connected {
                    defmt::info!("Connected to Network: {:?}", SSID);

                    // The IPAddresses of two DNS servers to resolve hostnames with.
                    // Note that failover from ip1 to ip2 is fully functional.
                    let ip1: IpAddress = [9, 9, 9, 9];
                    let ip2: IpAddress = [8, 8, 8, 8];
                    let dns_result = wifi.set_dns(ip1, Some(ip2));

                    defmt::info!("set_dns result: {:?}", dns_result);

                    let hostname = "github.com";
                    defmt::info!("Doing a DNS resolve for {}", hostname);

                    match wifi.resolve(hostname) {
                        Ok(ip) => {
                            defmt::info!("Server IP: {:?}", ip);
                        }
                        Err(e) => {
                            defmt::error!("Failed to resolve hostname {}", hostname);
                            defmt::error!("Err: {}", e);
                        }
                    }

                    wifi.leave().ok().unwrap();
                } else if status == ConnectionStatus::Disconnected {
                    defmt::info!("Disconnected from Network: {:?}", SSID);
                    sleep = 20000; // No need to loop as often after disconnecting
                }
            }
            Err(e) => {
                defmt::info!("Failed to Get Connection Result: {:?}", e);
            }
        }
    }
}
