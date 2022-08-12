//! # ESP32-WROOM-RP Pico Wireless Example
//!
//! This application demonstrates how to use the ESP32-WROOM-RP crate to communicate
//! with a remote ESP32 wifi and retrieve its firmware version.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]
#![allow(unused_variables)]

extern crate esp32_wroom_rp;

// The macro for our start-up function
use cortex_m_rt::entry;

// Needed for debug output symbols to be linked in binary image
use defmt_rtt as _;

use panic_probe as _;

// Alias for our HAL crate
use rp2040_hal as hal;

use embedded_hal_02::spi::MODE_0;
use embedded_time::rate::Extensions;
use rp2040_hal::clocks::Clock;
use rp2040_hal::pac;

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

    // The single-cycle I/O block controls our GPIO pins
    let sio = hal::Sio::new(pac.SIO);

    // Set the pins to their default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let uart_pins = (
        // UART TX (characters sent from RP2040) on pin 1 (GPIO0)
        pins.gpio0.into_mode::<hal::gpio::FunctionUart>(),
        // UART RX (characters reveived by RP2040) on pin 2 (GPIO1)
        pins.gpio1.into_mode::<hal::gpio::FunctionUart>(),
    );

    let uart = hal::uart::UartPeripheral::<_, _, _>::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            hal::uart::common_configs::_115200_8_N_1,
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    defmt::info!("ESP32-WROOM-RP get NINA firmware version example");

    // These are implicitly used by the spi driver if they are in the correct mode
    let spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialized SPI driver for an initialized one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),
        &MODE_0,
    );

    let esp_pins = esp32_wroom_rp::pins::EspControlPins {
        // CS on pin x (GPIO7)
        cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
        // GPIO0 on pin x (GPIO2)
        gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
        // RESETn on pin x (GPIO11)
        resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
        // ACK on pin x (GPIO10)
        ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
    };

    let mut wifi = esp32_wroom_rp::spi::Wifi::init(spi, esp_pins).unwrap();
    let firmware_version = wifi.firmware_version();
    defmt::info!("NINA firmware version: {:?}", firmware_version);

    loop {}
}
