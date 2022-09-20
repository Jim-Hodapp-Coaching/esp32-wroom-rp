[![Build & Test Crate + Examples](https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/actions/workflows/build_and_test.yml/badge.svg)](https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/actions/workflows/build_and_test.yml)

# esp32-wroom-rp
A Rust-based RP2040 series driver providing WiFi functionality via Espressif ESP32-WROOM-32U/UE WiFi daughter controllers/boards.

Supports the [ESP32-WROOM-32E](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf), [ESP32-WROOM-32UE](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-32e_esp32-wroom-32ue_datasheet_en.pdf) modules.

Future implementations will support the [ESP32-WROOM-DA](https://www.espressif.com/sites/default/files/documentation/esp32-wroom-da_datasheet_en.pdf) module.

## Usage

```rust
use esp32_wroom_rp::wifi;
use embedded_hal::blocking::delay::DelayMs;

let spi_miso = pins.gpio16.into_mode::<hal::gpio::FunctionSpi>();
let spi_sclk = pins.gpio18.into_mode::<hal::gpio::FunctionSpi>();
let spi_mosi = pins.gpio19.into_mode::<hal::gpio::FunctionSpi>();

let spi = hal::Spi::<_, _, 8>::new(pac.SPI0);

Exchange the uninitialized SPI driver for an initialized one
let spi = spi.init(
    &mut pac.RESETS,
    clocks.peripheral_clock.freq(),
    8_000_000u32.Hz(),
    &MODE_0,
);

let esp_pins = esp32_wroom_rp::gpio::EspControlPins {
    // CS on pin x (GPIO7)
    cs: pins.gpio7.into_mode::<hal::gpio::PushPullOutput>(),
    // GPIO0 on pin x (GPIO2)
    gpio0: pins.gpio2.into_mode::<hal::gpio::PushPullOutput>(),
    // RESETn on pin x (GPIO11)
    resetn: pins.gpio11.into_mode::<hal::gpio::PushPullOutput>(),
    // ACK on pin x (GPIO10)
    ack: pins.gpio10.into_mode::<hal::gpio::FloatingInput>(),
};

let mut wifi = esp32_wroom_rp::spi::Wifi::init(spi, esp_pins, &mut delay).unwrap();
let version = wifi.firmware_version();
```

## Hardware

In order to run this code you need to purchase some hardware. This section provides a list of required hardware
needed at minimum, and some suggested items to make your life even easier.

### Required Hardware

1. [Raspberry Pi Pico with pre-soldered headers](https://www.elektor.com/raspberry-pi-pico-rp2040-with-pre-soldered-headers) (2x)
   * [Alternate distributors](https://www.raspberrypi.com/products/raspberry-pi-pico/)

2. Pimoroni Pico Wireless Pack (1x)
   * [US distributor](https://www.digikey.com/en/products/detail/pimoroni-ltd/PIM548/15851367)
   * [UK distributor](https://shop.pimoroni.com/products/pico-wireless-pack?variant=32369508581459)
   * [EU distributor](https://www.elektor.com/pimoroni-raspberry-pi-pico-wireless-pack)

3. [Breadboard](https://www.sparkfun.com/products/12614) (1x)
   * __Note__: If you already have a medium/large breadboard, then don't worry about purchasing this specific one


### Optional but Helpful Hardware

1. [Break Away Headers](https://www.sparkfun.com/products/116) (1x)
   * If you want to solder headers to the non-pre-soldered BME280 sensor board from #2 above

2. [Multi-length Jumper Wire Kit 140pcs](https://www.sparkfun.com/products/124) (1x)

3. [Straight 7" Jumper Wires M/M](https://www.sparkfun.com/products/11026) (1x)
   * Helpful to have some of these on hand

4. [Straight 6" Jumper Wires M/F](https://www.sparkfun.com/products/12794) (1x)
   * Helpful to have some of these on hand

5. [Saleae Logic 8](https://www.saleae.com/) (1x)
   * __Note__: Only needed if you'd like to participate in developing/debugging parts of this project that communicate
   on the SPI/I2C buses

### Wiring Details

Start with the section [Pico to Pico Wiring in this article](https://reltech.substack.com/p/getting-started-with-rust-on-a-raspberry?s=w) to set up using two Picos together, one as a Picoprobe (flash/debug) and the other as your embedded target.

Once properly wired, it should look similar to the following:

![IMG_3747](https://user-images.githubusercontent.com/3219120/159986814-37c99e4f-97cb-43c8-aa2f-1b325a1eb670.jpg)

![IMG_3746](https://user-images.githubusercontent.com/3219120/159986853-d1f84e01-1caa-4f0f-bc84-53ef79fa25b1.jpg)

__Pico to ESP32 WiFi__

The following table lists the pin name and pin number to properly wire between a Pico board and an ESP32 WiFi. This can be done on a breadboard such as the one listed above. Note that V+/- rail means the +/- columns on the breadboard for use as +5 VDC and GND respectively.

| Pico              | ESP32 WiFi       | Adafuit Airlift | Breadboard |
| ----------------- | ---------------- | ----------------| ---------- |
|                   | GND (Pin 3)      | GND (Pin 3)     | V- rail    |
| GP2 (Pin 4)       | GPIO0 (Pin 4)    | GP0 (Pin 10)    |            |
| GP7 (Pin 10)      | ESP_CSn (Pin 10) | CS (Pin 7)      |            |
| GP8 (Pin 11)      |                  |                 |            |
| GP9 (Pin 12)      |                  |                 |            |
| GP10 (Pin 14)     | ACK (Pin 14)     | Busy (Pin 8)    |            |
| GP11 (Pin 15)     | RESETn (Pin 15)  | RSTn (Pin 9)    |            |
| GP12 (Pin 16)     | SW_A (Pin 16)    | N/A             |            |
|                   | GND (Pin 18)     |                 | V- rail    |
| VBUS (Pin 40)     | VBUS (Pin 40)    |                 |            |
| VSYS (Pin 39)     | VSYS (Pin 39)    | VIN (Pin 1)     | V+ rail    |
| GND (Pin 38)      | GND (Pin 38)     |                 | V- rail    |
| 3V3(OUT) (Pin 36) | 3V3 (Pin 36)     | 3Vo (Pin 2)     |            |
| GP19 (Pin 25)     | MOSI (Pin 25)    | MOSI (Pin 5)    |            |
| GP18 (Pin 24)     | SCLK (Pin 24)    | SCK (Pin 4)     |            |
|                   | GND (Pin 23)     |                 | V- rail    |
| GP16 (Pin 21)     | MISO (Pin 21)    | MISO (Pin 5)    |            |


***

## Software Requirements
- The standard Rust tooling (cargo, rustup) which you can install from https://rustup.rs/

- Toolchain support for the cortex-m0+ processors in the rp2040 (thumbv6m-none-eabi)

- flip-link - this allows you to detect stack-overflows on the first core, which is the only supported target for now.

## Installation of development dependencies
```sh
rustup target install thumbv6m-none-eabi
cargo install flip-link
cargo install probe-run
```

## Building the crate and running the examples

To build the esp32-wroom-rp crate:
```sh
cargo build
```

To build all examples
```sh
cd cross
cargo build
```

To build a specific example (e.g. get_fw_version):
```sh
cd cross
cargo build --bin get_fw_version
```

To run a specific example (e.g. get_fw_version):
```sh
cd cross
cargo run --bin get_fw_version
```

## Running the crate's unit tests
```sh
cargo test
```

## Getting Involved

This project launched in April, 2022). See the main page section [Getting Involved](https://github.com/Jim-Hodapp-Coaching#getting-involved) for more info on how to contribute to this project and the Rust Never Sleeps community.

To get involved, please [request to join the community here on GitHub](https://rustneversleeps.wufoo.com/forms/z1x3dy1j0ycafxq/) and then start contributing to the [research and design discussions](https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/discussions) currently underway.

## Project Team

* Architect: [Caleb Bourg](https://github.com/calebbourg)
* Rust Developer: [Glyn Matthews](https://github.com/glynos)
* Project Oversight: [Jim Hodapp](https://github.com/jhodapp)
