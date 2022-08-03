//! # get_firmware_version Example
//! 
//! This example demonstrates use of the `esp32_wroom_rp::spi::Wifi.get_firmware_version()` function

#![no_std]
#![no_main]

// Alias for our HAL crate
use rp2040_hal as hal;
use rp2040_hal::pac;
use rp2040_hal::Clock;

use embedded_hal::spi::MODE_0;
use embedded_time::rate::Extensions;

use esp32_wroom_rp::spi::Wifi;
use esp32_wroom_rp::EspPins;
use esp32_wroom_rp::io_interface::IoInterfaceImpl;

const XTAL_FREQ_HZ: u32 = 12_000_000u32;

fn main() {

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

    let mut uart = hal::uart::UartPeripheral::<_, _, _>::new(pac.UART0, uart_pins, &mut pac.RESETS)
        .enable(
            hal::uart::common_configs::_115200_8_N_1,
            clocks.peripheral_clock.freq(),
        )
        .unwrap();

    // let mut delay = DelayWrap(cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().integer()));

    // These are implicitly used by the spi driver if they are in the correct mode
    let spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
    let spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
    let spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();

    let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

    // Exchange the uninitialised SPI driver for an initialised one
    let spi = spi.init(
        &mut pac.RESETS,
        clocks.peripheral_clock.freq(),
        8_000_000u32.Hz(),
        &MODE_0,
    );

    let esp_pins = EspPins {
        // Chip select is active-low, so we'll initialise it to a driven-high state
        cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
        gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
        resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
        ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
    };

    let io_interface: IoInterfaceImpl = IoInterfaceImpl {
        esp_pins: esp_pins
    };

    let wifi: Wifi<SPI, IO> = Wifi::init(spi, io_interface);
  
}