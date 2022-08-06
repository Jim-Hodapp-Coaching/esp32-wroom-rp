//! # ESP32-WROOM-RP Pico Wireless Example
//!
//! This application demonstrates how to use the ESP32-WROOM-RP crate to talk to a remote
//! ESP32 wifi and retrieve its firmware version.
//!
//! See the `Cargo.toml` file for Copyright and license details.

#![no_std]
#![no_main]
#![allow(unused_variables)]

extern crate esp32_wroom_rp;

// The macro for our start-up function
use cortex_m_rt::entry;

use panic_probe as _;

/// Entry point to our bare-metal application.
///
/// The `#[entry]` macro ensures the Cortex-M start-up code calls this function
/// as soon as all global variables are initialized.
#[entry]
fn main() -> ! {
    loop {

    }
}