use embedded_hal_mock::delay::MockNoop;
use embedded_hal_mock::spi;

use esp32_wroom_rp::network::{Hostname, IpAddress, Port, TransportMode};
use esp32_wroom_rp::tcp_client::{Connect, TcpClient};
use esp32_wroom_rp::wifi::Wifi;

pub mod support;

use support::*;

#[test]
fn successful_tcp_connection_with_hostname_invokes_closure() {
    // ----- get_socket -----

    let get_socket_command = 0x3f;
    let mut number_of_params = 0x0;
    let mut number_of_params_to_receive = 0x1;

    let mut expectations = mock_command(get_socket_command, number_of_params);

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        get_socket_command,
        number_of_params_to_receive,
        &[0x0],
    ));

    // ----- req_host_by_name -----

    let req_host_by_name_command = 0x34;
    number_of_params = 0x1;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(
        req_host_by_name_command,
        number_of_params,
    ));

    expectations.append(&mut mock_single_byte_size_params(4, 0x46)); // hostname is "FFFF"

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(3));

    expectations.append(&mut mock_receive(
        req_host_by_name_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    // ----- get_host_by_name -----

    let get_host_by_name_command = 0x35;
    number_of_params = 0x0;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(
        get_host_by_name_command,
        number_of_params,
    ));

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        get_host_by_name_command,
        number_of_params_to_receive,
        &[0x46, 0x46, 0x46, 0x46],
    ));

    // ----- start_client_tcp -----

    let start_client_tcp_command = 0x2d;
    number_of_params = 0x4;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(
        start_client_tcp_command,
        number_of_params,
    ));
    expectations.append(&mut mock_single_byte_size_params(4, 0x46)); // Send fake IP Address
    expectations.append(&mut mock_single_byte_size_params(2, 0x11)); // Send fake Port
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Transport Mode

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        start_client_tcp_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    let get_client_state_tcp_command = 0x2f;
    number_of_params = 0x1;

    expectations.append(&mut mock_command(
        get_client_state_tcp_command,
        number_of_params,
    ));

    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(2));

    expectations.append(&mut mock_receive(
        get_client_state_tcp_command,
        number_of_params_to_receive,
        &[0x4], // ConnectionState::Established
    ));

    let stop_client_tcp_command = 0x2e;
    number_of_params = 0x1;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(stop_client_tcp_command, number_of_params));

    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(2));

    expectations.append(&mut mock_receive(
        stop_client_tcp_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    let spi = spi::Mock::new(&expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();

    let hostname: Hostname = "FFFF";
    let port: Port = 0x1111;
    let mode: TransportMode = TransportMode::Tcp;

    // if the value is successfully updated inside the closure then
    // we know the closure was invoked.
    let mut value: u8 = 1;
    let test_value = &mut value;

    TcpClient::build(&mut wifi)
        .connect(hostname, port, mode, &mut delay, &mut |_tcp_client| {
            *test_value = 2
        })
        .unwrap();

    assert_eq!(value, 2);
}

#[test]
fn successful_tcp_connection_with_ip_address_invokes_closure() {
    // ----- get_socket -----

    let get_socket_command = 0x3f;
    let mut number_of_params = 0x0;
    let mut number_of_params_to_receive = 0x1;

    let mut expectations = mock_command(get_socket_command, number_of_params);

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        get_socket_command,
        number_of_params_to_receive,
        &[0x0],
    ));

    // ------ start_client_tcp ------

    let start_client_tcp_command = 0x2d;
    number_of_params = 0x4;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(
        start_client_tcp_command,
        number_of_params,
    ));
    expectations.append(&mut mock_single_byte_size_params(4, 0x40)); // Send fake IP Address
    expectations.append(&mut mock_single_byte_size_params(2, 0x11)); // Send fake Port
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Transport Mode

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        start_client_tcp_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    // ----- get_client_state_tcp -----

    let get_client_state_tcp_command = 0x2f;
    number_of_params = 0x1;

    expectations.append(&mut mock_command(
        get_client_state_tcp_command,
        number_of_params,
    ));

    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(2));

    expectations.append(&mut mock_receive(
        get_client_state_tcp_command,
        number_of_params_to_receive,
        &[0x4], // ConnectionState::Established
    ));

    let stop_client_tcp_command = 0x2e;
    number_of_params = 0x1;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(stop_client_tcp_command, number_of_params));

    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(2));

    expectations.append(&mut mock_receive(
        stop_client_tcp_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    let spi = spi::Mock::new(&expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();

    let ip_address: IpAddress = [0x40, 0x40, 0x40, 0x40];
    let port: Port = 0x1111;
    let mode: TransportMode = TransportMode::Tcp;

    // if the value is successfully updated inside the closure then
    // we know the closure was invoked.
    let mut value: u8 = 1;
    let test_value = &mut value;

    TcpClient::build(&mut wifi)
        .connect(ip_address, port, mode, &mut delay, &mut |_tcp_client| {
            *test_value = 2
        })
        .unwrap();

    assert_eq!(value, 2);
}

#[test]
fn tcp_connection_timeout_error() {
    // ----- get_socket -----
    let get_socket_command = 0x3f;
    let mut number_of_params = 0x0;
    let mut number_of_params_to_receive = 0x1;

    let mut expectations = mock_command(get_socket_command, number_of_params);

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        get_socket_command,
        number_of_params_to_receive,
        &[0x0],
    ));

    // ----- start_client_tcp -----

    let start_client_tcp_command = 0x2d;
    number_of_params = 0x4;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(
        start_client_tcp_command,
        number_of_params,
    ));
    expectations.append(&mut mock_single_byte_size_params(4, 0x40)); // Send fake IP Address
    expectations.append(&mut mock_single_byte_size_params(2, 0x11)); // Send fake Port
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket
    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Transport Mode

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_receive(
        start_client_tcp_command,
        number_of_params_to_receive,
        &[0x1],
    ));

    let get_client_state_tcp_command = 0x2f;
    number_of_params = 0x1;

    for _ in 0..10_000 {
        expectations.append(&mut mock_command(
            get_client_state_tcp_command,
            number_of_params,
        ));

        expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

        expectations.append(&mut mock_end_byte());

        expectations.append(&mut mock_padding(2));

        // wait_response_cmd()
        // read start command
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xe0]));
        // read command byte | reply byte
        expectations.push(spi::Transaction::transfer(
            vec![0xff],
            vec![command_or_reply_byte(get_client_state_tcp_command)],
        ));
        // read number of params to receive
        expectations.push(spi::Transaction::transfer(
            vec![0xff],
            vec![number_of_params_to_receive],
        ));
        // test relies on max number of parameters being 8. This will probably change
        // as we understand more.
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x8]));
        // read full 8 byte buffer
        // The first byte is the connection state. We only consider a 0x4 to be a successful state
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0x1]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xff]));
        // read end byte
        expectations.push(spi::Transaction::transfer(vec![0xff], vec![0xee]));
    }

    let stop_client_tcp = 0x2e;
    number_of_params = 0x1;
    number_of_params_to_receive = 0x1;

    expectations.append(&mut mock_command(stop_client_tcp, number_of_params));

    expectations.append(&mut mock_single_byte_size_params(1, 0x0)); // Send fake Socket

    expectations.append(&mut mock_end_byte());

    expectations.append(&mut mock_padding(2));

    expectations.append(&mut mock_receive(
        stop_client_tcp,
        number_of_params_to_receive,
        &[0x1],
    ));

    let spi = spi::Mock::new(&expectations);

    let mut delay = MockNoop::new();

    let pins = EspControlMock {};

    let mut wifi = Wifi::init(spi, pins, &mut delay).ok().unwrap();

    let ip_address: IpAddress = [0x40, 0x40, 0x40, 0x40];
    let port: Port = 0x1111;
    let mode: TransportMode = TransportMode::Tcp;

    let result = TcpClient::build(&mut wifi).connect(
        ip_address,
        port,
        mode,
        &mut delay,
        &mut |_tcp_client| {},
    );

    assert_eq!(
        result.unwrap_err(),
        esp32_wroom_rp::Error::Network(esp32_wroom_rp::network::NetworkError::ConnectionTimeout)
    );
}
