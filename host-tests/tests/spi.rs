use embedded_hal_mock::spi;
use embedded_hal_mock::pin::{
    Mock as PinMock, State as PinState, Transaction as PinTransaction,
};
use embedded_hal_mock::delay::MockNoop;

use esp32_wroom_rp::wifi::Wifi;
use esp32_wroom_rp::gpio::EspControlInterface;

struct EspControlMock {}

impl EspControlInterface for EspControlMock {
    fn init(&mut self) {}

    fn reset<D>(&mut self, _delay: &mut D) {}

    fn get_esp_ack(&self) -> bool {
        true
    }

   fn  wait_for_esp_select(&mut self) {}

    fn wait_for_esp_ack(&self) {}

    fn wait_for_esp_ready(&self) {}

    fn esp_select(&mut self) {}

    fn esp_deselect(&mut self) {}

    fn get_esp_ready(&self) -> bool {
        true
    }
}

#[test]
fn invalid_command_induces_nina_protocol_version_mismatch_error() {
    let spi_expectations = vec![
        // send_cmd()
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        spi::Transaction::transfer(vec![0x37], vec![0x0]),
        spi::Transaction::transfer(vec![0x0], vec![0x0]),
        spi::Transaction::transfer(vec![0xee], vec![0x0]),

        // wait_response_cmd()
        spi::Transaction::transfer(vec![0xff], vec![0xef]),
    ];
    let mut spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let mut pins = EspControlMock {};

    let mut wifi = Wifi::init(&mut spi, &mut pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(f.unwrap_err(), esp32_wroom_rp::Error::Protocol(esp32_wroom_rp::protocol::ProtocolError::NinaProtocolVersionMismatch));

    spi.done();
}
