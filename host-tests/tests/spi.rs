use embedded_hal_mock::delay::MockNoop;
use embedded_hal_mock::spi;

use esp32_wroom_rp::wifi::Wifi;

pub mod support;

use support::*;

#[test]
fn too_many_parameters_error() {
    let command = 0x37;
    let number_of_params = 0x0;
    let mut expectations = mock_command(command, number_of_params);

    let mut too_man_parameters_expectations = vec![
        // wait_response_cmd()
        // read start command
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        // read command byte | reply byte
        spi::Transaction::transfer(vec![0xff], vec![command_reply_byte(command)]),
        // read number of params to receive
        spi::Transaction::transfer(vec![0xff], vec![0x1]),
        // test relies on max number of parameters being 8. This will probably change
        // as we understand more.
        spi::Transaction::transfer(vec![0xff], vec![0x9]),
    ];

    expectations.append(&mut too_man_parameters_expectations);

    let spi = spi::Mock::new(&expectations);

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
    let command = 0x37;
    let number_of_params = 0x0;
    let mut expectations = mock_command(command, number_of_params);
    let mut invalid_number_of_parameters_expactations = vec![
        // wait_response_cmd()
        // read start command
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        // read command byte | reply byte
        spi::Transaction::transfer(vec![0xff], vec![command_reply_byte(command)]),
        // read number of params to receive (should be 1)
        spi::Transaction::transfer(vec![0xff], vec![0x0]),
    ];

    expectations.append(&mut invalid_number_of_parameters_expactations);

    let spi = spi::Mock::new(&expectations);

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
    let command = 0x37;
    let number_of_params = 0x0;
    let mut expectations = mock_command(command, number_of_params);
    let mut invalid_command_expactations = vec![
        // wait_response_cmd()
        // read start command
        spi::Transaction::transfer(vec![0xff], vec![0xe0]),
        // read command byte (should be command | reply byte)
        spi::Transaction::transfer(vec![0xff], vec![0xff]),
    ];
    expectations.append(&mut invalid_command_expactations);

    let spi = spi::Mock::new(&expectations);

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
    let command = 0x37;
    let number_of_params = 0x0;
    let mut expectations = mock_command(command, number_of_params);

    // simulate reading 1000 bytes which will exhaust the retry limit.
    for _ in 0..1000 {
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x0]))
    }

    let spi = spi::Mock::new(&expectations);

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
    let command = 0x37;
    let number_of_params = 0x0;
    let mut expectations = mock_command(command, number_of_params);
    let mut invalid_command_expactations = vec![
        // wait_response_cmd()
        // read start command (should be ee)
        spi::Transaction::transfer(vec![0xff], vec![0xef]),
    ];
    expectations.append(&mut invalid_command_expactations);

    let spi = spi::Mock::new(&expectations);

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
