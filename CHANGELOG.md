# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2022-01-12

Project related:
* First fully public release published to crates.io

Features list:
* Monitor WiFi network connection status (revisit)
* Send a valid TCP data stream to remote server (no receiving/parsing a response)
* Support the use of both IP addresses and host names for remote servers

Enhancements list:
* Address all current outstanding security audit issues: https://github.com/Jim-Hodapp-Coaching/esp32-wroom-rp/security/code-scanning


## [0.2.0] - 2022-10-27

New Features:
* Join a provided WiFi network (non-enterprise)
* Leave a WiFi network

Enhancements:
* Runnable test harness for unit and doctests
* Improve architecture for Nina protocol implementation functions
* Eliminate all current rustc compiler warnings
* Improve on error handling throughout existing code

## [0.1.0] - 2022-08-19

After a long and fun road through a [proof-of-concept](https://github.com/Jim-Hodapp-Coaching/esp32-pico-wifi) through to this simple but significant milestone, we are happy to announce a first unstable release of our embedded Rust crate `esp32-wroom-rp`. This provides the groundwork for significant WiFI functionality for the first generation RP2040 series feather boards provided by several different vendors.

Besides putting significant aspects of the software design and architecture in place, this release includes the following features:

### Feature list:

- Retrieves the current version of the NINA firmware on the ESP32 target
- Provides an example embedded program demonstrating retrieving the NINA firmware version