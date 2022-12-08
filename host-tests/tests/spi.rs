use embedded_hal_mock::delay::MockNoop;
use embedded_hal_mock::spi;

use esp32_wroom_rp::gpio::EspControlInterface;
use esp32_wroom_rp::wifi::Wifi;

struct EspControlMock {}

impl EspControlInterface for EspControlMock {
    fn init(&mut self) {}

    fn reset<D>(&mut self, _delay: &mut D) {}

    fn get_esp_ack(&self) -> bool {
        true
    }

    fn wait_for_esp_select(&mut self) {}

    fn wait_for_esp_ack(&self) {}

    fn wait_for_esp_ready(&self) {}

    fn esp_select(&mut self) {}

    fn esp_deselect(&mut self) {}

    fn get_esp_ready(&self) -> bool {
        true
    }
}

#[test]
fn too_many_parameters_error() {
    let spi_expectations = vec![
        // send_cmd()
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        spi::Transaction::transfer(vec![0x37], vec![0x0]),
        spi::Transaction::transfer(vec![0x0], vec![0x0]),
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
        // wait_response_cmd()
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        spi::Transaction::transfer(vec![0xff], vec![0xb7]),
        spi::Transaction::transfer(vec![0xff], vec![0x1]),
        // test relies on max number of parameters being 8. This will probably change
        // as we understand more.
        spi::Transaction::transfer(vec![0xff], vec![0x9]),
    ];
    let spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(
        f.unwrap_err(),
        esp32_wroom_rp::Error::Protocol(esp32_wroom_rp::protocol::ProtocolError::TooManyParameters)
    );

    wifi.destroy().done();
}

#[test]
fn invalid_number_of_parameters_error() {
    let spi_expectations = vec![
        // send_cmd()
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        spi::Transaction::transfer(vec![0x37], vec![0x0]),
        spi::Transaction::transfer(vec![0x0], vec![0x0]),
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
        // wait_response_cmd()
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        spi::Transaction::transfer(vec![0xff], vec![0xb7]),
        spi::Transaction::transfer(vec![0xff], vec![0x0]),
    ];
    let spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(
        f.unwrap_err(),
        esp32_wroom_rp::Error::Protocol(
            esp32_wroom_rp::protocol::ProtocolError::InvalidNumberOfParameters
        )
    );

    wifi.destroy().done();
}

#[test]
fn invalid_command_induces_invalid_command_error() {
    let spi_expectations = vec![
        // send_cmd()
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        spi::Transaction::transfer(vec![0x37], vec![0x0]),
        spi::Transaction::transfer(vec![0x0], vec![0x0]),
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
        // wait_response_cmd()
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        spi::Transaction::transfer(vec![0xff], vec![0x0]),
    ];
    let spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(
        f.unwrap_err(),
        esp32_wroom_rp::Error::Protocol(esp32_wroom_rp::protocol::ProtocolError::InvalidCommand)
    );

    wifi.destroy().done();
}

#[test]
fn timeout_induces_communication_timeout_error() {
    let mut spi_expectations = vec![
        // send_cmd()
        spi::Transaction::transfer(vec![0xe0], vec![0x0]),
        spi::Transaction::transfer(vec![0x37], vec![0x0]),
        spi::Transaction::transfer(vec![0x0], vec![0x0]),
        spi::Transaction::transfer(vec![0xee], vec![0x0]),
    ];

    // simulate reading 1000 bytes which will exhaust the retry limit.
    for _ in 0..1000 {
        spi_expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x0]))
    }

    let spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(
        f.unwrap_err(),
        esp32_wroom_rp::Error::Protocol(
            esp32_wroom_rp::protocol::ProtocolError::CommunicationTimeout
        )
    );

    wifi.destroy().done();
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
    let spi = spi::Mock::new(&spi_expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();
    let f = wifi.firmware_version();

    assert_eq!(
        f.unwrap_err(),
        esp32_wroom_rp::Error::Protocol(
            esp32_wroom_rp::protocol::ProtocolError::NinaProtocolVersionMismatch
        )
    );

    wifi.destroy().done();
}
